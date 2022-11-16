mod broker;
mod msg_parser;
use crate::msg_parser::msg_parser::{test_connect_encode_decode, test_publish_encode_decode};
use crate::broker::broker::MBroker;
// use msg_parser::msg_parser::{test_connect_encode_decode, test_publish_encode_decode};

use mqtt_v5::types::{ConnectPacket, ConnectReason, ProtocolVersion, SubscribePacket, SubscriptionTopic, QoS, RetainHandling};
// use bytes::{BytesMut, Bytes};

fn test_new_client(broker: &mut MBroker, id: String) {
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

    // let mut broker = MBroker::new();
    let res = broker.accept_new_client(conn_p);
    assert_eq!(res.reason_code, ConnectReason::Success);
}

fn test_new_sub(broker: &mut MBroker, topic: String, id: u16) {
    // create a subscribe packer
    let sub_p = SubscribePacket {
        packet_id: id,
        subscription_identifier: None,
        user_properties: Vec::new(),
        subscription_topics: vec![SubscriptionTopic {
            topic_filter: topic.parse().unwrap(),
            maximum_qos: QoS::AtLeastOnce,
            no_local: false,
            retain_as_published: false,
            retain_handling: RetainHandling::SendAtSubscribeTime,
        }],
    };

    let res = broker.accept_sub(sub_p);
    assert_eq!(res.packet_id, id);
}

fn main() {
    // Create new broker
    let mut broker = MBroker::new();

    println!("Testing adding new client 1004");
    test_new_client(&mut broker, "1004".to_string());
    println!("SUCCESS: added new client\n");

    println!("Testing adding new client 1005");
    test_new_client(&mut broker, "1005".to_string());

    println!("Testing connect packet parsing");
    test_connect_encode_decode();
    println!("SUCCESS: Connect packet parsed\n");

    println!("Testing publish packet parsing");
    test_publish_encode_decode();
    println!("SUCCESS: Publish packet parsed\n");

    println!("Testing subscribing to a topic");
    test_new_sub(&mut broker, "gwu".to_string(), 01);
    println!("SUCCESS: Topic 'gwu' subscribed");

    println!("Testing subscribing to a topic");
    test_new_sub(&mut broker, "gwu".to_string(), 02);
    println!("SUCCESS: Topic 'gwu/ccas' subscribed");

    println!("Testing subscribing to a topic");
    test_new_sub(&mut broker, "uwm".to_string(), 03);
    println!("SUCCESS: Topic 'gwu/seas' subscribed");

    println!("SUCCESS");
}

// #[cfg(test)]

//     mod tests {
//         #[test]
//         fn test_new_client() {
//             let mut broker = MBroker::new();
//             let id = "1004".to_string();
//             // Create a Connect packet
//             let conn_p = ConnectPacket {
//                 protocol_name: String::from("cm_mqtt"),
//                 protocol_version: ProtocolVersion::V500,
//                 clean_start: true,
//                 keep_alive: 1,
//                 user_properties: Vec::new(),
//                 client_id: id,
//                 session_expiry_interval: None,
//                 receive_maximum: None,
//                 maximum_packet_size: None,
//                 topic_alias_maximum: None,
//                 request_response_information: None,
//                 request_problem_information: None,
//                 authentication_method: None,
//                 authentication_data: None,
//                 will: None,
//                 user_name: None,
//                 password: None,
//             };
        
//             // let mut broker = MBroker::new();
//             let res = broker.accept_new_client(conn_p);
//             assert_eq!(res.reason_code, ConnectReason::Success);
//         }

// }