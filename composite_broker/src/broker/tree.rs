pub mod tree {
    use mqtt_v5::topic::{Topic, TopicFilter, TopicLevel};
    use std::collections::{hash_map::Entry, HashMap};

    // TODO(bschwind) - Support shared subscriptions

    #[derive(Debug)]
    pub struct SubscriptionTreeNode<T> {
        subscribers: Vec<(u64, T)>,
        single_level_wildcards: Option<Box<SubscriptionTreeNode<T>>>,
        multi_level_wildcards: Vec<(u64, T)>,
        concrete_topic_levels: HashMap<String, SubscriptionTreeNode<T>>,
    }

    #[derive(Debug)]
    pub struct SubscriptionTree<T> {
        root: SubscriptionTreeNode<T>,
        counter: u64,
    }

    impl<T: std::fmt::Debug> SubscriptionTree<T> {
        pub fn new() -> Self {
            Self { root: SubscriptionTreeNode::new(), counter: 0 }
        }

        pub fn insert(&mut self, topic_filter: &TopicFilter, value: T) -> u64 {
            let counter = self.counter;
            self.root.insert(topic_filter, value, counter);
            self.counter += 1;

            counter
        }

        #[allow(dead_code)]
        pub fn matching_subscribers(&self, topic: &Topic) -> impl Iterator<Item = &T> {
            self.root.matching_subscribers(topic)
        }

        #[allow(dead_code)]
        pub fn remove(&mut self, topic_filter: &TopicFilter, counter: u64) -> Option<T> {
            self.root.remove(topic_filter, counter)
        }

        #[allow(dead_code)]
        fn is_empty(&self) -> bool {
            self.root.is_empty()
        }
    }

    impl<T: std::fmt::Debug> SubscriptionTreeNode<T> {
        fn new() -> Self {
            Self {
                subscribers: Vec::new(),
                single_level_wildcards: None,
                multi_level_wildcards: Vec::new(),
                concrete_topic_levels: HashMap::new(),
            }
        }

        fn is_empty(&self) -> bool {
            self.subscribers.is_empty()
                && self.single_level_wildcards.is_none()
                && self.multi_level_wildcards.is_empty()
                && self.concrete_topic_levels.is_empty()
        }

        fn insert(&mut self, topic_filter: &TopicFilter, value: T, counter: u64) {
            let mut current_tree = self;
            let mut multi_level = false;

            for level in topic_filter.levels() {
                match level {
                    TopicLevel::SingleLevelWildcard => {
                        if current_tree.single_level_wildcards.is_none() {
                            current_tree.single_level_wildcards =
                                Some(Box::new(SubscriptionTreeNode::new()));
                        }

                        current_tree = current_tree.single_level_wildcards.as_mut().unwrap();
                    },
                    TopicLevel::MultiLevelWildcard => {
                        multi_level = true;
                        break;
                    },
                    TopicLevel::Concrete(concrete_topic_level) => {
                        if !current_tree.concrete_topic_levels.contains_key(concrete_topic_level) {
                            current_tree
                                .concrete_topic_levels
                                .insert(concrete_topic_level.to_string(), SubscriptionTreeNode::new());
                        }

                        // TODO - Do this without another hash lookup
                        current_tree =
                            current_tree.concrete_topic_levels.get_mut(concrete_topic_level).unwrap();
                    },
                    // _ => {}
                }
            }

            if multi_level {
                current_tree.multi_level_wildcards.push((counter, value));
            } else {
                current_tree.subscribers.push((counter, value));
            }
        }

        fn remove(&mut self, topic_filter: &TopicFilter, counter: u64) -> Option<T> {
            let mut current_tree = self;
            let mut stack: Vec<(*mut SubscriptionTreeNode<T>, usize)> = vec![];

            let levels: Vec<TopicLevel> = topic_filter.levels().collect();
            let mut level_index = 0;

            for level in &levels {
                match level {
                    TopicLevel::SingleLevelWildcard => {
                        if current_tree.single_level_wildcards.is_some() {
                            stack.push((&mut *current_tree, level_index));
                            level_index += 1;

                            current_tree = current_tree.single_level_wildcards.as_mut().unwrap();
                        } else {
                            return None;
                        }
                    },
                    TopicLevel::MultiLevelWildcard => {
                        break;
                    },
                    TopicLevel::Concrete(concrete_topic_level) => {
                        if current_tree.concrete_topic_levels.contains_key(*concrete_topic_level) {
                            stack.push((&mut *current_tree, level_index));
                            level_index += 1;

                            current_tree = current_tree
                                .concrete_topic_levels
                                .get_mut(*concrete_topic_level)
                                .unwrap();
                        } else {
                            return None;
                        }
                    },
                    // _ => {}
                }
            }

            // Get the return value
            let return_val = {
                let level = &levels[levels.len() - 1];

                if *level == TopicLevel::MultiLevelWildcard {
                    if let Some(pos) =
                        current_tree.multi_level_wildcards.iter().position(|(c, _)| *c == counter)
                    {
                        Some(current_tree.multi_level_wildcards.remove(pos))
                    } else {
                        None
                    }
                } else if let Some(pos) =
                    current_tree.subscribers.iter().position(|(c, _)| *c == counter)
                {
                    Some(current_tree.subscribers.remove(pos))
                } else {
                    None
                }
            };

            // Go up the stack, cleaning up empty nodes
            while let Some((stack_val, level_index)) = stack.pop() {
                let mut tree = unsafe { &mut *stack_val };

                let level = &levels[level_index];

                match level {
                    TopicLevel::SingleLevelWildcard => {
                        if tree.single_level_wildcards.as_ref().map(|t| t.is_empty()).unwrap_or(false) {
                            tree.single_level_wildcards = None;
                        }
                    },
                    TopicLevel::MultiLevelWildcard => {
                        // TODO - Ignore this case?
                    },
                    TopicLevel::Concrete(concrete_topic_level) => {
                        if let Entry::Occupied(o) =
                            tree.concrete_topic_levels.entry((*concrete_topic_level).to_string())
                        {
                            if o.get().is_empty() {
                                o.remove_entry();
                            }
                        }
                    },
                    // _ => {}
                }
            }

            return_val.map(|(_, val)| val)
            // Some(return_val)
        }

        fn matching_subscribers(&self, topic: &Topic) -> impl Iterator<Item = &T> {
            let mut subscriptions = Vec::new();
            let mut tree_stack = vec![(self, 0)];
            let levels: Vec<TopicLevel> = topic.levels().collect();

            while !tree_stack.is_empty() {
                let (current_tree, current_level) = tree_stack.pop().unwrap();
                let level = &levels[current_level];

                // Don't allow wildcard subscribers to receive messages
                // with leading dollar signs, like '$SYS/stats'
                // if current_level != 0 || !level.has_leading_dollar() {
                //     subscriptions.extend(
                //         current_tree.multi_level_wildcards.iter().map(|(_, subscriber)| subscriber),
                //     );
                // }

                // if let Some(sub_tree) = &current_tree.single_level_wildcards {
                //     // Don't allow wildcard subscribers to receive messages
                //     // with leading dollar signs, like '$SYS/stats'
                //     if current_level != 0 || !level.has_leading_dollar() {
                //         if current_level + 1 < levels.len() {
                //             tree_stack.push((sub_tree, current_level + 1));
                //         } else {
                //             subscriptions
                //                 .extend(sub_tree.subscribers.iter().map(|(_, subscriber)| subscriber));
                //         }
                //     }
                // }

                if let TopicLevel::Concrete(level) = level {
                    if current_tree.concrete_topic_levels.contains_key(*level) {
                        let sub_tree = current_tree.concrete_topic_levels.get(*level).unwrap();

                        if current_level + 1 < levels.len() {
                            let sub_tree = current_tree.concrete_topic_levels.get(*level).unwrap();
                            tree_stack.push((sub_tree, current_level + 1));
                        } else {
                            subscriptions
                                .extend(sub_tree.subscribers.iter().map(|(_, subscriber)| subscriber));

                            // TODO(bschwind) - Verify this works properly with better tests.
                            subscriptions.extend(
                                sub_tree.multi_level_wildcards.iter().map(|(_, subscriber)| subscriber),
                            );
                        }
                    }
                }
            }
            subscriptions.into_iter()
        }
    }
}
