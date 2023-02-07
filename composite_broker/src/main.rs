mod broker;
mod msg_parser;
use std::io;
// use bytes::{BytesMut};
use std::time;
use crate::broker::broker::MBroker;
// use crate::msg_parser::msg_parser::cm_encode;
// use bytes::BytesMut;
use mqtt_v5::types::Packet;
use std ::net::{TcpListener,TcpStream};
use std::io::{Read,Write};
use std::thread;
use crate::msg_parser::msg_parser::{cm_decode};
// use mqtt_v5::types::{Packet::{}}; // ConnectPacket. decoder, ProtocolVersion

// static mut BROKER: MBroker = MBroker::new();

// Handle access stream; create a struct to hold the stream’s state
// Perform I/O operations
fn handle_sender(mut stream: TcpStream, addr: &str, mut broker: MBroker) -> io::Result<()>{

    // Handle multiple access stream
    let mut buf = [0;512];
    for _ in 0..1000{
        let bytes_read = stream.read(&mut buf)?;  // let the receiver get a message from a sender
        
        if bytes_read == 0 {
            // sender stream in a mutable variable
            return Ok(());
        }

        stream.write(&buf[..bytes_read])?;

        // Read and determine path for message
        if String::from_utf8_lossy(&buf).starts_with("mqtt") {// contains("connect") {
            // if "connect" message received, alert client to send connect packet
            println!("Incoming command from client");
        }
        else {
            // decoding packet 
            let decode = cm_decode(&mut buf);

            match decode {
                Ok(Packet::Connect(p)) => {
                    broker.accept_connect(addr, p);         // connect to the broker
                    broker.get_client_list();                               // show the client list
                },
                Ok(Packet::Publish(p)) => {
                    broker.accept_pub(addr, p);
                    broker.get_outgoing_list();
                },
                Ok(Packet::Subscribe(p)) => {
                    broker.accept_sub(addr, p);
                    broker.get_sub_list();
                },
                
                _ => panic!("Incorrect type returned"),
            };
        }
        
        // And you can sleep this connection with the connected sender
        thread::sleep(time::Duration::from_secs(1));  
    }
    // success value
    Ok(())
}


fn main() -> io::Result<()>{
    let broker = MBroker::new();
    // Enable port 7878 binding
    let receiver_listener = TcpListener::bind("127.0.0.1:7878").expect("Failed and bind with the sender");
    // Getting a handle of the underlying thread.
    // let mut thread_vec: Vec<thread::JoinHandle<()>> = Vec::new();
    // listen to incoming connections messages and bind them to a sever socket address.
    for stream in receiver_listener.incoming() {
        let stream = stream.expect("failed");
        // let the receiver connect with the sender
        // let handle = thread::spawn(move || {
            //receiver failed to read from the stream
            handle_sender(stream, "127.0.0.1:7878", broker.clone()).unwrap_or_else(|error| eprintln!("{:?}",error))
        // });
        
        // Push messages in the order they are sent
        // thread_vec.push(handle);
    }

    // for handle in thread_vec {
    //     // return each single value Output contained in the heap
    //     handle.join().unwrap();
    // }
    // success value
    Ok(())
}


#[cfg(test)]

mod tests {
    use crate::broker::broker::MBroker;
    use mqtt_v5::{types::{Packet, PublishPacket, SubscribePacket, ConnectPacket, ConnectReason, QoS,
        ProtocolVersion, RetainHandling, SubscribeAckReason, PublishAckReason, SubscriptionTopic}, 
        topic::{TopicFilter, Topic}};
    use bytes::{Bytes, BytesMut};
    use crate::msg_parser::msg_parser::{cm_decode, cm_encode};
    
    static ADDR: &str = "127.0.0.1:7878"; 
    static ADDR2: &str = "192.0.2.1";

