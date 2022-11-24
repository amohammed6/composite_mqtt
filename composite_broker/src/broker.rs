mod tree;
// use tree::tree::SubscriptionTree;
pub mod broker {
    // use std::ops::Sub;

    // broker function
    use mqtt_v5::{
        topic::TopicFilter,
        types::{
        properties::AssignedClientIdentifier, //PublishPacket, PublishAckPacket
        ConnectAckPacket,
        ConnectPacket,
        ConnectReason, SubscribePacket, SubscribeAckPacket, SubscribeAckReason,
        }
    };

    use super::tree::tree::SubscriptionTree;

    // global ds
    // 1-level subscriptions
    // specified type T (for subscriptions)
    // client_id
    #[derive(Debug)]
    struct Subs {
        client_id: String,
        // active: bool,
    }
    // impl <T> Iterator for Subs<> where T: fmt::Display {
    //     type Item = T;
    //     type IntoIter = vec_deque::IntoIter<T>;
    //     fn into_iter(self) -> Self::IntoIter {
    //         self.store.into_iter()
    //     }
    // }

    // clients (may be handled by session?)
    pub struct MBroker {
        clients: Vec<Subs>,
        subscriptions: SubscriptionTree<Subs>, // use as subscriptions list
        unsub_list: Vec<TopicFilter>,   // use for unsubscriptions list
    }
    impl MBroker {
        pub fn new() -> Self {
            Self {
                subscriptions: SubscriptionTree::new(),
                unsub_list: Vec::new(),
                clients: Vec::new()
            }
        }
        // receive connect packet
        pub fn accept_new_client(&mut self, connect_packet: ConnectPacket) -> ConnectAckPacket {
            // create connect_ack packet
            let conn_ack = ConnectAckPacket {
                session_present: true,
                reason_code: ConnectReason::Success,
                session_expiry_interval: None,
                receive_maximum: None,
                maximum_qos: None,
                retain_available: None,
                maximum_packet_size: None,
                assigned_client_identifier: Some(AssignedClientIdentifier(
                    connect_packet.client_id.clone(),
                )),
                topic_alias_maximum: None,
                reason_string: None,
                response_information: None,
                user_properties: vec![],
                wildcard_subscription_available: None,
                subscription_identifiers_available: None,
                shared_subscription_available: None,
                server_keep_alive: None,
                server_reference: None,
                authentication_method: None,
                authentication_data: None,
            };

            // add client id to client ds
            self.clients.push(Subs {
                client_id: connect_packet.client_id,
            }); // , active: true

            // send the client the ack
            conn_ack
        }

        // receive subscribe packet
        pub fn accept_sub(&mut self, sub_packet: SubscribePacket) -> SubscribeAckPacket {

            // add to subscription tree
            for topic in &sub_packet.subscription_topics {
                let subscription = Subs{client_id: sub_packet.packet_id.clone().to_string()};

                // store in unsubscriptions list
                self.unsub_list.push(topic.topic_filter.clone());

                // store in subscriptions list
                println!("SubTree count: \t{}",self.subscriptions.insert(&topic.topic_filter, subscription));
                // let match_subs = self.subscriptions.matching_subscribers(topic);
                // println!("Matching Subscribers: \t{:?}" , self.subscriptions.matching_subscribers(topic.parse().unwrap()));
            }

            let sub_ack = SubscribeAckPacket {
                packet_id: sub_packet.packet_id.clone(),
                reason_codes: vec![SubscribeAckReason::GrantedQoSZero],
                reason_string: None,
                user_properties:  Vec::new(),
            };

            sub_ack
        }
        // loop through subscription_topics attribute
        // add to subscription tree of specified type T
        // collect the QoS
        // create subscribe_ack packet
        // add the collected QoS
        // send the client the ack

        // receive publish packet
        // resources : publish_message
        // input : publish packet
        // pub fn accept_publish_packet (&mut self, pub_packet: PublishPacket) -> PublishAckPacket {

        // }

    }
}
