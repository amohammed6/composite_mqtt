
use byte::BytesExt; 
use mqtt_sn::{
    ClientId, Connect, Flags, Message, Publish, PublishData, Subscribe, TopicName, TopicNameOrId,
    Unsubscribe,
};
use std::borrow::Borrow;
use std::io::{self};
use std::time::{SystemTime}; // Duration
use std::net::UdpSocket; 
use std::{env, str};
use std::{thread, time};
use std::sync::{Arc, Mutex};

fn send_connect(socket: &UdpSocket, addr: String, client_id: String) {
    // make connect packet
    let packet = Message::Connect(Connect {
        flags: Flags::default(),
        duration: 30,
        client_id: ClientId::from(&client_id[..]),
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

fn send_pub(topic_id: u16, data: String, packet_num: u8, socket: &UdpSocket, addr: String) {
    // (u16, [u8; 128])
    let content: &str = &data;
    let data = [packet_num];
    // make publish packet
    let packet = Message::Publish(Publish {
        flags: Flags::default(),
        topic_id: topic_id,
        msg_id: 0,
        data: PublishData::from(str::from_utf8(&data).unwrap()), // .strip_suffix("\n").expect("Can't strip")
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

fn recv_thread(socket: &UdpSocket, recv_time: &mut Arc<Mutex<&mut [u64;100]>>) {
    let mut ack_buffer = [0u8; 512];

    for i in 0..100 {
        // receive from broker
        socket
            .recv_from(&mut ack_buffer.as_mut())
            .expect("Could not read into buffer");

        // check that the received bytes are the ack packet and decode
        let p: Message = ack_buffer.read(&mut 0).unwrap();
        match p {
            Message::Publish(m) => {
                let id = m.data.as_bytes()[0];
                let mut times = recv_time.lock().unwrap();
                times[id as usize] = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
            }
            _ => {
                println!("Publish packet not received");
            }
        }

        if i % 10 == 0 {
            println!("Received {} Messages", i);
        }
    }
}

fn create_client(broker: &str, client_id: String) {
    let socket = UdpSocket::bind("0.0.0.0:0".to_string()).expect("Could not bind client socket");
    let packet_num = 1;

    send_connect(&socket, broker.to_string(), client_id);
    send_sub(
        "topic1".to_string(),
        &packet_num, 
        &socket, 
        broker.to_string());
}


fn main() {
    let packet_num = 1;
    let args: Vec<String> = env::args().collect(); // collect address from command line
    let bind_addr = "0.0.0.0:0";
    let broker = args[2].borrow();
    let num_clients: i32 = args[1].to_string().parse().unwrap(); // concatenate to make the socket addr
    let mut topic_id: u16 = 0;
    let mut ack_buffer = [0u8; 512];

    for i in 0..num_clients {
        create_client(broker, format!("client{}", i));
        println!("Created Client {}", i);
    }

    let socket = UdpSocket::bind(bind_addr).expect("Could not bind client socket");
    send_connect(&socket, broker.to_string(), "bench".to_string());

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

    send_sub(
        "topic1".to_string(),
        &packet_num, 
        &socket, 
        broker.to_string());

    socket
        .recv_from(&mut ack_buffer.as_mut())
        .expect("Could not read into buffer");



    let mut times = [0u64;100];
    let recv_time = Arc::new(Mutex::new(&mut times));

    let sock = socket.try_clone().expect("Failed to clone socket");
    let _t = thread::spawn( move || {
        recv_thread(&sock, &mut recv_time.clone());
    } );

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

    for i in 0..100 {
        let sock = socket.try_clone().expect("Failed to clone socket");

        send_pub(
            topic_id,
            "topic1:test".to_string(),
            i,
            &sock,
            broker.to_string()
        );

        if i % 10 == 0 {
            println!("Sent {} Messages", i);
        }
    }

    let sleep_t = time::Duration::from_millis(2000);
    thread::sleep(sleep_t);

    //println!("times {:?}", &mut times);
    //println!("AVG Round-trip time: {:.2?}",;
}