    #[test]
    fn test_read_publish_packet() {
        // Create a Publish packet
        let packet2 = Packet::Publish(PublishPacket {
            is_duplicate: false,
            qos: QoS::AtLeastOnce,
            retain: true,
            topic: "gwu".parse().unwrap(),
            user_properties: Vec::new(),
            payload: Bytes::from("this is gwu"), // immutable to preserve security,
            packet_id: Some(42),                 // required
            payload_format_indicator: None,
            message_expiry_interval: None,
            topic_alias: None,
            response_topic: None,
            correlation_data: None,
            subscription_identifier: None,
            content_type: None,
        });

        // create the buffer for encoding
        let mut buf2 = BytesMut::new();
        let res2 = cm_encode(packet2, &mut buf2);

        assert!(res2.is_ok());

        // decode publish packet
        let decode2 = cm_decode(&mut buf2);

        match decode2 {
            Ok(Packet::Publish(p)) => println!("\tPublish packet received {:?}", p.packet_id),
            _ => panic!("Incorrect type returned"),
        };
    }

    #[test]
    fn test_read_connect_packet() {
        // Create a Connect packet
        let packet = Packet::Connect(ConnectPacket {
            protocol_name: String::from("cm_mqtt"),
            protocol_version: ProtocolVersion::V500,
            clean_start: true,
            keep_alive: 1,
            user_properties: Vec::new(),
            client_id: String::from("1004"),
            session_expiry_interval: None,
            receive_maximum: None,
            maximum_packet_size: None,
            topic_alias_maximum: None,
            request_response_information: None,
            request_problem_information: None,
            authentication_method: None,
            authentication_data: None,
            will: None,
            user_name: None,
            password: None,
        });
        // let packet = types::Packet::Connect(conn_p);

        // create buffer for encoding
        let mut buf = BytesMut::new();
        let res = cm_encode(packet, &mut buf);
        assert!(res.is_ok());

        // decode the connect packet
        let decode = cm_decode(&mut buf);

        assert!(decode.is_ok());
        match decode {
            Ok(Packet::Connect(p)) => println!("\tConnect packet received {}", p.client_id),
            _ => panic!("Incorrect type returned"),
        };
    }
    
    #[test]
    fn test_new_client1() {
        let mut broker = MBroker::new();
        let id = "1004".to_string();
        // Create a Connect packet
        let conn_p = ConnectPacket {
            protocol_name: String::from("cm_mqtt"),
            protocol_version: ProtocolVersion::V500,
            clean_start: true,
            keep_alive: 1,
            user_properties: Vec::new(),
            client_id: id,
            session_expiry_interval: None,
            receive_maximum: None,
            maximum_packet_size: None,
            topic_alias_maximum: None,
            request_response_information: None,
            request_problem_information: None,
            authentication_method: None,
            authentication_data: None,
            will: None,
            user_name: None,
            password: None,
        };
    
        let res = broker.accept_connect(ADDR, conn_p);
        broker.get_client_list();
        match res {
            Packet::ConnectAck(p) => {
                assert_eq!(p.reason_code, ConnectReason::Success);
            },
            _=> {}
        }
    }

    
    #[test]
    fn test_new_client2() {
        let mut broker = MBroker::new();
        // Create two Connect packet
        let conn_p1 = ConnectPacket {
            protocol_name: String::from("cm_mqtt"),
            protocol_version: ProtocolVersion::V500,
            clean_start: true,
            keep_alive: 1,
            user_properties: Vec::new(),
            client_id: "1004".to_string(),
            session_expiry_interval: None,
            receive_maximum: None,
            maximum_packet_size: None,
            topic_alias_maximum: None,
            request_response_information: None,
            request_problem_information: None,
            authentication_method: None,
            authentication_data: None,
            will: None,
            user_name: None,
            password: None,
        };

        let conn_p2 = ConnectPacket {
            protocol_name: String::from("cm_mqtt"),
            protocol_version: ProtocolVersion::V500,
            clean_start: true,
            keep_alive: 1,
            user_properties: Vec::new(),
            client_id: "1005".to_string(),
            session_expiry_interval: None,
            receive_maximum: None,
            maximum_packet_size: None,
            topic_alias_maximum: None,
            request_response_information: None,
            request_problem_information: None,
            authentication_method: None,
            authentication_data: None,
            will: None,
            user_name: None,
            password: None,
        };

        let r1 = broker.accept_connect(ADDR, conn_p1);
        let r2 = broker.accept_connect(ADDR, conn_p2);
        broker.get_client_list();
        match r1 {
            Packet::ConnectAck(p) => {
                assert_eq!(p.reason_code, ConnectReason::Success);
            },
            _=> {}
        }

        match r2 {
            Packet::ConnectAck(p) => {
                assert_eq!(p.reason_code, ConnectReason::Success);
            },
            _=> {}
        }
    }

    
    #[test]
    fn test_new_sub() {
        let mut broker = MBroker::new();
        let id = "1004".to_string();
        // Create a Connect packet
        let conn_p = ConnectPacket {
            protocol_name: String::from("cm_mqtt"),
            protocol_version: ProtocolVersion::V500,
            clean_start: true,
            keep_alive: 1,
            user_properties: Vec::new(),
            client_id: id,
            session_expiry_interval: None,
            receive_maximum: None,
            maximum_packet_size: None,
            topic_alias_maximum: None,
            request_response_information: None,
            request_problem_information: None,
            authentication_method: None,
            authentication_data: None,
            will: None,
            user_name: None,
            password: None,
        };
    
        broker.accept_connect(ADDR, conn_p);
        
        // create a subscribe packet
        let sub_p = SubscribePacket {
            packet_id: 01,
            subscription_identifier: None,
            user_properties: Vec::new(),
            subscription_topics: vec![SubscriptionTopic {
                topic_filter: TopicFilter::Concrete { filter: "gwu".to_string(), level_count: 1 },
                maximum_qos: QoS::AtLeastOnce,
                no_local: false,
                retain_as_published: false,
                retain_handling: RetainHandling::SendAtSubscribeTime,
            }],
        };

        let res = broker.accept_sub(ADDR, sub_p);
        broker.get_sub_list();
        assert_eq!(res.reason_codes.contains(&Some(SubscribeAckReason::GrantedQoSZero).unwrap()), true);
    }

