pub mod broker {
    // broker function
    use mqtt_v5::types::{ConnectAckPacket, ConnectReason, ConnectPacket, properties::{AssignedClientIdentifier}};  
    
    // global ds
        // 1-level subscriptions
        // specified type T (for subscriptions)
            // client_id
    pub struct Subs {
        client_id: String,
        // active: bool,
    }
        // clients (may be handled by session?)
    pub struct MBroker {
        // clients: Vec<String>,
        subscriptions: Vec<Subs>,
    }
    impl MBroker {
        pub fn new() -> Self {
            Self { subscriptions: Vec::new()} //clients: Vec::new()
        }

        // receive connect packet
        // resources : take_over_existing_client, handle_new_client
        // input: connect packet
        pub fn accept_new_client (&mut self, connect_packet: ConnectPacket) -> ConnectAckPacket{
            // println!("connect packet received: {}", connect_packet.client_id);

            // create connect_ack packet
            let conn_ack = ConnectAckPacket {
                session_present: true,
                reason_code: ConnectReason::Success,
                session_expiry_interval: None,
                receive_maximum: None,
                maximum_qos: None,
                retain_available: None,
                maximum_packet_size: None,
                assigned_client_identifier: Some(AssignedClientIdentifier(connect_packet.client_id.clone())), 
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
            self.subscriptions.push(Subs{client_id: connect_packet.client_id}); // , active: true
            for x in &self.subscriptions {
                println!("\t{}", x.client_id);
            }
            // send the client the ack
            conn_ack
        }
    //
        // receive publish packet
        // resources : publish_message
        // input : publish packet
            // 

        // receive subscribe packet
        // resources : handle_subscribe
        // input : 
        // output : status code
            // loop through subscription_topics attribute
                // add to subscription tree of specified type T
                // collect the QoS
            // create subscribe_ack packet
                // add the collected QoS
            // send the client the ack
    }

    
}

