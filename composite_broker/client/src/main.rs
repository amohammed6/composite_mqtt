
use byte::BytesExt; 
use mqtt_sn::{
    ClientId, Connect, Flags, Message, Publish, PublishData, Subscribe, TopicName, TopicNameOrId,
    Unsubscribe,
};
use std::borrow::Borrow;
use std::io::{self};
use std::time::{Instant}; // Duration
use std::net::UdpSocket; 
use std::{env, str};

fn send_connect(socket: &UdpSocket, addr: String) {
    // make connect packet
    let packet = Message::Connect(Connect {
        flags: Flags::default(),
        duration: 30,
        client_id: ClientId::from("bench"),
    });

    // encode it
    let mut buf = [0u8; 128]; // create buffer for encoding
    let mut len = 0usize;
    buf.write(&mut len, packet.clone()).unwrap();

    // println!("\tSending ConnectPacket for a new client\n");
    socket
        .send_to(&buf.as_mut(), addr)
        .expect("Couldn't send to broker");
}

fn send_sub(topic: String, packet_num: &u16, socket: &UdpSocket, addr: String) -> u16 {
    let packet: Message;
    let p_num = *packet_num;
    if topic.parse::<u16>().is_err() {
        // make sub packet
        // println!("Is a string {}", topic);
        packet = Message::Subscribe(Subscribe {
            flags: Flags::default(),
            msg_id: p_num,
            topic: TopicNameOrId::Name(TopicName::from(&topic)),
        });
    } else {
        // make sub packet
        let mut flags = Flags::default();
        flags.set_topic_id_type(0x2); // necessary for topic_id
        packet = Message::Subscribe(Subscribe {
            flags,
            msg_id: p_num,
            topic: match topic.parse::<u16>() {
                Ok(num) => {
                    // is a number
                    // println!("Is a number {}", num);
                    TopicNameOrId::Id(num as u16)
                }
                Err(_n) => {
                    // is a string
                    println!("Is a string {}", topic);
                    TopicNameOrId::Id(0)
                }
            },
        });
    }

    // encode it
    let mut buf = [0u8; 128]; // create buffer for encoding
    let mut len = 0usize;
    buf.write(&mut len, packet).unwrap();

    // send it
    // println!("\tSending SubscribePacket for topic: {}", topic);
    socket
        .send_to(&buf.as_mut(), addr)
        .expect("Couldn't send to broker");
    p_num + 1
}

fn send_pub(topic_id: u16, data: String, packet_num: &u16, socket: &UdpSocket, addr: String) -> u16 {
    // (u16, [u8; 128])
    let content: &str = &data;
    let p_num = *packet_num;
    println!("{}", p_num);
    // make publish packet
    let packet = Message::Publish(Publish {
        flags: Flags::default(),
        topic_id: topic_id,
        msg_id: p_num,
        data: PublishData::from(content.trim()), // .strip_suffix("\n").expect("Can't strip")
    });

    // encode it
    let mut buf = [0u8; 128]; // create buffer for encoding
    let mut len = 0usize;
    buf.write(&mut len, packet).expect("Didn't write to buffer");

    // send it
    // println!("\tSending PublishPacket for topic '{}'", topic_id);
    socket
        .send_to(&buf.as_mut(), addr)
        .expect("Couldn't send to broker");
    p_num + 1
}

fn send_unsub(topic_name: String, packet_num: &u16, socket: &UdpSocket, addr: String) -> u16 {
    let p_num = *packet_num;
    let topic = topic_name
        .strip_prefix("unsub ")
        .expect("Couldn't strip")
        .strip_suffix("\n")
        .expect("Can't strip");

    // make the unsub packet
    let packet = Message::Unsubscribe(Unsubscribe {
        flags: match topic.parse::<u16>() {
            Ok(_) => {
                let mut flags = Flags::default();
                flags.set_topic_id_type(0x2); // necessary for topic_id
                flags
            }
            Err(_) => Flags::default(),
        },
        msg_id: p_num,
        topic: match topic.parse::<u16>() {
            Ok(num) => TopicNameOrId::Id(num as u16),
            Err(_n) => TopicNameOrId::Name(topic.into()),
        },
    });
    let mut buf = [0u8; 128]; // create buffer for encoding
    let mut len = 0usize;
    buf.write(&mut len, packet).unwrap();

    // send it
    println!("\tSending SubscribePacket for topic: {}", topic);
    socket
        .send_to(&buf.as_mut(), addr)
        .expect("Couldn't send to broker");

    p_num + 1
}

fn read_publish_packet(buf: [u8; 512]) {
    // decode
    let p: Message = buf.read(&mut 0).unwrap();
    // println!("Receiving data");
    match p {
        Message::Publish(m) => {
            //println!("Received publish packet: {:?}", m.data.as_str())
        }
        _ => {}
    }
}


/*
    I change the port with each run
    Run with
        cargo run -- 127.0.0.1 8000
        cargo run -- 127.0.0.1 7878
*/

fn main() {
    let broker = "10.10.1.2:8888";
    let mut packet_num = 1;
    let args: Vec<String> = env::args().collect(); // collect address from command line
    let bind_addr = args[1].clone() + ":" + args[2].borrow(); // concatenate to make the socket addr
    let mut ack_buffer = [0u8; 512];
    let mut topic_id: u16 = 0;

    let socket = UdpSocket::bind(bind_addr).expect("Could not bind client socket");
    send_connect(&socket, broker.to_string());

    socket
        .recv_from(&mut ack_buffer.as_mut())
        .expect("Could not read into buffer");

    let p: Message = ack_buffer.read(&mut 0).unwrap();
    match p {
        Message::ConnAck(_m) => {
            println!("\tConnack packet received");
            read_publish_packet(ack_buffer);
        }
        _ => {
            println!("Connack packet not received");
        }
    }

    packet_num = send_sub(
        "topic1".to_string(),
        &packet_num, 
        &socket, 
        broker.to_string());

    socket
        .recv_from(&mut ack_buffer.as_mut())
        .expect("Could not read into buffer");

    let p: Message = ack_buffer.read(&mut 0).unwrap();
    match p {
        Message::SubAck(m) => {
            println!("\tSuback packet received");
            topic_id = m.topic_id;
        }
        _ => {
            println!("Suback packet not received");
        }
    }

    let bench_time = Instant::now();

    for i in 0..1000 {
        let sock = socket.try_clone().expect("Failed to clone socket");


        packet_num = send_pub(
            topic_id,
            "topic1:test".to_string(),
            &mut 0,
            &sock,
            broker.to_string()
        );

        // receive from broker
        socket
            .recv_from(&mut ack_buffer.as_mut())
            .expect("Could not read into buffer");

        // check that the received bytes are the ack packet and decode
        let p: Message = ack_buffer.read(&mut 0).unwrap();
        match p {
            Message::Publish(_m) => {
                read_publish_packet(ack_buffer);
            }
            _ => {
                println!("Publish packet not received");
            }
        }
    }

    println!("AVG Round-trip time: {:.2?}", bench_time.elapsed()/1000);
}