    #[test]
    fn test_multiple_subs_gwu() {
        let mut broker = MBroker::new();
        // connect
        let id = "1004".to_string();
        // Create a Connect packet
        let conn_p = ConnectPacket {
            protocol_name: String::from("cm_mqtt"),
            protocol_version: ProtocolVersion::V500,
            clean_start: true,
            keep_alive: 1,
            user_properties: Vec::new(),
            client_id: id,
            session_expiry_interval: None,
            receive_maximum: None,
            maximum_packet_size: None,
            topic_alias_maximum: None,
            request_response_information: None,
            request_problem_information: None,
            authentication_method: None,
            authentication_data: None,
            will: None,
            user_name: None,
            password: None,
        };
    
        broker.accept_connect(ADDR, conn_p);

        // create subscribe packets
        let sub_p1 = SubscribePacket {
            packet_id: 01,
            subscription_identifier: None,
            user_properties: Vec::new(),
            subscription_topics: vec![SubscriptionTopic {
                topic_filter: TopicFilter::Concrete { filter: "seas".to_string(), level_count: 1 },
                maximum_qos: QoS::AtLeastOnce,
                no_local: false,
                retain_as_published: false,
                retain_handling: RetainHandling::SendAtSubscribeTime,
            }],
        };

        let sub_p2 = SubscribePacket {
            packet_id: 02,
            subscription_identifier: None,
            user_properties: Vec::new(),
            subscription_topics: vec![SubscriptionTopic {
                topic_filter: TopicFilter::Concrete { filter: "ccas".to_string(), level_count: 1 },
                maximum_qos: QoS::AtLeastOnce,
                no_local: false,
                retain_as_published: false,
                retain_handling: RetainHandling::SendAtSubscribeTime,
            }],
        };

        let sub_p3 = SubscribePacket {
            packet_id: 03,
            subscription_identifier: None,
            user_properties: Vec::new(),
            subscription_topics: vec![SubscriptionTopic {
                topic_filter: TopicFilter::Concrete { filter: "elliot".to_string(), level_count: 1 },
                maximum_qos: QoS::AtLeastOnce,
                no_local: false,
                retain_as_published: false,
                retain_handling: RetainHandling::SendAtSubscribeTime,
            }],
        };

        let r1 = broker.accept_sub(ADDR, sub_p1);
        assert_eq!(r1.reason_codes.contains(&Some(SubscribeAckReason::GrantedQoSZero).unwrap()), true);

        let r2 = broker.accept_sub(ADDR, sub_p2);
        assert_eq!(r2.reason_codes.contains(&Some(SubscribeAckReason::GrantedQoSZero).unwrap()), true);

        let r3 = broker.accept_sub(ADDR, sub_p3);
        assert_eq!(r3.reason_codes.contains(&Some(SubscribeAckReason::GrantedQoSZero).unwrap()), true);
        broker.get_sub_list(); 
    }
  
