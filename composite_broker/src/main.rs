mod broker;
use byte::BytesExt;
use std::sync::{Arc, Mutex};
use crate::broker::broker::{MBroker, Subscriptions};
use mqtt_sn::{self, Message};
use std::{
    io,
    net::{SocketAddr, UdpSocket},
    thread,
    time::Duration,
}; 

const NUM_THREADS: u16 = 0;

fn handle_packets(
    broker: &mut MBroker,
    socket: &UdpSocket,
    decode: Message,
    addr: SocketAddr,
    sub_list: Subscriptions,
) {
    let mut len = 0usize;
    match decode {
        Message::Connect(p) => {
            println!("Accepting client at {}", addr.to_string());
            let ret_p = broker.accept_connect(addr.to_string(), p); // returns ack
            // encode and write to buffer
            let mut ret_buf = [0u8; 128]; // will contain encoded bytes
            ret_buf
                .write(&mut len, ret_p)
                .expect("Didn't write to buffer"); // write to the buffer
            
            // send the ack packet to this client
            let send_buf = &mut ret_buf[..len];
            socket
                .send_to(send_buf, addr)
                .expect("Failed to send a response");
        }
        Message::Subscribe(p) => {
            let ret_p = broker.accept_sub(addr.to_string(), p, sub_list);
            // encode and write to buffer
            let mut ret_buf = [0u8; 128]; // will contain encoded bytes
            ret_buf
                .write(&mut len, ret_p)
                .expect("Didn't write to buffer"); // write to the buffer
            
            // send the ack packet to this client
            let send_buf = &mut ret_buf[..len];
            socket
                .send_to(send_buf, addr)
                .expect("Failed to send to client");
        }
        Message::Unsubscribe(p) => {
            let res = broker.accept_unsub(addr.to_string(), p, sub_list);
            
            // encode and write to buffer
            let mut ret_buf = [0u8; 128]; // will contain encoded bytes
            ret_buf.write(&mut 0, res).expect("Didn't write to buffer"); // write to the buffer
            
            // send the ack packet to this client
            socket
                .send_to(&ret_buf.as_mut(), addr)
                .expect("Failed to send to client");
        }
        _ => panic!("Incorrect type returned"),
    };
}

