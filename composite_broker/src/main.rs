use mqtt_v5::{
    types, encoder, decoder, // topic::Topic,
};
use bytes::{BytesMut, Bytes}; // Bytes
// use std::str::FromStr;

fn main() {
    // create a packet : pkt
    let empty_vec1: Vec::<types::properties::UserProperty> = Vec::new();
    let conn_p = types::ConnectPacket{
        protocol_name: String::from("hey"),
        protocol_version: types::ProtocolVersion::V500,
        clean_start: true,
        keep_alive: 1,
        user_properties: empty_vec1,
        client_id: String::from("1004"),
        session_expiry_interval: None, receive_maximum: None, maximum_packet_size: None, topic_alias_maximum: None,
        request_response_information: None, request_problem_information: None, authentication_method: None,
        authentication_data: None, will: None, user_name: None, password: None,
    };

    let packet = types::Packet::Connect(conn_p);

    // encode the packet
    let mut buf = BytesMut::with_capacity(64);

    encoder::encode_mqtt(&packet, &mut buf, types::ProtocolVersion::V500);

    // decode the packet
    let (mut r_pro_name_c, mut r_client_id_c): (String, String) = (" ".to_string()," ".to_string());
    let decode = decoder::decode_mqtt(&mut buf, types::ProtocolVersion::V500);
    let decode_packet = decode.unwrap().unwrap();
    match decode_packet {
        types::Packet::Connect(connect_packet) => {
            r_pro_name_c = connect_packet.protocol_name;
            r_client_id_c = connect_packet.client_id;
        },
        _=> {}
    }
    println!("Connect Packet test");
    println!("{:?} and id of {:?}", r_pro_name_c, r_client_id_c);
    

    // Creating Publish packet
    let empty_vec2: Vec::<types::properties::UserProperty> = Vec::new();
    let pub_p = types::PublishPacket{
        is_duplicate: false, 
        qos: types::QoS::AtLeastOnce,
        retain: true,
        topic: "gwu".parse().unwrap(), 
        user_properties: empty_vec2,
        payload: Bytes::from("this is gwu"), // immutable to preserve security,
        packet_id: Some(42),    // required
        payload_format_indicator: None, message_expiry_interval: None, topic_alias: None, response_topic: None,
        correlation_data: None, subscription_identifier: None, content_type: None,
    };

    // encode publish packet
    let packet2 = types::Packet::Publish(pub_p);
    let mut buf2 = BytesMut::new();

    encoder::encode_mqtt(&packet2, &mut buf2, types::ProtocolVersion::V500);
    
    // decode publish packet
    let mut r_topic_p: String = "".to_string(); // (mut r_pro_name_p, mut r_client_id_p): (String, String) = (" ".to_string()," ".to_string());
   
    let decode2 = decoder::decode_mqtt(&mut buf2, types::ProtocolVersion::V500).unwrap().unwrap();
    // let decode_packet2 = decode2.unwrap().unwrap();

    // println!("{:?}", decode2);

    match decode2 {
        types::Packet::Publish(publish_packet) => {
            r_topic_p = String::from(publish_packet.topic.topic_name());
        },
        _=> {}
    }
    println!("Publish Packet test");
    println!("Published to topic: {:?} ", r_topic_p);
}