    #[test]
    fn test_multiple_subs_unis() {
        let mut broker = MBroker::new();
        // Create a Connect packet
        let conn_p = ConnectPacket {
            protocol_name: String::from("cm_mqtt"),
            protocol_version: ProtocolVersion::V500,
            clean_start: true,
            keep_alive: 1,
            user_properties: Vec::new(),
            client_id: "1004".to_string() ,
            session_expiry_interval: None,
            receive_maximum: None,
            maximum_packet_size: None,
            topic_alias_maximum: None,
            request_response_information: None,
            request_problem_information: None,
            authentication_method: None,
            authentication_data: None,
            will: None,
            user_name: None,
            password: None,
        };
    
        broker.accept_connect(ADDR, conn_p);
        // create subscribe packets
        let sub_p1 = SubscribePacket {
            packet_id: 01,
            subscription_identifier: None,
            user_properties: Vec::new(),
            subscription_topics: vec![SubscriptionTopic {
                topic_filter: TopicFilter::Concrete { filter: "gwu".to_string(), level_count: 1 },
                maximum_qos: QoS::AtLeastOnce,
                no_local: false,
                retain_as_published: false,
                retain_handling: RetainHandling::SendAtSubscribeTime,
            }],
        };

        let sub_p2 = SubscribePacket {
            packet_id: 02,
            subscription_identifier: None,
            user_properties: Vec::new(),
            subscription_topics: vec![SubscriptionTopic {
                topic_filter: TopicFilter::Concrete { filter: "udel".to_string(), level_count: 1 },
                maximum_qos: QoS::AtLeastOnce,
                no_local: false,
                retain_as_published: false,
                retain_handling: RetainHandling::SendAtSubscribeTime,
            }],
        };

        let sub_p3 = SubscribePacket {
            packet_id: 03,
            subscription_identifier: None,
            user_properties: Vec::new(),
            subscription_topics: vec![SubscriptionTopic {
                topic_filter: TopicFilter::Concrete { filter: "uwm".to_string(), level_count: 1 },
                maximum_qos: QoS::AtLeastOnce,
                no_local: false,
                retain_as_published: false,
                retain_handling: RetainHandling::SendAtSubscribeTime,
            }],
        };

        let r1 = broker.accept_sub(ADDR, sub_p1);
        assert_eq!(r1.reason_codes.contains(&Some(SubscribeAckReason::GrantedQoSZero).unwrap()), true);

        let r2 = broker.accept_sub(ADDR, sub_p2);
        assert_eq!(r2.reason_codes.contains(&Some(SubscribeAckReason::GrantedQoSZero).unwrap()), true);

        let r3 = broker.accept_sub(ADDR, sub_p3);
        assert_eq!(r3.reason_codes.contains(&Some(SubscribeAckReason::GrantedQoSZero).unwrap()), true);
    }
  
