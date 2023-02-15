pub mod broker {
    use std::{collections::HashMap};
    use mqtt_v5::{topic::TopicFilter, types::{ConnectAckPacket, ConnectPacket, ConnectReason, 
        properties::AssignedClientIdentifier, SubscribeAckPacket, SubscribePacket, SubscribeAckReason,
        properties::ReasonString, PublishPacket, PublishAckPacket, PublishAckReason, Packet, self
    }};
    

    
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
        client_list: Vec<Client>,
        concrete_subscriptions_list: HashMap<String, Vec<Client>>,
        store_outgoing_publish: HashMap<String, Vec<PublishPacket>>,
    } 

    #[allow(dead_code)]
    impl MBroker {

        // make a new Broker
        pub fn new() -> Self {
            Self { 
                num_packets: 2000,
                num_clients: 0,
                client_list: Vec::new(), 
                concrete_subscriptions_list: HashMap::new(),
                store_outgoing_publish: HashMap::new(),
            }
        } // end new

        // accept a connect packet
        // param: address of the client, connect packet
        // return: connect ack
        pub fn accept_connect(&mut self, addr: String, connect_packet: ConnectPacket) -> Packet {
            let addr_port: Vec<&str> = addr.split(":").collect();
            // error checking that client isn't already connected
            if self.find_client_by_address(addr_port[0], addr_port[1]) {
                let packet = Packet::ConnectAck(ConnectAckPacket {
                    session_present: true,
                    assigned_client_identifier: Some(AssignedClientIdentifier(
                        connect_packet.client_id.clone(),
                    )),
                    reason_code: ConnectReason::ClientIdentifierNotValid,
                    user_properties: vec![],
                    reason_string:Some(types::properties::ReasonString("Client already registered".to_string())),
                    session_expiry_interval:None, receive_maximum:None, maximum_qos:None, retain_available:None, 
                    maximum_packet_size:None, topic_alias_maximum:None, response_information:None,
                    wildcard_subscription_available:None, subscription_identifiers_available:None,
                    shared_subscription_available:None, server_keep_alive:None, server_reference:None,
                    authentication_data:None, authentication_method:None,
                });
                return packet
            }
            // make a new client 
            self.num_clients += 1;
            let c: Client = Client::new(self.num_clients.to_string(), addr_port[0].to_string(), addr_port[1].to_string());

            // add to client list
            self.client_list.push(c);

            // create connect_ack packet
            let conn_ack = Packet::ConnectAck(ConnectAckPacket {
                session_present: true,
                assigned_client_identifier: Some(AssignedClientIdentifier(
                    connect_packet.client_id.clone(),
                )),
                reason_code: ConnectReason::Success,
                user_properties: vec![],
                reason_string:Some(types::properties::ReasonString("Success".to_string())),
                session_expiry_interval:None, receive_maximum:None, maximum_qos:None, retain_available:None, 
                maximum_packet_size:None, topic_alias_maximum:None, response_information:None,
                wildcard_subscription_available:None, subscription_identifiers_available:None,
                shared_subscription_available:None, server_keep_alive:None, server_reference:None,
                authentication_data:None, authentication_method:None,
            });
            self.num_packets +=1;
            conn_ack
        } // end connect


        // accept a subscribe packet -> (Packet, Option<Vec<String>>)
        pub fn accept_sub(&mut self, addr: String, packet: SubscribePacket) -> Packet {
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
                let sub_ack = Packet::SubscribeAck(SubscribeAckPacket{ 
                    packet_id:packet.packet_id.clone(),
                    reason_codes: vec![SubscribeAckReason::NotAuthorized],
                    reason_string: Some(ReasonString("Client is not recognized by Broker".to_string())),
                    user_properties: Vec::new()
                });
                return sub_ack;
            }
            self.num_packets +=1;
            // get each topic from the packet
            for topic in &packet.subscription_topics {
                // check if the topic exists
                match &topic.topic_filter {
                    TopicFilter::Concrete { filter, level_count:_ } =>
                    {
                        // add to the subscription list
                        match self.concrete_subscriptions_list.get_mut(filter) {
                            Some(entry) => {
                                // if the topic is already in the list, add the client
                                entry.push(Client { client_id: cli.client_id.clone(), address: addr_port[0].to_string(), port: addr_port[1].to_string()});
                                // check if packets are waiting to be published to this topic
                                /* 
                                if self.store_outgoing_publish.contains_key(&filter.to_string()) {
                                    // stored_packet = true;
                                    outgoing_topics.push(filter.to_string());
                                }
                                */
                            },
                            None => {
                                self.concrete_subscriptions_list.insert(filter.to_string(), vec![Client { client_id: cli.client_id.clone(), address: addr_port[0].to_string(), port: addr_port[1].to_string() }]);
                            }
                        }
                    }
                    _=> {   // other filter types aren't accepted
                        println!("\nOther filter entered");
                        return Packet::SubscribeAck(SubscribeAckPacket{ 
                            packet_id:packet.packet_id.clone(),
                            reason_codes: vec![SubscribeAckReason::NotAuthorized],
                            reason_string: Some(ReasonString("TopicFilter is not recognized by Broker".to_string())),
                            user_properties: Vec::new() 
                        });
                    }
                };
            }
            Packet::SubscribeAck(SubscribeAckPacket{
                packet_id: packet.packet_id.clone() + 1, 
                reason_string: Some(ReasonString("Successful subscription".to_string())), 
                user_properties: Vec::new(), 
                reason_codes: vec![SubscribeAckReason::GrantedQoSZero]
            }) // Some(outgoing_topics)
        } // end subscribe

        // return the client address (String) and the publish ack packet
        pub fn accept_pub(&mut self, addr: String, packet: PublishPacket) -> (Packet, Packet, Vec<String>) {
            let addr_port: Vec<&str> = addr.split(":").collect();
            let pub_p = Packet::Publish(PublishPacket { 
                ..packet.clone()
            });
            if !self.find_client_by_address(addr_port[0], addr_port[1]) {
                println!("Address not in broker");
                let ack = Packet::PublishAck(PublishAckPacket { 
                    packet_id: self.num_packets +1, 
                    reason_code: PublishAckReason::NotAuthorized, 
                    reason_string: Some(ReasonString("Address not in broker".to_string())), 
                    user_properties: vec![] 
                });
                return (ack, pub_p, Vec::new())
            }
            self.num_packets +=1;
            // create return variables
            let mut ret_clients : Vec<String> = Vec::new();

            let topic = &packet.topic.topic_name().to_string();           
            let top = self.concrete_subscriptions_list.get(topic);

            // find the topic in the subscriptions list
            match top {
                Some(list) => {
                    println!("\tFound the topic");
                    // add the addresses for the clients that are subscribed to the topic to the return vector
                    for cli in list {
                        ret_clients.push(cli.address.clone()+":"+&cli.port.clone());
                    }
                },
                None => {
                    let mut vec_packets : Vec<PublishPacket> = Vec::new();
                    vec_packets.push(packet);
                    self.store_outgoing_publish.insert(topic.to_string(),vec_packets);
                    println!("\tStored for future subscribers");    
                }
            }

            // make the ack packet
            let ack = Packet::PublishAck(PublishAckPacket { 
                packet_id: self.num_packets +1, 
                reason_code: PublishAckReason::Success, 
                reason_string: Some(ReasonString("Successful publishing".to_string())), 
                user_properties: vec![] 
            });
            
            (ack, pub_p, ret_clients)
        } // end publish

        pub fn get_outgoing_packets(&mut self, topic: &String) -> Option<&Vec<PublishPacket>>{
            let ret = self.store_outgoing_publish.get(topic);

            match ret {
                Some(list) => {
                    return Some(list);
                }
                _ => {
                    println!("None available");
                    return None
                }
            }
        }

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

        pub fn get_outgoing_list(&mut self) {
            println!("\tPrinting current outgoing list...");
            for (k, _v) in &self.store_outgoing_publish {
                println!("Packet topic: {:?}", k)
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

        fn does_topic_exist(&mut self, topic: String) -> bool {
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
