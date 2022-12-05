pub mod msg_parser {
    use bytes::{BytesMut};
    use mqtt_v5::{
        decoder, encoder,
        types::{ProtocolVersion},
    };

    #[allow(dead_code)]
    pub fn cm_encode(
        packet: mqtt_v5::types::Packet,
        buffer: &mut BytesMut,
    ) -> Result<&mut BytesMut, String> {
        encoder::encode_mqtt(&packet, buffer, ProtocolVersion::V500);
        if buffer.is_empty() {
            Err("Packet wasn't encoded".to_string())
        } else {
            Ok(buffer)
        }
    }

    // Decode function
    //  input: bytes of encoded packet
    //  output: packet
    pub fn cm_decode(buffer: & [u8]) -> Result<mqtt_v5::types::Packet, String> {
        let mut b = BytesMut::from(buffer);
        if buffer.is_empty() {
            Err("Buffer was empty".to_string())
        } else {
            Ok(decoder::decode_mqtt(&mut b, ProtocolVersion::V500)
                .unwrap()
                .unwrap())
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
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
    
}