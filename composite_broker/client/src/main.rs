mod msg_parser;
use std::{str,env};
use std::net::{UdpSocket}; // SocketAddr
use std::borrow::Borrow;
use std::io::{self};  // ,Write, prelude::*,BufReader
use mqtt_v5::{encoder, types::{Packet, ConnectPacket, PublishPacket, SubscribePacket, SubscriptionTopic, QoS, 
    RetainHandling, ProtocolVersion}, topic::{TopicFilter, Topic}}; // decoder
use bytes::{Bytes, BytesMut};
use crate::msg_parser::msg_parser::{cm_decode};

fn send_connect() -> BytesMut {
    // make connect packet
    let packet = Packet::Connect(ConnectPacket {
        protocol_name: String::from("cm_mqtt"),
        protocol_version: ProtocolVersion::V500,
        clean_start: true,
        keep_alive: 1,
        user_properties: Vec::new(),
        client_id: String::from("1001"),
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
    encoder::encode_mqtt(&packet, &mut buf, ProtocolVersion::V500);

    // return it
    println!("\tSending ConnectPacket for a new client\n");
    buf
}

fn send_sub(topic_name: String, packet_num: &u16) -> (u16, BytesMut) {
    let v: Vec<&str> = topic_name.split(" ").collect();
    let p_num = *packet_num;
    // make sub packet
    let packet = Packet::Subscribe(SubscribePacket {
        packet_id: p_num,
        subscription_identifier: None,
        user_properties: Vec::new(),
        subscription_topics: vec![SubscriptionTopic {
            topic_filter: TopicFilter::Concrete { 
                filter: v[1].strip_suffix("\n").expect("Can't strip").parse().unwrap(), 
                level_count: 1 },
            maximum_qos: QoS::AtLeastOnce,
            no_local: false,
            retain_as_published: false,
            retain_handling: RetainHandling::SendAtSubscribeTime,
        }],
    });

    // encode it
    let mut buf = BytesMut::new();      // create buffer for encoding 
    encoder::encode_mqtt(&packet, &mut buf, ProtocolVersion::V500);

    // return it
    println!("\tSending SubscribePacket for topic: {}", v[1]);
    (p_num + 1, buf)
}

fn send_pub(args: String, packet_num: &u16) -> (u16, BytesMut) {
    let topic_name = args.split(':').collect::<Vec<&str>>()[0];
    let content: String = args.split(':').collect::<Vec<&str>>()[1].to_string();
    let p_num = *packet_num;
    // make publish packet
    let packet = Packet::Publish(PublishPacket {
        is_duplicate: false,
        qos: QoS::AtLeastOnce,
        retain: true,
        topic: topic_name.clone().split_at(9).1.parse::<Topic>().unwrap(),
        user_properties: Vec::new(),
        payload: Bytes::from(content.clone()), // immutable to preserve security,
        packet_id: Some(*packet_num),                 // required
        payload_format_indicator: None,
        message_expiry_interval: None,
        topic_alias: None,
        response_topic: None,
        correlation_data: None,
        subscription_identifier: None,
        content_type: None,
    });

    // encode it
    let mut buf = BytesMut::new();      // create buffer for encoding
    encoder::encode_mqtt(&packet, &mut buf, ProtocolVersion::V500);

    // return it
    println!("\tSending PublishPacket for topic '{}'", topic_name.clone().split_at(9).1);
    return (p_num + 1, buf);
}

fn read_publish_packet(buf: [u8; 1500]) {
    let p = cm_decode(&buf);
    match p {
        Ok(Packet::Publish(p)) => {
            println!("Received publish packet: {}", String::from_utf8_lossy(&p.payload))
        },
        _=>{}
    }
}

/*
    I change the port with each run
    Run with 
        cargo run -- 127.0.0.1 8000
        cargo run -- 127.0.0.1 7878
*/

fn main() -> io::Result<()>{
    let mut packet_num = 00;
    // collect address from command line
    let args: Vec<String> = env::args().collect();
    // concatenate to make the socket addr
    let bind_addr = args[1].clone() + ":"+ args[2].borrow();

    // make the socket
    let socket = UdpSocket::bind(bind_addr).expect("Could not bind client socket");

    loop {
        let mut input = String::new();
        let mut ack_buffer = [0u8; 1500];
        // let mut pub_buf = [0u8; 1500];
        let mut ret_buf = BytesMut::new();

        // read the input from the command line
        io::stdin().read_line(&mut input).expect("Failed to read from stdin");
        if input.as_bytes() != "\n".as_bytes() {
            socket.send_to(input.as_bytes(), args[1].clone() + ":8888").expect("Failed to write to server");
        }
        else {continue;}
        
        // add logic for calling the functions
        if input.contains("connect") {
            ret_buf = send_connect();
        }
        else if input.contains("sub") {
            let res = send_sub(input, &packet_num);
            packet_num = res.0;
            ret_buf = res.1;
        }
        else if input.contains("pub") {
            let res = send_pub(input, &packet_num);
            packet_num = res.0;
            ret_buf = res.1;
        }

        // send it to the broker
        socket.send_to(&ret_buf.as_mut(), args[1].clone() + ":8888").expect("Couldn't send to broker");

        // receive from broker
        socket.recv(&mut ack_buffer.as_mut()).expect("Could not read into buffer");

        // check that the received bytes are the ack packet
        let p = cm_decode(&ack_buffer);
        match p {
            Ok(Packet::ConnectAck(p)) => {
                println!("\tConnect Ack packet received: {:?}\n", p.reason_string.unwrap().0);
            },
            Ok(Packet::SubscribeAck(p)) => {
                println!("\tSubscribe Ack packet received: {:?}\n", p.reason_string.unwrap().0);
            },
            Ok(Packet::PublishAck(p)) => {
                println!("\tPublish Ack packet received: {:?}\n", p.reason_string.unwrap().0);
                socket.recv(&mut ack_buffer.as_mut()).expect("Could not read into buffer");
                if !ack_buffer.is_empty() {
                    read_publish_packet(ack_buffer);
                }
            }
            _=> {
                println!("Ack packet not received");
            }
        }
    }

    /* 
    let mut packet_num = 00;
    // Struct used to start requests to the server.
    // let args: Vec<String> = env::args().collect(); // 0.0.0.0:8888, 127.0.0.1:7878, 127.0.0.1:8888
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;             // Check TcpStream Connection to the server
    println!("TCP Connection to MusQraTT Broker Starting...");
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
        // println!("");
    }
    */
    // Ok(())
}