fn thread_fn(
    broker: &mut Arc<Mutex<MBroker>>,
    sub_list: Subscriptions,
    socket: &UdpSocket,
    mut buf: [u8; 128],
) {
    loop {
        match socket.recv_from(&mut buf) {
            // receive the message into buffer
            Ok((_, src)) => {
                //println!("Handling incoming from {}", src);

                let thread_broker = broker.clone();
                let thread_subs = sub_list.clone(); 
                let decode: Message = buf.read(&mut 0).unwrap(); // decode and pass it in

                match decode {
                    Message::Publish(p) => {
                        if let Ok(mut b) = thread_broker.lock() {
                            let res = b.accept_pub(src.to_string(), p, thread_subs);

                            // encode the ack packet and the publish packet again
                            let client_list = &res.2;
                            let (mut ack_buf, mut pub_buf) = ([0u8; 128], [0u8; 128]);

                            let mut len = 0usize;
                            pub_buf
                                .write(&mut len, res.1)
                                .expect("Didn't write to buffer");

                            /* QOS 0: we dont PUBACK */

                            // send the publish packet to the client list
                            let send_buf = &mut pub_buf[..len];
                            let mut i = 0usize;
                            while i < client_list.len() {
                                socket
                                    .send_to(send_buf, &client_list[i])
                                    .expect("Failed to send to subscriber");
                                i += 1;
                            }
                        }
                    }
                    _ => {
                        if let Ok(mut b) = thread_broker.lock() {
                            handle_packets(&mut b, &socket, decode, src, thread_subs);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Couldn't receive a datagram {}", e);
            }
        }
    }
}

fn main() ->  io::Result<()>{
    // make UDP socket
    let socket = UdpSocket::bind("0.0.0.0:8888").expect("Could not bind socket");
    let socket = Arc::new(socket);

    // allocating the dashmap
    let sub_list = Subscriptions::new();

    // make the broker
    let broker: MBroker = MBroker::new();
    let broker = Mutex::new(broker);
    let broker = Arc::new(broker);


    for _t in 0..NUM_THREADS {
        // intialize thread resources
        let mut thread_broker = broker.clone();
        let thread_subs = sub_list.clone(); // add another clone for the dashmap
        let buf = [0u8; 128];
        let sock = socket.try_clone().expect("Failed to clone socket"); // use socket clone to send to client

        // allow the thread to handle & send the packets
        let _t = thread::spawn( move || {
            thread_fn(&mut thread_broker, thread_subs, &sock, buf);
        } );

    }

    /* lets not waste this thread... */
    let mut thread_broker = broker.clone();
    let buf = [0u8; 128];
    thread_fn(&mut thread_broker, sub_list, &socket, buf);
    
    Ok(())
}

#[cfg(test)]

mod tests {
    use assert_hex::*;
    use core::panic;

    use crate::broker::broker::{MBroker, Subscriptions};
    use byte::BytesExt;
    use mqtt_sn::{
        ClientId, Connect, Flags, Message, Publish, PublishData, RejectedReason, ReturnCode,
        Subscribe, TopicName, Unsubscribe,
    }; 

    static ADDR: &str = "127.0.0.1:7878";
    static ADDR2: &str = "192.0.0.1:7777";

    #[test]
    fn test_read_publish_packet() {
        let mut bytes = [0u8; 20];
        let mut len = 0usize;
        let connect_packet = Message::Publish(Publish {
            flags: Flags::default(),
            topic_id: 30,
            msg_id: 01,
            data: PublishData::from("George"),
        });
        // encode
        bytes.write(&mut len, connect_packet.clone()).unwrap();
        // decode
        let decode: Message = bytes.read(&mut 0).unwrap();
        assert_eq!(connect_packet, decode);
        match decode {
            Message::Publish(m) => {
                assert_eq!(30, m.topic_id);
                println!("{:?}", m.topic_id);
            }
            _ => {}
        }
    }

    #[test]
    fn subscribe_encode_parse_id() {
        let mut bytes = [0u8; 20];
        let mut len = 0usize;
        let mut flags = Flags::default();
        flags.set_topic_id_type(0x2); // necessary when using topic_id
        let expected = Message::Subscribe(Subscribe {
            flags,
            msg_id: 0x1234,
            topic: mqtt_sn::TopicNameOrId::Id(0x5678),
        });
        bytes.write(&mut len, expected.clone()).unwrap();
        assert_eq_hex!(&bytes[..len], [0x07u8, 0x12, 0x02, 0x12, 0x34, 0x56, 0x78]);
        let actual: Message = bytes.read(&mut 0).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_read_subscribe_packet() {
        let mut bytes = [0u8; 20];
        let mut len = 0usize;
        let mut flags = Flags::default();
        flags.set_topic_id_type(0x2); // necessary when using topic_id
        let connect_packet = Message::Subscribe(Subscribe {
            flags,
            msg_id: 01,
            topic: mqtt_sn::TopicNameOrId::Id(30),
        });
        // encode
        bytes.write(&mut len, connect_packet.clone()).unwrap();
        // decode
        let decode: Message = bytes.read(&mut 0).unwrap();
        assert_eq!(connect_packet, decode);
        match decode {
            Message::Subscribe(m) => {
                // assert_eq!(mqtt_sn::TopicNameOrId::Id(30), m.topic);
                println!("{:?}", m.topic);
            }
            _ => {}
        }
    }

    #[test]
    fn test_read_connect_packet() {
        let mut bytes = [0u8; 20];
        let mut len = 0usize;
        let connect_packet = Message::Connect(Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from("hey"),
        });
        // connect_packet.try_write(&mut bytes, ()).expect("Couldn't write");
        bytes.write(&mut len, connect_packet.clone()).unwrap();
        let decode: Message = bytes.read(&mut 0).unwrap();
        assert_eq!(connect_packet, decode);
        match decode {
            Message::Connect(m) => {
                println!("{:?}", m.client_id)
            }
            _ => {}
        }
    }

    #[test]
    fn test_new_client1() {
        let mut broker = MBroker::new();
        let id = "1004";
        // Create a Connect packet
        let conn_p = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from(id),
        };

        let res = broker.accept_connect(ADDR.to_string(), conn_p);
        // broker.get_client_list();
        match res {
            Message::ConnAck(p) => {
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _ => {}
        }
    }

    #[test]
    fn test_new_client2() {
        let mut broker = MBroker::new();
        // Create two Connect packet
        let conn_p1 = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from("1004"),
        };

        let conn_p2 = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from("1005"),
        };

        let r1 = broker.accept_connect(ADDR.to_string(), conn_p1);
        let r2 = broker.accept_connect(ADDR.to_string(), conn_p2);
        // broker.get_client_list();
        match r1 {
            Message::ConnAck(p) => {
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _ => {}
        }

        match r2 {
            Message::ConnAck(p) => {
                assert_eq!(p.code, ReturnCode::Rejected(RejectedReason::Congestion));
            }
            _ => {}
        }
    }

    #[test]
    fn test_new_sub_id() {
        let sub_list = Subscriptions::new();
        let mut broker = MBroker::new();
        let id = "1004";
        let mut flags = Flags::default();
        flags.set_topic_id_type(0x2); // topic_id
                                      // Create a Connect packet
        let conn_p = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from(id),
        };

        broker.accept_connect(ADDR.to_string(), conn_p);

        // create a subscribe packet
        let sub_p = Subscribe {
            flags,
            msg_id: 01,
            topic: mqtt_sn::TopicNameOrId::Id(01),
        };

        let res = broker.accept_sub(ADDR.to_string(), sub_p, sub_list);
        // broker.get_sub_list();
        match res {
            Message::SubAck(p) => {
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _ => {
                panic!("Error: didn't receive ack packet");
            }
        }
    }

    #[test]
    fn test_new_sub_name() {
        let sub_list = Subscriptions::new();
        let mut broker = MBroker::new();
        let id = "1004";
        // Create a Connect packet
        let conn_p = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from(id),
        };

        broker.accept_connect(ADDR.to_string(), conn_p);

        // create a subscribe packet
        let sub_p = Subscribe {
            flags: Flags::default(),
            msg_id: 01,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("01")),
        };

        let res = broker.accept_sub(ADDR.to_string(), sub_p, sub_list);
        // broker.get_sub_list();
        match res {
            Message::SubAck(p) => {
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _ => {
                panic!("Error: didn't receive ack packet");
            }
        }
    }

    #[test]
    fn test_multiple_subs_names_ids() {
        let sub_list = Subscriptions::new();
        let mut broker = MBroker::new();
        let id = "1004";
        let mut flags = Flags::default();
        flags.set_topic_id_type(0x2); // topic_id
                                      // Create a Connect packet
        let conn_p = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from(id),
        };

        broker.accept_connect(ADDR.to_string(), conn_p);

        // create a subscribe packet
        let sub_p1 = Subscribe {
            flags: Flags::default(),
            msg_id: 01,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("01")),
        };
        let sub_p2 = Subscribe {
            flags,
            msg_id: 01,
            topic: mqtt_sn::TopicNameOrId::Id(01),
        };

        let res1 = broker.accept_sub(ADDR.to_string(), sub_p1, sub_list.clone());
        let res2 = broker.accept_sub(ADDR.to_string(), sub_p2, sub_list.clone());
        // broker.get_sub_list();
        match res1 {
            Message::SubAck(p) => {
                println!("Sub for \"01\" Topic id: {}", p.topic_id);
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _ => {
                panic!("Error: didn't receive ack packet");
            }
        }
        match res2 {
            Message::SubAck(p) => {
                println!("Sub for 01 Topic id: {}", p.topic_id);
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _ => {
                panic!("Error: didn't receive ack packet");
            }
        }
    }

    #[test]
    fn test_multiple_subs_gwu() {
        let sub_list = Subscriptions::new();
        let mut broker = MBroker::new();
        // connect
        let id = "1004";
        // Create a Connect packet
        let conn_p = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from(id),
        };

        broker.accept_connect(ADDR.to_string(), conn_p);

        // create subscribe packets
        let sub_p1 = Subscribe {
            flags: Flags::default(),
            msg_id: 01,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("seas")),
        };
        let sub_p2 = Subscribe {
            flags: Flags::default(),
            msg_id: 02,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("ccas")),
        };
        let sub_p3 = Subscribe {
            flags: Flags::default(),
            msg_id: 02,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("elliot")),
        };

        let r1 = broker.accept_sub(ADDR.to_string(), sub_p1, sub_list.clone());
        match r1 {
            Message::SubAck(p) => {
                println!("Sub for seas Topic id: {}", p.topic_id);
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _ => {
                panic!("Error: didn't receive ack packet");
            }
        }

        let r2 = broker.accept_sub(ADDR.to_string(), sub_p2, sub_list.clone());
        match r2 {
            Message::SubAck(p) => {
                println!("Sub for ccas Topic id: {}", p.topic_id);
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _ => {
                panic!("Error: didn't receive ack packet");
            }
        }

        let r3 = broker.accept_sub(ADDR.to_string(), sub_p3, sub_list.clone());
        match r3 {
            Message::SubAck(p) => {
                println!("Sub for elliot Topic id: {}", p.topic_id);
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _ => {
                panic!("Error: didn't receive ack packet");
            }
        }
        // broker.get_sub_list();
    }

    #[test]
    fn test_multiple_subs_unis() {
        let sub_list = Subscriptions::new();
        let mut broker = MBroker::new();
        // Create a Connect packet
        let conn_p = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from("1004"),
        };

        broker.accept_connect(ADDR.to_string(), conn_p);
        // create subscribe packets
        let sub_p1 = Subscribe {
            flags: Flags::default(),
            msg_id: 01,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("gwu")),
        };
        let sub_p2 = Subscribe {
            flags: Flags::default(),
            msg_id: 02,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("udel")),
        };
        let sub_p3 = Subscribe {
            flags: Flags::default(),
            msg_id: 03,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("uwm")),
        };

        let r1 = broker.accept_sub(ADDR.to_string(), sub_p1, sub_list.clone());
        match r1 {
            Message::SubAck(p) => {
                println!("Sub for gwu Topic id: {}", p.topic_id);
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _ => {
                panic!("Error: didn't receive ack packet");
            }
        }

        let r2 = broker.accept_sub(ADDR.to_string(), sub_p2, sub_list.clone());
        match r2 {
            Message::SubAck(p) => {
                println!("Sub for udel Topic id: {}", p.topic_id);
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _ => {
                panic!("Error: didn't receive ack packet");
            }
        }

        let r3 = broker.accept_sub(ADDR.to_string(), sub_p3, sub_list.clone());
        match r3 {
            Message::SubAck(p) => {
                println!("Sub for uwm Topic id: {}", p.topic_id);
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _ => {
                panic!("Error: didn't receive ack packet");
            }
        }
    }

    #[test]
    fn test_new_pub() {
        let sub_list = Subscriptions::new();
        let mut broker = MBroker::new();
        // Create a client's connect packet
        let conn_p = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from("1004"),
        };
        broker.accept_connect(ADDR.to_string(), conn_p);

        // create a subscribe packet
        let sub_p = Subscribe {
            flags: Flags::default(),
            msg_id: 01,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("gwu")),
        };
        let sub_ret = broker.accept_sub(ADDR.to_string(), sub_p, sub_list.clone());
        let topic = match sub_ret {
            Message::SubAck(p) => p.topic_id,
            _ => 0,
        };

        // create publish packet
        let pub_p = Publish {
            flags: Flags::default(),
            topic_id: topic,
            msg_id: 02,
            data: PublishData::from("George Washington"),
        };

        let res = broker.accept_pub(ADDR.to_string(), pub_p, sub_list.clone());
        match res.0 {
            Message::PubAck(p) => {
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _ => {
                panic!("Error: didn't receive ack packet");
            }
        }

        match res.1 {
            Message::Publish(p) => {
                println!("Published data: {:?}", p.data);
                assert_eq!(p.topic_id, topic);
            }
            _ => {
                panic!("Error didn't receive ")
            }
        }
        println!("{:?}", res.2); // two expected
    }

    #[test]
    fn test_new_pub_2clients() {
        let sub_list = Subscriptions::new();
        let mut broker = MBroker::new();
        // Create a client's connect packet
        let conn_p1 = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from("1004"),
        };
        broker.accept_connect(ADDR.to_string(), conn_p1);

        // Create a client's connect packet
        let conn_p2 = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from("1005"),
        };
        broker.accept_connect(ADDR2.to_string(), conn_p2);

        // create a subscribe packet
        let sub_p = Subscribe {
            flags: Flags::default(),
            msg_id: 01,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("gwu")),
        };
        let sub_ret = broker.accept_sub(ADDR.to_string(), sub_p, sub_list.clone());
        let topic = match sub_ret {
            Message::SubAck(p) => p.topic_id,
            _ => 0,
        };

        // create publish packet
        let pub_p = Publish {
            flags: Flags::default(),
            topic_id: topic,
            msg_id: 02,
            data: PublishData::from("George Washington University"),
        };

        let res = broker.accept_pub(ADDR2.to_string(), pub_p, sub_list.clone());
        match res.0 {
            Message::PubAck(p) => {
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _ => {
                panic!("Error: didn't receive ack packet");
            }
        }

        match res.1 {
            Message::Publish(p) => {
                println!("Published data: {:?}", p.data);
                assert_eq!(p.topic_id, topic);
            }
            _ => {
                panic!("Error: publish packet not returned")
            }
        }

        println!("{:?}", res.2); // two expected
    }

    #[test]
    fn test_unsub() {
        let sub_list = Subscriptions::new();
        let mut broker = MBroker::new();
        let mut flags = Flags::default();
        flags.set_topic_id_type(0x2); // topic_id
                                      // Create a client's connect packet
        let conn_p1 = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from("1004"),
        };
        broker.accept_connect(ADDR.to_string(), conn_p1);

        // subscribe to a topic
        println!("Subscribing...");
        let sub_p = Subscribe {
            flags: Flags::default(),
            msg_id: 01,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("gwu")),
        };
        let sub_ret = broker.accept_sub(ADDR.to_string(), sub_p, sub_list.clone());
        let topic = match sub_ret {
            Message::SubAck(p) => p.topic_id,
            _ => 0,
        };

        // unsubscribe to the topic
        // println!("Unsubscribing");
        let unsub_p = Unsubscribe {
            flags,
            msg_id: 02,
            topic: mqtt_sn::TopicNameOrId::Id(topic),
        };
        let unsub_ret = broker.accept_unsub(ADDR.to_string(), unsub_p, sub_list.clone());
        // broker.get_sub_list();
        match unsub_ret {
            Message::UnsubAck(p) => {
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _ => {
                panic!("Error: didn't receive ack packet");
            }
        }
    }

    #[test]
    fn test_unsub_invalid() {
        let sub_list = Subscriptions::new();
        let mut broker = MBroker::new();
        let mut flags = Flags::default();
        flags.set_topic_id_type(0x2); // topic_id
                                      // Create a client's connect packet
        let conn_p1 = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from("1004"),
        };
        broker.accept_connect(ADDR.to_string(), conn_p1);

        // subscribe to a topic
        let sub_p = Subscribe {
            flags: Flags::default(),
            msg_id: 01,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("gwu")),
        };
        let sub_ret = broker.accept_sub(ADDR.to_string(), sub_p, sub_list.clone());
        let topic = match sub_ret {
            Message::SubAck(p) => p.topic_id + 1,
            _ => 0,
        };

        // unsubscribe to the topic
        let unsub_p = Unsubscribe {
            flags,
            msg_id: 02,
            topic: mqtt_sn::TopicNameOrId::Id(topic),
        };
        let unsub_ret = broker.accept_unsub(ADDR.to_string(), unsub_p, sub_list.clone());
        match unsub_ret {
            Message::UnsubAck(p) => {
                assert_eq!(p.code, ReturnCode::Rejected(RejectedReason::InvalidTopicId));
            }
            _ => {
                panic!("Error: didn't receive ack packet");
            }
        }
    }
}
