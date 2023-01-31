pub mod broker {
    use std::collections::HashMap;
    use mqtt_v5::{topic::TopicFilter, types::{ConnectAckPacket, ConnectPacket, ConnectReason, 
        properties::AssignedClientIdentifier, SubscribeAckPacket, SubscribePacket, SubscribeAckReason,
        properties::ReasonString, PublishPacket, PublishAckPacket, PublishAckReason
    }};

    
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub struct Client {
        client_id: String,
        address: String,
    }

    impl Client {
        fn new (client: String, address: String) -> Client {
            Client {client_id: client, address: address}
        }
    }

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub struct MBroker {
        num_packets: u16,
        client_list: Vec<Client>,
        concrete_subscriptions_list: HashMap<String, Vec<Client>>,
        store_outgoing_publish: Vec<PublishPacket>,
    }

    #[allow(dead_code)]
    impl MBroker {

        // make a new Broker
        pub fn new() -> Self {
            Self { 
                num_packets: 2000,
                client_list: Vec::new(), 
                concrete_subscriptions_list: HashMap::new(),
                store_outgoing_publish: Vec::new(),
            }
        } // end new

        // accept a connect packet
        // param: address of the client, connect packet
        // return: connect ack
        pub fn accept_connect(&mut self, addr: &str, connect_packet: ConnectPacket) -> ConnectAckPacket {
            // make a new client 
            let c: Client = Client::new(connect_packet.client_id.clone(), addr.to_string());

            // add to client list
            self.client_list.push(c);

            // create connect_ack packet
            let conn_ack = ConnectAckPacket {
                session_present: true,
                assigned_client_identifier: Some(AssignedClientIdentifier(
                    connect_packet.client_id.clone(),
                )),
                reason_code: ConnectReason::Success,
                user_properties: vec![],
                session_expiry_interval:None, receive_maximum:None, maximum_qos:None, retain_available:None, 
                maximum_packet_size:None, topic_alias_maximum:None, reason_string:None, response_information:None,
                wildcard_subscription_available:None, subscription_identifiers_available:None,
                shared_subscription_available:None, server_keep_alive:None, server_reference:None,
                authentication_data:None, authentication_method:None,
            };
            self.num_packets +=1;
            conn_ack
        } // end connect

        // accept a subscribe packet
        pub fn accept_sub(&mut self, addr: &str, packet: SubscribePacket) -> SubscribeAckPacket {
            self.num_packets +=1;
            let mut cli: Client = Client::new(String::new(), String::new());
            // find the client in reference
            for client in self.client_list.clone() {
                if client.address.eq(&addr.to_string()) {
                    cli = client.clone();
                } 
            }

            // if not found
            if cli.client_id.trim().is_empty() {
                println!("Client not found");
                let sub_ack = SubscribeAckPacket {
                    packet_id:packet.packet_id.clone(),
                    reason_codes: vec![SubscribeAckReason::NotAuthorized],
                    reason_string: Some(ReasonString("Client is not recognized by Broker".to_string())),
                    user_properties: Vec::new()
                };
                return sub_ack;
            }

            // get each topic from the packet
            for topic in &packet.subscription_topics {
                // check if the topic exists
                match &topic.topic_filter {
                    TopicFilter::Concrete { filter, level_count:_ } =>
                    {
                        // println!("Concrete entered"); 
                        // add to the subscription list
                        match self.concrete_subscriptions_list.get_mut(filter) {
                            Some(entry) => {
                                entry.push(Client { client_id: cli.client_id.clone(), address: addr.to_string()})
                            },
                            None => {
                                self.concrete_subscriptions_list.insert(filter.to_string(), vec![Client { client_id: cli.client_id.clone(), address: addr.to_string() }]);
                            }
                        }
                    }
                    _=> {   // other filter types aren't accepted
                        println!("Other filter entered");
                        return SubscribeAckPacket { 
                            packet_id:packet.packet_id.clone(),
                            reason_codes: vec![SubscribeAckReason::NotAuthorized],
                            reason_string: Some(ReasonString("TopicFilter is not recognized by Broker".to_string())),
                            user_properties: Vec::new() };
                    }
                };
            }
            println!("Success");
            SubscribeAckPacket { 
                packet_id: packet.packet_id.clone() + 1, 
                reason_string: None, 
                user_properties: Vec::new(), 
                reason_codes: vec![SubscribeAckReason::GrantedQoSZero]
            }
        } // end subscribe

        // return the client address (String) and the publish ack packet
        pub fn accept_pub(&mut self, addr: &str, packet: PublishPacket) -> (PublishAckPacket, Vec<String>) {
            self.num_packets +=1;
            // create return variables
            let mut ret_clients : Vec<String> = Vec::new();
            ret_clients.push(addr.to_string());         // the ack will be sent to the client address, pushed first

            // look for the clients that are subscribed to the topic
            let topic = packet.topic.topic_name();
            let list = self.concrete_subscriptions_list.get_key_value(topic);
            // find the topic in the subscriptions list
            if list.is_some() {
                let clients = list.unwrap().1;
                // add the addresses for the clients that are subscribed to the topic to the return vector
                for cli in clients {
                    ret_clients.push(cli.address.clone());
                }
            }
            else { // add to the outgoing storage
                self.store_outgoing_publish.push(packet);
            }

            // make the ack packet
            let ack = PublishAckPacket { 
                packet_id: self.num_packets +1, 
                reason_code: PublishAckReason::Success, 
                reason_string: None, 
                user_properties: vec![] 
            };
            
            (ack, ret_clients)
        } // end publish

        pub fn get_client_list(&mut self) {
            println!("Printing current client list...\n{:?}\n", self.client_list)
        }

        pub fn get_sub_list(&mut self) {
            println!("Printing current topic-client list...");
            for (k, v) in &self.concrete_subscriptions_list {
                println!("topic: {:?} \tclient list: {:?}", k,v);
            }
        }
    }
}
