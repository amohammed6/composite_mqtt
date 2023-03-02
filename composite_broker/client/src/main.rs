use std::{str,env};
use std::io::{self};  
use byte::{BytesExt}; // TryWrite
use std::borrow::Borrow;
use std::net::{UdpSocket}; // SocketAddr
use mqtt_sn::{Message, Connect, Flags, ClientId, Subscribe, TopicName, Publish, PublishData, TopicNameOrId, 
    Unsubscribe};

fn send_connect(socket: &UdpSocket, addr: String)  { // -> [u8;128]
    // make connect packet
    let packet = Message::Connect(Connect {
        flags: Flags::default(),
        duration: 30,
        client_id: ClientId::from("1001")
    });

    // encode it
    let mut buf = [0u8; 128];      // create buffer for encoding
    let mut len = 0usize;
    buf.write(&mut len, packet.clone()).unwrap(); // .expect("Didn't write to buffer");


    // send it
    println!("\tSending ConnectPacket for a new client\n");
    socket.send_to(&buf.as_mut(), addr).expect("Couldn't send to broker");
    // buf
}

fn send_sub(topic_name: String, packet_num: &u16, socket: &UdpSocket, addr: String) -> u16  { // (u16, [u8; 128])
    let packet : Message;
    let p_num = *packet_num;
    let topic = topic_name.strip_prefix("sub ").expect("Couldn't strip").strip_suffix("\n").expect("Can't strip");
    // let topic = v[1]..trim_start();
    if topic.parse::<u16>().is_err() {
        // make sub packet
        println!("Is a string {}", topic);
        packet = Message::Subscribe(Subscribe {
            flags: Flags::default(),
            msg_id: p_num,
            topic: TopicNameOrId::Name(TopicName::from(topic))
        });
    }
    else {
        // make sub packet
        let mut flags = Flags::default();
        flags.set_topic_id_type(0x2); // topic_id
        packet = Message::Subscribe(Subscribe {
            flags,
            msg_id: p_num,
            topic: match topic.parse::<u16>() {
                Ok(num) => {
                    // is a number
                    println!("Is a number {}", num);
                    TopicNameOrId::Id(num as u16)
                },
                Err(_n) => {
                    // is a string
                    println!("Is a string {}", topic);
                    TopicNameOrId::Name(topic.into())
                }
            }
        });
    }
    
    // encode it
    let mut buf = [0u8; 128];      // create buffer for encoding 
    let mut len = 0usize;
    buf.write(&mut len, packet).unwrap();

    // send it
    println!("\tSending SubscribePacket for topic: {}", topic);
    socket.send_to(&buf.as_mut(), addr).expect("Couldn't send to broker");
    // (p_num + 1, buf)
    p_num + 1
}

fn send_pub(args: String, packet_num: &u16, socket: &UdpSocket, addr: String) -> u16 { // (u16, [u8; 128])
    let topic_name = args.split(':').collect::<Vec<&str>>()[0];
    let content: &str = args.split(':').collect::<Vec<&str>>()[1];
    let p_num = *packet_num;
    let topic_id: u16 = topic_name.strip_prefix("pub ").expect("Couldn't strip").parse().unwrap();
    // make publish packet
    let packet = Message::Publish(Publish {
        flags: Flags::default(),
        topic_id: topic_id,
        msg_id: p_num,
        data: PublishData::from(content.strip_suffix("\n").expect("Can't strip"))
    });

    // encode it
    let mut buf = [0u8; 128];      // create buffer for encoding
    let mut len = 0usize;
    buf.write(&mut len, packet).expect("Didn't write to buffer");

    // send it
    println!("\tSending PublishPacket for topic '{}'", topic_id);
    socket.send_to(&buf.as_mut(), addr).expect("Couldn't send to broker");
    // return (p_num + 1, buf);
    p_num + 1
}

