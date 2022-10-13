pub mod msg_parser {
    use mqtt_v5::{
        types, encoder, decoder, 
    };
    use bytes::{BytesMut, Bytes}; 
    
    pub fn parse_test () {
        // Create a Connect packet
        let c_id1: String = String::from("1004");
        let conn_p = types::ConnectPacket{
            protocol_name: String::from("cm_mqtt"),
            protocol_version: types::ProtocolVersion::V500,
            clean_start: true,
            keep_alive: 1,
            user_properties: Vec::new(),
            client_id: String::from("1004"),
            session_expiry_interval: None, receive_maximum: None, maximum_packet_size: None, topic_alias_maximum: None,
            request_response_information: None, request_problem_information: None, authentication_method: None,
            authentication_data: None, will: None, user_name: None, password: None,
        };
        let packet = types::Packet::Connect(conn_p);

        // create buffer for encoding
        let mut buf = BytesMut::new();

        // testing block
        println!("Connect Packet test");
        assert_eq!(cm_encode(packet, &mut buf), false);
        println!("\tConnect packet encoding success");
        

        // decode the connect packet
        let mut conn_client_id: String = String::new();
        let decode = cm_decode(&mut buf);
        match decode {
            types::Packet::Connect(connect_packet) => {
                conn_client_id = connect_packet.client_id;
            },
            _=> {}
        }

        // testing block
        assert_eq!(c_id1, conn_client_id);
        println!("\tConnect packet decoding success");

        // Create a Publish packet
        let top : String = String::from("gwu");
        let pub_p = types::PublishPacket{
            is_duplicate: false, 
            qos: types::QoS::AtLeastOnce,
            retain: true,
            topic: "gwu".parse().unwrap(), 
            user_properties: Vec::new(),
            payload: Bytes::from("this is gwu"), // immutable to preserve security,
            packet_id: Some(42),    // required
            payload_format_indicator: None, message_expiry_interval: None, topic_alias: None, response_topic: None,
            correlation_data: None, subscription_identifier: None, content_type: None,
        };
        let packet2 = types::Packet::Publish(pub_p);

        // create the buffer for encoding
        let mut buf2 = BytesMut::new();

        // testing block
        println!("Publish Packet test");
        assert_eq!(cm_encode(packet2, &mut buf2), false);
        println!("\tPublish packet encoding success");

        // decode publish packet
        let mut pub_topic: String = String::new(); 
        let decode2 = cm_decode(&mut buf2);
        match decode2 {
            types::Packet::Publish(publish_packet) => {
                pub_topic = String::from(publish_packet.topic.topic_name());
            },
            _=> {}
        }

        // testing block
        assert_eq!(pub_topic, top);
        println!("\tPublish packet decoding success");
    }
    
    pub fn cm_encode(
        packet: mqtt_v5::types::Packet, 
        buffer: &mut BytesMut) 
        -> bool {
        encoder::encode_mqtt(&packet, buffer, types::ProtocolVersion::V500);
        buffer.is_empty()
    }
    
    // Decode function
    //  input: bytes of encoded packet
    //  output: packer
    pub fn cm_decode(buffer: &mut BytesMut) -> mqtt_v5::types::Packet {
        decoder::decode_mqtt(buffer, types::ProtocolVersion::V500).unwrap().unwrap()
    }
}