    #[test]
    fn test_new_pub() {
        let mut broker = MBroker::new();
        let id = "1004".to_string();
        // Create a client's connect packet
        let conn_p1 = ConnectPacket {
            protocol_name: String::from("cm_mqtt"),
            protocol_version: ProtocolVersion::V500,
            clean_start: true,
            keep_alive: 1,
            user_properties: Vec::new(),
            client_id: id,
            session_expiry_interval: None,receive_maximum: None, maximum_packet_size: None,
            topic_alias_maximum: None,request_response_information: None, request_problem_information: None,
            authentication_method: None, authentication_data: None, will: None,user_name: None,password: None,
        };
        broker.accept_connect(ADDR, conn_p1);

        // create a subscribe packet
        let sub_p = SubscribePacket {
            packet_id: 01,
            subscription_identifier: None,
            user_properties: Vec::new(),
            subscription_topics: vec![SubscriptionTopic {
                topic_filter: TopicFilter::Concrete { filter: "gwu".to_string(), level_count: 1 },
                maximum_qos: QoS::AtMostOnce,
                no_local: false,
                retain_as_published: false,
                retain_handling: RetainHandling::SendAtSubscribeTime,
            }],
        };
        broker.accept_sub(ADDR, sub_p);

        // create publish packet
        let pub_p = PublishPacket {
            is_duplicate: false,
            qos: QoS::AtMostOnce, 
            retain: false,
            user_properties: Vec::new(),
            topic: "gwu".parse::<Topic>().unwrap(),
            payload: Bytes::from("George Washington University"),
            packet_id: None, payload_format_indicator: None, message_expiry_interval: None, 
            topic_alias: None, response_topic: None, content_type: None, correlation_data: None, 
            subscription_identifier: None
        };

        let res = broker.accept_pub(ADDR, pub_p);
        assert_eq!(res.0.reason_code, PublishAckReason::Success);
        println!("{:?}", res.1);  // two expected
    }

    #[test]
    fn test_new_pub_2clients() {
        let mut broker = MBroker::new();
        // let id = "1004".to_string();
        // Create a client's connect packet
        let conn_p1 = ConnectPacket {
            protocol_name: String::from("cm_mqtt"),
            protocol_version: ProtocolVersion::V500,
            clean_start: true,
            keep_alive: 1,
            user_properties: Vec::new(),
            client_id: ADDR.to_string(),
            session_expiry_interval: None,receive_maximum: None, maximum_packet_size: None,
            topic_alias_maximum: None,request_response_information: None, request_problem_information: None,
            authentication_method: None, authentication_data: None, will: None,user_name: None,password: None,
        };
        broker.accept_connect(ADDR, conn_p1);

        // Create a client's connect packet
        let conn_p1 = ConnectPacket {
            protocol_name: String::from("cm_mqtt"),
            protocol_version: ProtocolVersion::V500,
            clean_start: true,
            keep_alive: 1,
            user_properties: Vec::new(),
            client_id: ADDR2.to_string(),
            session_expiry_interval: None,receive_maximum: None, maximum_packet_size: None,
            topic_alias_maximum: None,request_response_information: None, request_problem_information: None,
            authentication_method: None, authentication_data: None, will: None,user_name: None,password: None,
        };
        broker.accept_connect(ADDR2, conn_p1);

        // create a subscribe packet
        let sub_p = SubscribePacket {
            packet_id: 01,
            subscription_identifier: None,
            user_properties: Vec::new(),
            subscription_topics: vec![SubscriptionTopic {
                topic_filter: TopicFilter::Concrete { filter: "gwu".to_string(), level_count: 1 },
                maximum_qos: QoS::AtMostOnce,
                no_local: false,
                retain_as_published: false,
                retain_handling: RetainHandling::SendAtSubscribeTime,
            }],
        };
        broker.accept_sub(ADDR, sub_p);

        // create publish packet
        let pub_p = PublishPacket {
            is_duplicate: false,
            qos: QoS::AtMostOnce, 
            retain: false,
            user_properties: Vec::new(),
            topic: "gwu".parse::<Topic>().unwrap(),
            payload: Bytes::from("George Washington University"),
            packet_id: None, payload_format_indicator: None, message_expiry_interval: None, 
            topic_alias: None, response_topic: None, content_type: None, correlation_data: None, 
            subscription_identifier: None
        };

        let res = broker.accept_pub("192.0.2.1", pub_p);
        assert_eq!(res.0.reason_code, PublishAckReason::Success);
        println!("{:?}", res.1);  // two expected
    }
}