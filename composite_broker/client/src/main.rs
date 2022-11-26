// mod super::msg_parser;
use std::str;
use std::net::TcpStream;
use std::io::{self,Write, prelude::*,BufReader}; 
use crate::super::msg_parser::msg_parser::{cm_decode, cm_encode};
use mqtt_v5::types::{Packet, ConnectPacket, ProtocolVersion};
use bytes::{BytesMut};


fn accept_connect(mut stream: &TcpStream) {
    // make connect packet
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

    // encode it
    let mut buf = BytesMut::new();      // create buffer for encoding
    cm_encode(packet, &mut buf);

    // write to stream
    stream.write(buf.as_mut()).expect("failed to send connect packet");
}

fn main() -> io::Result<()>{
    // Struct used to start requests to the server.
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;             // Check TcpStream Connection to the server
    for _ in 0..1000 {
        let mut input = String::new();                                  // Allow sender to enter message input 
        io::stdin().read_line(&mut input).expect("Failed to read");     // First access the input message and read it
        stream.write(input.as_bytes()).expect("failed to write");       // Write the message so that the receiver can access it 

        // Add buffering so that the receiver can read messages from the stream
        let mut reader =BufReader::new(&stream);
        let mut buffer: Vec<u8> = Vec::new();               // Check if this input message values are u8
        reader.read_until(b'\n',&mut buffer)?;              // Read input information
            
        if str::from_utf8(&buffer).unwrap().contains("connect") {
            accept_connect(&stream);
            
        }
        // println!("read from server:{}",str::from_utf8(&buffer).unwrap());
        println!("");
    }
    Ok(())
}