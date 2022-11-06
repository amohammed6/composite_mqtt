mod msg_parser;
mod broker;
use crate::msg_parser::msg_parser::{test_publish_encode_decode, test_connect_encode_decode};
use crate::broker::broker::MBroker;

use mqtt_v5::{
    types::{ConnectPacket, ConnectReason, ProtocolVersion}, // encoder, decoder, 
};
// use bytes::{BytesMut, Bytes};

fn test_new_client() {
    // Create a Connect packet
    let conn_p = ConnectPacket{
        protocol_name: String::from("cm_mqtt"),
        protocol_version: ProtocolVersion::V500,
        clean_start: true,
        keep_alive: 1,
        user_properties: Vec::new(),
        client_id: String::from("1004"),
        session_expiry_interval: None, receive_maximum: None, maximum_packet_size: None, topic_alias_maximum: None,
        request_response_information: None, request_problem_information: None, authentication_method: None,
        authentication_data: None, will: None, user_name: None, password: None,
    };

    let mut broker = MBroker::new();
    let res = broker.accept_new_client(conn_p);
    assert_eq!(res.reason_code, ConnectReason::Success);
}

fn main() {
    println!("Testing adding new client");
    test_new_client();
    println!("\tAdding new client \nSUCCESS\n");

    println!("Testing connect packet parsing");
    test_connect_encode_decode();
    println!("\tConnect packet parsed \nSUCCESS\n");

    println!("Testing publish packet parsing");
    test_publish_encode_decode();
    println!("\tPublish packet parsed \nSUCCESS\n");

    println!("SUCCESS");
}
