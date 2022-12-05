use std::str;
use std::net::TcpStream;
use std::io::{self,Write, prelude::*,BufReader}; 
use mqtt_v5::{encoder, types::{Packet, ConnectPacket, PublishPacket, SubscribePacket, SubscriptionTopic, QoS, 
    RetainHandling, ProtocolVersion}};
use bytes::{Bytes, BytesMut};

fn send_connect(mut stream: &TcpStream) {
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
    // cm_encode(packet, &mut buf); 
    encoder::encode_mqtt(&packet, &mut buf, ProtocolVersion::V500);

    // write to stream
    stream.write(buf.as_mut()).expect("failed to send connect packet");
}

fn send_sub(mut stream: &TcpStream, topic_name: String, packet_num: &u16) -> u16 {
    let v: Vec<&str> = topic_name.split(" ").collect();
    // make sub packet
    let packet = Packet::Subscribe(SubscribePacket {
        packet_id: *packet_num,
        subscription_identifier: None,
        user_properties: Vec::new(),
        subscription_topics: vec![SubscriptionTopic {
            topic_filter: v[1].parse().unwrap(),
            maximum_qos: QoS::AtLeastOnce,
            no_local: false,
            retain_as_published: false,
            retain_handling: RetainHandling::SendAtSubscribeTime,
        }],
    });
    // increment the packet number
    let p_num = *packet_num;

    // encode it
    let mut buf = BytesMut::new();      // create buffer for encoding
    // cm_encode(packet, &mut buf); 
    encoder::encode_mqtt(&packet, &mut buf, ProtocolVersion::V500);

    // write to stream
    stream.write(buf.as_mut()).expect("failed to send subscribe packet");

    p_num + 1
}

fn send_pub(mut stream: &TcpStream, args: String, packet_num: &u16) ->u16 {
    // let v: Vec<&str> = args.split(':').collect();
    let topic_name = args.split(':').collect::<Vec<&str>>()[0];
    let content: String = args.split(':').collect::<Vec<&str>>()[1].to_string();
    // make publish packet
    let packet = Packet::Publish(PublishPacket {
        is_duplicate: false,
        qos: QoS::AtLeastOnce,
        retain: true,
        topic: topic_name.split_at(8).1.parse().unwrap(),
        user_properties: Vec::new(),
        payload: Bytes::from(content), // immutable to preserve security,
        packet_id: Some(*packet_num),                 // required
        payload_format_indicator: None,
        message_expiry_interval: None,
        topic_alias: None,
        response_topic: None,
        correlation_data: None,
        subscription_identifier: None,
        content_type: None,
    });
    // increment the packet number
    let p_num = *packet_num;

    // encode it
    let mut buf = BytesMut::new();      // create buffer for encoding
    // cm_encode(packet, &mut buf); 
    encoder::encode_mqtt(&packet, &mut buf, ProtocolVersion::V500);

    // write to stream
    stream.write(buf.as_mut()).expect("failed to send subscribe packet");

    p_num + 1
}

fn main() -> io::Result<()>{
    let mut packet_num = 00;
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
            
        if input.contains("connect") {
            send_connect(&stream);
        }
        // subscribe to a topic
        else if input.contains("sub") {
            // let v = input.split(' ').collect();
            packet_num = send_sub(&stream, input, &packet_num);
        }
        // publish to a topic
        else if input.contains("pub") {
            // let v = input.split(':').collect();
            packet_num = send_pub(&stream, input, &packet_num);
        }
        println!("");
    }
    Ok(())
}