mod broker;
mod msg_parser;
// use crate::msg_parser::msg_parser::{test_connect_encode_decode, test_publish_encode_decode};
// use crate::broker::broker::MBroker;
// use msg_parser::msg_parser::{test_connect_encode_decode, test_publish_encode_decode};

// use mqtt_v5::types::{SubscribePacket, SubscriptionTopic, QoS, RetainHandling};
// use bytes::{BytesMut, Bytes};


fn main() {
    // Create new broker
    // let mut broker = MBroker::new();

    println!("SUCCESS");
}

#[cfg(test)]

mod tests {
    use crate::broker::broker::MBroker;
    use mqtt_v5::types::{Packet, PublishPacket, SubscribePacket, ConnectPacket, ConnectReason, QoS,
        ProtocolVersion, RetainHandling, SubscriptionTopic};
    use bytes::{Bytes, BytesMut};
    use crate::msg_parser::msg_parser::{cm_decode, cm_encode};
    
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
    
        let res = broker.accept_new_client(conn_p);
        assert_eq!(res.reason_code, ConnectReason::Success);
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

        let r1 = broker.accept_new_client(conn_p1);
        let r2 = broker.accept_new_client(conn_p2);

        assert_eq!(r1.reason_code, ConnectReason::Success);
        assert_eq!(r2.reason_code, ConnectReason::Success);
    }

    #[test]
    fn test_new_sub() {
        let mut broker = MBroker::new();
        // create a subscribe packet
        let sub_p = SubscribePacket {
            packet_id: 01,
            subscription_identifier: None,
            user_properties: Vec::new(),
            subscription_topics: vec![SubscriptionTopic {
                topic_filter: "gwu".parse().unwrap(),
                maximum_qos: QoS::AtLeastOnce,
                no_local: false,
                retain_as_published: false,
                retain_handling: RetainHandling::SendAtSubscribeTime,
            }],
        };

        let res = broker.accept_sub(sub_p);
        assert_eq!(res.packet_id, 01);
    }

    #[test]
    fn test_multiple_subs_gwu() {
        let mut broker = MBroker::new();
        // create subscribe packets
        let sub_p1 = SubscribePacket {
            packet_id: 01,
            subscription_identifier: None,
            user_properties: Vec::new(),
            subscription_topics: vec![SubscriptionTopic {
                topic_filter: "gwu".parse().unwrap(),
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
                topic_filter: "gwu/ccas".parse().unwrap(),
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
                topic_filter: "gwu/seas".parse().unwrap(),
                maximum_qos: QoS::AtLeastOnce,
                no_local: false,
                retain_as_published: false,
                retain_handling: RetainHandling::SendAtSubscribeTime,
            }],
        };

        let r1 = broker.accept_sub(sub_p1);
        assert_eq!(r1.packet_id, 01);

        let r2 = broker.accept_sub(sub_p2);
        assert_eq!(r2.packet_id, 02);

        let r3 = broker.accept_sub(sub_p3);
        assert_eq!(r3.packet_id, 03);
    }

    #[test]
    fn test_multiple_subs_unis() {
        let mut broker = MBroker::new();
        // create subscribe packets
        let sub_p1 = SubscribePacket {
            packet_id: 01,
            subscription_identifier: None,
            user_properties: Vec::new(),
            subscription_topics: vec![SubscriptionTopic {
                topic_filter: "gwu".parse().unwrap(),
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
                topic_filter: "udel".parse().unwrap(),
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
                topic_filter: "uwm".parse().unwrap(),
                maximum_qos: QoS::AtLeastOnce,
                no_local: false,
                retain_as_published: false,
                retain_handling: RetainHandling::SendAtSubscribeTime,
            }],
        };

        let r1 = broker.accept_sub(sub_p1);
        assert_eq!(r1.packet_id, 01);

        let r2 = broker.accept_sub(sub_p2);
        assert_eq!(r2.packet_id, 02);

        let r3 = broker.accept_sub(sub_p3);
        assert_eq!(r3.packet_id, 03);
    }

}