fn send_unsub(topic_name: String, packet_num: &u16, socket: &UdpSocket, addr: String) -> u16 {
    // let v: Vec<&str> = topic_name.split(" ").collect();
    let p_num = *packet_num;
    let topic = topic_name.strip_prefix("unsub ").expect("Couldn't strip").strip_suffix("\n").expect("Can't strip");

    // make the unsub packet 
    let packet = Message::Unsubscribe(Unsubscribe { 
        flags: match topic.parse::<u16>() {
            Ok(_) => {
                let mut flags = Flags::default();
                flags.set_topic_id_type(0x2); // topic_id
                flags
            }
            Err(_) => {
                Flags::default()
            }
        },
        msg_id: p_num, 
        topic: match topic.parse::<u16>() {
            Ok(num) => {
                TopicNameOrId::Id(num as u16)
            }
            Err(_n) => {
                TopicNameOrId::Name(topic.into())
            }
        }
    });
    let mut buf = [0u8; 128];      // create buffer for encoding 
    let mut len = 0usize;
    buf.write(&mut len, packet).unwrap();

    // send it
    println!("\tSending SubscribePacket for topic: {}", topic);
    socket.send_to(&buf.as_mut(), addr).expect("Couldn't send to broker");

    p_num+1
}

fn read_publish_packet(buf: [u8; 1500]) {
    // decode
    let p: Message = buf.read(&mut 0).unwrap();
    // println!("Receiving data");
    match p {
        Message::Publish(m) => {
            println!("Received publish packet: {:?}", m.data.as_str())
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
    let args: Vec<String> = env::args().collect();      // collect address from command line
    let bind_addr = args[1].clone() + ":"+ args[2].borrow();   // concatenate to make the socket addr

    // make the socket
    let socket = UdpSocket::bind(bind_addr).expect("Could not bind client socket");

    loop {
        let mut input = String::new();
        let mut ack_buffer = [0u8; 1500];
        let sock = socket.try_clone().expect("Failed to clone socket");

        // read the input from the command line
        io::stdin().read_line(&mut input).expect("Failed to read from stdin");
        // if input.as_bytes() == "\n".as_bytes() {continue;}
        
        // add logic for calling the functions
        if !input.is_empty() {
            if input.contains("connect") {
                send_connect(&sock, "10.10.1.2:8888".to_string());
            }
            else if input.contains("unsub") {
                packet_num = send_unsub(
                    input.strip_prefix("mqtt_").expect("Couldn't strip").to_string(), 
                    &packet_num, 
                    &sock, 
                    "10.10.1.2:47138".to_string())
            }
            else if input.contains("sub") {
                packet_num = send_sub(
                    input.strip_prefix("mqtt_").expect("Couldn't strip").to_string(),
                    &packet_num, 
                    &sock, 
                    "10.10.1.2:47138".to_string());
            }
            else if input.contains("pub") {
                packet_num = send_pub(
                    input.strip_prefix("mqtt_").expect("Couldn't strip").to_string(), 
                    &packet_num, 
                    &sock, 
                    "10.10.1.2:47138".to_string());
            }
        }
   
        // receive from broker
        socket.recv_from(&mut ack_buffer.as_mut()).expect("Could not read into buffer");
        // socket.recv(&mut ack_buffer.as_mut()).expect("Could not read into buffer");

        // check that the received bytes are the ack packet and decode
        let p: Message = ack_buffer.read(&mut 0).unwrap();
        match p {
            Message::ConnAck(m) => {
                println!("\tConnect Ack packet received: {:?}\n", m.code);
            },
            Message::SubAck(m) => {
                println!("\tSubscribe Ack packet received: {:?}\n\tTopic id: {}", m.code, m.topic_id);
            },
            Message::PubAck(m) => {
                println!("\tPublish Ack packet received: {:?}\n", m.code);
                socket.recv(&mut ack_buffer.as_mut()).expect("Could not read into buffer");
                if !ack_buffer.is_empty() {
                    read_publish_packet(ack_buffer);
                }
            }
            Message::Publish(_m) => {
                println!("\tPublish packet received");
                read_publish_packet(ack_buffer);
            }
            Message::UnsubAck(m) => {
                println!("\tUnsubscribe Ack packet received: {:?}\n", m.code);
            }
            _=> {
                println!("Ack packet not received");
            }
        }
        
    }

}