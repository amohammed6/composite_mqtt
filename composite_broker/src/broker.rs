pub mod broker {
    use dashmap::DashMap;
    use rand::prelude::*;
    use std::{collections::HashMap};
    use std::sync::Arc;
    // use mqtt_v5::{topic::TopicFilter, types::{ConnectAckPacket, ConnectPacket, ConnectReason, 
    //     properties::AssignedClientIdentifier, SubscribeAckPacket, SubscribePacket, SubscribeAckReason,
    //     properties::ReasonString, PublishPacket, PublishAckPacket, PublishAckReason, Packet, self
    // }};
    use mqtt_sn::{self, Message, Connect, ConnAck, ReturnCode, RejectedReason, Subscribe, SubAck, 
        Flags, Publish, PubAck, UnsubAck, Unsubscribe}; // TopicName

    
    #[derive(Debug, Clone, PartialEq)]
    #[allow(dead_code)]
    pub struct Client {
        client_id: String,
        address: String,
        port: String
    }

    impl Client {
        fn new (client: String, address: String, port: String) -> Client {
            Client {client_id: client, address: address, port: port}
        }
    }

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub struct Subscriptions {
        subscription_list: Arc<DashMap<u16, Vec<Client>>>
    }

    impl Subscriptions {
        pub fn new () -> Self {
            Subscriptions { 
                subscription_list: Arc::new(DashMap::new() )
            }
        }
        #[allow(dead_code)]
        // return the client address (String) and the publish ack packet
        pub fn accept_pub(&mut self, _addr: String, packet: Publish) -> (Message, Message, Vec<String>) {
            // let addr_port: Vec<&str> = addr.split(":").collect();
            let pub_p = Message::Publish(Publish { 
                ..packet.clone()
            });

            /* 
            // if !self.find_client_by_address(addr_port[0], addr_port[1]) {
            //     // println!("Address not in broker");
            //     let ack = Message::PubAck(PubAck { 
            //         topic_id: packet.topic_id, 
            //         msg_id: 500, 
            //         code: ReturnCode::Rejected(RejectedReason::NotSupported) 
            //     });
            //     return (ack, pub_p, Vec::new())
            // }*/

            // create return variables
            let mut ret_clients : Vec<String> = Vec::new();
            let mut found = false;
            let topic = &packet.topic_id;           
            // let top = self.concrete_subscriptions_list.subscription_list.get(topic);
            let top = self.subscription_list.get(topic);

            // find the topic in the subscriptions list
            match top {
                Some(entry) => {
                    found = true;
                    let list = entry.value();
                    let list = list.to_vec();
                    // println!("\tFound the topic");
                    // add the addresses for the clients that are subscribed to the topic to the return vector
                    for cli in list {
                        ret_clients.push(cli.address.clone()+":"+&cli.port.clone());
                    }
                },
                None => {
                    println!("\tTopic not registered");    
                }
            }

            // make the ack packet
            if found {
                let ack = Message::PubAck(PubAck { 
                    topic_id: packet.topic_id, 
                    msg_id: 200, 
                    code: ReturnCode::Accepted 
                });
                
                return (ack, pub_p, ret_clients);
            }
            else {
                let ack = Message::PubAck(PubAck { 
                    topic_id: packet.topic_id, 
                    msg_id: 500, 
                    code: ReturnCode::Rejected(RejectedReason::InvalidTopicId) 
                }); 
                return (ack, pub_p, ret_clients);
            }
        } // end publish
    }

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub struct MBroker {
        num_packets: u16,
        num_clients: u16,
        num_topics: u16,
        client_list: Vec<Client>,
        concrete_subscriptions_list: Arc<DashMap<u16, Vec<Client>>>, 
        // remove from MBroker struct, create another struct of just the dashmap 
        topicname_id_pairs: HashMap<String, u16>,
    } 

    #[allow(dead_code)]
    impl MBroker {

        // make a new Broker
        pub fn new(list: Arc<DashMap<u16, Vec<Client>>>) -> Self {
            Self { 
                num_packets: 2000,
                num_clients: 0,
                num_topics: 0,
                client_list: Vec::new(), 
                concrete_subscriptions_list: list, //DashMap::new(),
                topicname_id_pairs: HashMap::new()
            }
        } // end new

        // accept a connect packet
        // param: address of the client, connect packet
        // return: connect ack
        pub fn accept_connect(&mut self, addr: String, _connect_packet: Connect) -> Message {
            let addr_port: Vec<&str> = addr.split(":").collect();
            // error checking that client isn't already connected
            if self.find_client_by_address(addr_port[0], addr_port[1]) {
                let packet = Message::ConnAck(ConnAck { 
                    code: ReturnCode::Rejected(RejectedReason::Congestion)
                });
                return packet
            }
            // make a new client 
            self.num_clients += 1;
            let c: Client = Client::new(self.num_clients.to_string(), addr_port[0].to_string(), addr_port[1].to_string());

            // add to client list
            self.client_list.push(c);

            // create connect_ack packet
            let conn_ack = Message::ConnAck(ConnAck { 
                code: ReturnCode::Accepted 
            } );
            self.num_packets +=1;
            conn_ack
        } // end connect


        // accept a subscribe packet -> (Packet, Option<Vec<String>>)
        #[allow(unused_assignments)]
        pub fn accept_sub(&mut self, addr: String, packet: Subscribe, sub_list: Arc<DashMap<u16, Vec<Client>>>) -> Message {
            self.num_packets +=1;
            let addr_port: Vec<&str> = addr.split(":").collect();
            let mut cli: Client = Client::new(String::new(), String::new(), String::new());
            // find the client in reference
            for client in self.client_list.clone() {
                if client.address.eq(&addr_port[0].to_string()) {
                    cli = client.clone();
                } 
            }

            // if not found
            if cli.client_id.trim().is_empty() {
                // println!("\nClient not found");
                let sub_ack = Message::SubAck(SubAck {
                    flags: packet.flags,
                    msg_id: self.num_packets,
                    topic_id: match packet.topic {
                        mqtt_sn::TopicNameOrId::Name(_) => 0,
                        mqtt_sn::TopicNameOrId::Id(id) => id
                    },
                    code: ReturnCode::Rejected(RejectedReason::NotSupported)
                });
                
                return sub_ack;
            }

            // check if topic is in list
            let mut topic = 0;
            match packet.topic {
                mqtt_sn::TopicNameOrId::Id(id) => {
                    // println!("Found id: {}", id);
                    topic = id
                },
                mqtt_sn::TopicNameOrId::Name(name) => {
                    // generate a random number that is not a key in the list already
                    // println!("Topic is : {}", name.to_string());
                    let mut rng = rand::thread_rng();
                    let mut n2: u16 = rng.gen_range(0..999);
                    while sub_list.contains_key(&n2) {
                    // while self.concrete_subscriptions_list.subscription_list.contains_key(&n2) {
                        n2 = rng.gen()
                    }
                    // add it to the pairs list
                    self.topicname_id_pairs.insert(name.clone().to_string(), n2.clone());
                    topic = n2.clone()
                }
            };
            // match self.concrete_subscriptions_list.subscription_list.get_mut(&topic) {
            match sub_list.get_mut(&topic) {
                Some(mut entry) => {
                    entry.push(Client { client_id: cli.client_id.clone(), address: addr_port[0].to_string(), port: addr_port[1].to_string()});
                },
                None => {
                    // self.concrete_subscriptions_list.subscription_list.insert(topic, vec![Client { client_id: cli.client_id.clone(), address: addr_port[0].to_string(), port: addr_port[1].to_string() }]);
                    sub_list.insert(topic, vec![Client { client_id: cli.client_id.clone(), address: addr_port[0].to_string(), port: addr_port[1].to_string() }]);
                }
            }
            Message::SubAck(SubAck { 
                flags: Flags::default(), 
                msg_id: self.num_packets, 
                topic_id: topic,
                code: ReturnCode::Accepted })

        } // end subscribe

        // return the client address (String) and the publish ack packet
        pub fn accept_pub(&mut self, addr: String, packet: Publish, sub_list: Arc<DashMap<u16, Vec<Client>>>) -> (Message, Message, Vec<String>) {
            self.num_packets +=1;
            let addr_port: Vec<&str> = addr.split(":").collect();
            let pub_p = Message::Publish(Publish { 
                ..packet.clone()
            });

            if !self.find_client_by_address(addr_port[0], addr_port[1]) {
                // println!("Address not in broker");
                let ack = Message::PubAck(PubAck { 
                    topic_id: packet.topic_id, 
                    msg_id: self.num_packets, 
                    code: ReturnCode::Rejected(RejectedReason::NotSupported) 
                });
                return (ack, pub_p, Vec::new())
            }

            // create return variables
            let mut ret_clients : Vec<String> = Vec::new();
            let mut found = false;
            let topic = &packet.topic_id;           
            // let top = self.concrete_subscriptions_list.subscription_list.get(topic);
            let top = sub_list.get(topic);

            // find the topic in the subscriptions list
            match top {
                Some(entry) => {
                    found = true;
                    let list = entry.value();
                    // println!("\tFound the topic");
                    // add the addresses for the clients that are subscribed to the topic to the return vector
                    for cli in list {
                        ret_clients.push(cli.address.clone()+":"+&cli.port.clone());
                    }
                },
                None => {
                    println!("\tTopic not registered");    
                }
            }

            // make the ack packet
            if found {
                let ack = Message::PubAck(PubAck { 
                    topic_id: packet.topic_id, 
                    msg_id: self.num_packets, 
                    code: ReturnCode::Accepted 
                });
                
                return (ack, pub_p, ret_clients);
            }
            else {
                let ack = Message::PubAck(PubAck { 
                    topic_id: packet.topic_id, 
                    msg_id: self.num_packets, 
                    code: ReturnCode::Rejected(RejectedReason::InvalidTopicId) 
                }); 
                return (ack, pub_p, ret_clients);
            }
        } // end publish

        pub fn accept_unsub(&mut self, addr: String, packet: Unsubscribe, sub_list: Arc<DashMap<u16, Vec<Client>>>) -> Message {
            self.num_packets +=1;
            let addr_port: Vec<&str> = addr.split(":").collect();
            // find the topic
            let topic =  match &packet.topic {
                mqtt_sn::TopicNameOrId::Id(id) => id,
                mqtt_sn::TopicNameOrId::Name(name) => {
                    // find it in the topic pairs
                    self.topicname_id_pairs.get(&name.as_str().to_string()).unwrap()
                }
            };
            // let top = self.concrete_subscriptions_list.subscription_list.get_mut(topic);
            let top = sub_list.get_mut(topic);

            match top {
                Some(mut list) => {
                    // let list = entry.value_mut();
                    let mut removed: Client = Client { client_id: "0".to_string(), address: "0".to_string(), port: "0".to_string()};
                    // find the client in the topic list
                    let mut i = 0;
                    for client in list.value() {
                        if client.address == addr_port[0] && client.port == addr_port[1] {
                            removed = list.remove(i);
                            break;
                        }
                        i+=1;
                    }
                    // check if you found the client in the topic list
                    if removed.client_id == "0".to_string() {
                        return 
                        Message::UnsubAck(UnsubAck { 
                            msg_id: self.num_packets,
                            code: ReturnCode::Rejected(RejectedReason::InvalidTopicId)
                        })
                    }
                    else {
                        // if it's the last one, remove the topic
                        if list.value().len() == 0 {
                            // println!("Trying to remove");
                            // list.key()
                            // let it = self.concrete_subscriptions_list.subscription_list.remove(topic);
                            // println!("List is 0 , {:?}", it);
                        }
                    }
                },
                None => {
                    return 
                    Message::UnsubAck(UnsubAck { 
                        msg_id: self.num_packets,
                        code: ReturnCode::Rejected(RejectedReason::InvalidTopicId)
                    })
                }
            }

            Message::UnsubAck(UnsubAck { 
                msg_id: self.num_packets,
                code: ReturnCode::Accepted
            })
        }

        /*
            HELPER FUNCTION
        */
        pub fn get_client_list(&mut self) {
            println!("\tPrinting current client list...\n\t{:?}\n", self.client_list)
        }

        pub fn get_sub_list(&mut self) {
            println!("\tPrinting current topic-client list...");
            // for (_ ,k, v) in &self.concrete_subscriptions_list.subscription_list {
            //     println!("\tTopic: {:?} \n\t\tClient List: {:?}", k,v);
            // }
            // ht.keys().iter().for_each(|k| printf("{}", k));
            // for k in self.concrete_subscriptions_list.subscription_list.iter() {
            // for k in self.concrete_subscriptions_list.iter() {
            //     println!("\tTopic: {:?} \n\t\tClient List: {:?}", k.key(),k.value());
            // }   
            println!();
        }

        fn find_client_by_address(&mut self, addr: &str, port: &str) -> bool {
            for cl in &self.client_list {
                if cl.address == addr.to_string() && cl.port == port.to_string()
                {
                    return true;
                }
            }
            false
        }

        // fn does_topic_exist(&mut self, topic: u16) -> bool {
        //     for (k, _v) in &self.concrete_subscriptions_list {
        //         if k.eq(&topic) { 
        //             println!("found");
        //             return true;
        //         }
        //     }
        //     println!("nope");
        //     false
        // }
    }
}
