pub mod broker {
    use rand::prelude::*;
    use std::{collections::HashMap};
    // use mqtt_v5::{topic::TopicFilter, types::{ConnectAckPacket, ConnectPacket, ConnectReason, 
    //     properties::AssignedClientIdentifier, SubscribeAckPacket, SubscribePacket, SubscribeAckReason,
    //     properties::ReasonString, PublishPacket, PublishAckPacket, PublishAckReason, Packet, self
    // }};
    use mqtt_sn::{self, Message, Connect, ConnAck, ReturnCode, RejectedReason, Subscribe, SubAck, 
        Flags, Publish, PubAck}; // TopicName

    
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
    pub struct MBroker {
        num_packets: u16,
        num_clients: u16,
        num_topics: u16,
        client_list: Vec<Client>,
        concrete_subscriptions_list: HashMap<u16, Vec<Client>>,
        topic_name_arr: Vec<String>,
    } 

    #[allow(dead_code)]
    impl MBroker {

        // make a new Broker
        pub fn new() -> Self {
            Self { 
                num_packets: 2000,
                num_clients: 0,
                num_topics: 0,
                client_list: Vec::new(), 
                concrete_subscriptions_list: HashMap::new(),
                topic_name_arr: Vec::new()
            }
        } // end new

        // accept a connect packet
        // param: address of the client, connect packet
        // return: connect ack
        pub fn accept_connect(&mut self, addr: String, connect_packet: Connect) -> Message {
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
        pub fn accept_sub(&mut self, addr: String, packet: Subscribe) -> Message {
            let addr_port: Vec<&str> = addr.split(":").collect();
            // let mut stored_packet = false;
            // let mut outgoing_topics: Vec<String> = Vec::new();
            let mut cli: Client = Client::new(String::new(), String::new(), String::new());
            // find the client in reference
            for client in self.client_list.clone() {
                if client.address.eq(&addr_port[0].to_string()) {
                    cli = client.clone();
                } 
            }

            // if not found
            if cli.client_id.trim().is_empty() {
                println!("\nClient not found");
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
            self.num_packets +=1;

            // check if topic is in list
            let mut topic = 0;
            match packet.topic {
                mqtt_sn::TopicNameOrId::Id(id) => {
                    topic = id
                },
                mqtt_sn::TopicNameOrId::Name(_) => {
                    // generate a random number that is not a key in the list already
                    let mut rng = rand::thread_rng();
                    let mut n2: u16 = 0;
                    while self.concrete_subscriptions_list.contains_key(&n2) {
                        n2 = rng.gen()
                    }
                    topic = n2.clone()
                }
            };
            
            match self.concrete_subscriptions_list.get_mut(&topic) {
                Some(entry) => {
                    entry.push(Client { client_id: cli.client_id.clone(), address: addr_port[0].to_string(), port: addr_port[1].to_string()});
                },
                None => {
                    self.concrete_subscriptions_list.insert(topic, vec![Client { client_id: cli.client_id.clone(), address: addr_port[0].to_string(), port: addr_port[1].to_string() }]);
                }
            }
            /* 
            // for topic in &packet.subscription_topics {
            //     // check if the topic exists
            //     match &topic.topic_filter {
            //         TopicFilter::Concrete { filter, level_count:_ } =>
            //         {
            //             // add to the subscription list
            //             match self.concrete_subscriptions_list.get_mut(filter) {
            //                 Some(entry) => {
            //                     // if the topic is already in the list, add the client
            //                     entry.push(Client { client_id: cli.client_id.clone(), address: addr_port[0].to_string(), port: addr_port[1].to_string()});
            //                 },
            //                 None => {
            //                     self.concrete_subscriptions_list.insert(filter.to_string(), vec![Client { client_id: cli.client_id.clone(), address: addr_port[0].to_string(), port: addr_port[1].to_string() }]);
            //                 }
            //             }
            //         }
            //         _=> {   // other filter types aren't accepted
            //             println!("\nOther filter entered");
            //             return Packet::SubscribeAck(SubscribeAckPacket{ 
            //                 packet_id:packet.packet_id.clone(),
            //                 reason_codes: vec![SubscribeAckReason::NotAuthorized],
            //                 reason_string: Some(ReasonString("TopicFilter is not recognized by Broker".to_string())),
            //                 user_properties: Vec::new() 
            //             });
            //         }
            //     };
            // }
            */
            Message::SubAck(SubAck { 
                flags: Flags::default(), 
                msg_id: self.num_packets, 
                topic_id: topic,
                code: ReturnCode::Accepted })

        } // end subscribe

        // return the client address (String) and the publish ack packet
        pub fn accept_pub(&mut self, addr: String, packet: Publish) -> (Message, Message, Vec<String>) {
            let addr_port: Vec<&str> = addr.split(":").collect();
            let pub_p = Message::Publish(Publish { 
                ..packet.clone()
            });

            if !self.find_client_by_address(addr_port[0], addr_port[1]) {
                println!("Address not in broker");
                let ack = Message::PubAck(PubAck { 
                    topic_id: packet.topic_id, 
                    msg_id: self.num_packets +1, 
                    code: ReturnCode::Rejected(RejectedReason::NotSupported) 
                });
                return (ack, pub_p, Vec::new())
            }
            self.num_packets +=1;

            // create return variables
            let mut ret_clients : Vec<String> = Vec::new();
            let mut found = false;
            let topic = &packet.topic_id;           
            let top = self.concrete_subscriptions_list.get(topic);

            // find the topic in the subscriptions list
            match top {
                Some(list) => {
                    found = true;
                    println!("\tFound the topic");
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
                    msg_id: self.num_packets +1, 
                    code: ReturnCode::Accepted 
                });
                
                return (ack, pub_p, ret_clients);
            }
            else {
                let ack = Message::PubAck(PubAck { 
                    topic_id: packet.topic_id, 
                    msg_id: self.num_packets +1, 
                    code: ReturnCode::Rejected(RejectedReason::InvalidTopicId) 
                }); 
                return (ack, pub_p, ret_clients);
            }
        } // end publish

        /*
            HELPER FUNCTION
        */
        pub fn get_client_list(&mut self) {
            println!("\tPrinting current client list...\n\t{:?}\n", self.client_list)
        }

        pub fn get_sub_list(&mut self) {
            println!("\tPrinting current topic-client list...");
            for (k, v) in &self.concrete_subscriptions_list {
                println!("\tTopic: {:?} \n\t\tClient List: {:?}", k,v);
            }
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

        fn does_topic_exist(&mut self, topic: u16) -> bool {
            for (k, _v) in &self.concrete_subscriptions_list {
                if k.eq(&topic) { 
                    println!("found");
                    return true;
                }
            }
            println!("nope");
            false
        }
    }
}
