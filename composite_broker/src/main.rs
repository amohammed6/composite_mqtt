mod broker;
mod msg_parser;
use std::{io}; // , time  borrow::Borrow
use crate::broker::broker::MBroker;
use bytes::BytesMut;
use std ::net::{UdpSocket, SocketAddr};
// use std::io::{Read,Write};
// use std::thread;
use mqtt_sn::{self, Message}; // Register, Connect, ClientId, Flags
use byte::{BytesExt}; // check_len, TryRead, TryWrite

fn handle_packets (broker: &mut MBroker, socket: &UdpSocket, buffer: &[u8;255], addr: SocketAddr) -> MBroker {
    println!("\tPacket being handled");
    if buffer.is_empty() {
        return broker.clone();
    }
    let decode : Message = buffer.read(&mut 0).unwrap();
    match decode {
        Message::Connect(p) => {
            println!("\tConnecting...");
            let ret_p = broker.accept_connect(addr.to_string(), p);  // returns ack
            let mut ret_buf = BytesMut::new();      // will contain encoded bytes
            ret_buf.write(&mut 0, ret_p).expect("Didn't write to buffer");// write to the buffer
            socket.send_to(&ret_buf.as_mut(), addr).expect("Failed to send a response");
        },
        Message::Subscribe(p) => {
            println!("\tSubscribing...");
            let ret_p = broker.accept_sub(addr.to_string(), p);
            let mut ret_buf = BytesMut::new();      // will contain encoded bytes
            ret_buf.write(&mut 0, ret_p).expect("Didn't write to buffer"); // write to the buffer    
            socket.send_to(&ret_buf.as_mut(), addr).expect("Failed to send to client");
        },
        Message::Publish(p) => {
            println!("\tPublishing...");
                let res = broker.accept_pub(addr.to_string(), p);
                // encode the ack packet and the publish packet again
                let client_list = &res.2;
                let (mut ack_buf, mut pub_buf) = (BytesMut::new(), BytesMut::new());
                ack_buf.write(&mut 0, res.0).expect("Didn't write to buffer"); 
                pub_buf.write(&mut 0, res.1).expect("Didn't write to buffer");
                // send the ack packet to this client
                socket.send_to(&ack_buf, addr).expect("Failed to send to client");
                for cli in client_list {
                    println!("Sending to client...{}", cli);
                    socket.send_to(&pub_buf, cli).expect("Failed to send to subscriber");
                }
        },
        _ => panic!("Incorrect type returned"),
    };
    broker.clone()
}

fn main() -> io::Result<()>{

    // make UDP socket
    let socket = UdpSocket::bind("0.0.0.0:8888").expect("Could not bind socket");
    // make the broker 
    let mut broker = MBroker::new();

    loop {
        let mut buf = [0u8; 255];
        let sock = socket.try_clone().expect("Failed to clone socket");    // use socket clone to send to client

        match socket.recv_from(&mut buf) {  // receive the message into buffer
            Ok((_, src)) => {
                if String::from_utf8_lossy(&buf).starts_with("mqtt") {
                    println!("Handling incoming from {}", src);
                }
                else {
                    println!("Receiving packet from {}", src);
                    broker = handle_packets(&mut broker.clone(), &sock, &buf, src);// .unwrap_or_else(|error| eprintln!("{:?}",error))
                }
            },
            Err(e) => {
                eprintln!("Couldn't receive a datagram {}", e);
            }
        }
    }

}


#[cfg(test)]

mod tests {
    use core::panic;

    use crate::broker::broker::MBroker;
    use mqtt_sn::{Connect, Flags, ClientId, Message, ReturnCode, RejectedReason, Subscribe, TopicName, Publish, PublishData};
    use byte::{BytesExt}; // TryWrite
    
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
            data: PublishData::from("George") 
        });
        // connect_packet.try_write(&mut bytes, ()).expect("Couldn't write");
        bytes.write(&mut len, connect_packet.clone()).unwrap();
        let decode : Message =bytes.read(&mut 0).unwrap();
        assert_eq!(connect_packet, decode);
        match decode {
            Message::Connect(m) => {
                println!("{:?}", m.client_id)
            }
            _ =>{}
        }
    }

    #[test]
    fn test_read_connect_packet() {
        let mut bytes = [0u8; 20];
        let mut len = 0usize;
        let connect_packet = Message::Connect(Connect{
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from("hey")
        });
        // connect_packet.try_write(&mut bytes, ()).expect("Couldn't write");
        bytes.write(&mut len, connect_packet.clone()).unwrap();
        let decode : Message =bytes.read(&mut 0).unwrap();
        assert_eq!(connect_packet, decode);
        match decode {
            Message::Connect(m) => {
                println!("{:?}", m.client_id)
            }
            _ =>{}
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
            client_id: ClientId::from(id)
        };
    
        let res = broker.accept_connect(ADDR.to_string(), conn_p);
        // broker.get_client_list();
        match res {
            Message::ConnAck(p) => {
                assert_eq!(p.code, ReturnCode::Accepted);
            },
            _=> {}
        }
    }

    
    #[test]
    fn test_new_client2() {
        let mut broker = MBroker::new();
        // Create two Connect packet
        let conn_p1 = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from("1004")
        };

        let conn_p2 = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from("1005")
        };

        let r1 = broker.accept_connect(ADDR.to_string(), conn_p1);
        let r2 = broker.accept_connect(ADDR.to_string(), conn_p2);
        // broker.get_client_list();
        match r1 {
            Message::ConnAck(p) => {
                assert_eq!(p.code, ReturnCode::Accepted);
            },
            _=> {}
        }

        match r2 {
            Message::ConnAck(p) => {
                assert_eq!(p.code, ReturnCode::Rejected(RejectedReason::Congestion));
            },
            _=> {}
        }
    }

    
    #[test]
    fn test_new_sub_id() {
        let mut broker = MBroker::new();
        let id = "1004";
        // Create a Connect packet
        let conn_p = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from(id)
        };
    
        broker.accept_connect(ADDR.to_string(), conn_p);
        
        // create a subscribe packet
        let sub_p = Subscribe {
            flags: Flags::default(),
            msg_id: 01,
            topic: mqtt_sn::TopicNameOrId::Id(01)
        };

        let res = broker.accept_sub(ADDR.to_string(), sub_p);
        // broker.get_sub_list();
        match res {
            Message::SubAck(p) => {
                assert_eq!(p.code, ReturnCode::Accepted);
            },
            _ => {
                panic!("Error: didn't receive ack packet");
            } 
        }
        
    }

    #[test]
    fn test_new_sub_name() {
        let mut broker = MBroker::new();
        let id = "1004";
        // Create a Connect packet
        let conn_p = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from(id)
        };
    
        broker.accept_connect(ADDR.to_string(), conn_p);
        
        // create a subscribe packet
        let sub_p = Subscribe {
            flags: Flags::default(),
            msg_id: 01,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("01"))
        };

        let res = broker.accept_sub(ADDR.to_string(), sub_p);
        // broker.get_sub_list();
        match res {
            Message::SubAck(p) => {
                assert_eq!(p.code, ReturnCode::Accepted);
            },
            _ => {
                panic!("Error: didn't receive ack packet");
            } 
        }
        
    }

    #[test]
    fn test_multiple_subs_names_ids() {
        let mut broker = MBroker::new();
        let id = "1004";
        // Create a Connect packet
        let conn_p = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from(id)
        };
    
        broker.accept_connect(ADDR.to_string(), conn_p);
        
        // create a subscribe packet
        let sub_p1 = Subscribe {
            flags: Flags::default(),
            msg_id: 01,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("01"))
        };
        let sub_p2 = Subscribe {
            flags: Flags::default(),
            msg_id: 01,
            topic: mqtt_sn::TopicNameOrId::Id(01)
        };

        let res1 = broker.accept_sub(ADDR.to_string(), sub_p1);
        let res2 = broker.accept_sub(ADDR.to_string(), sub_p2);
        // broker.get_sub_list();
        match res1 {
            Message::SubAck(p) => {
                println!("Sub for \"01\" Topic id: {}", p.topic_id);
                assert_eq!(p.code, ReturnCode::Accepted);
            },
            _ => {
                panic!("Error: didn't receive ack packet");
            } 
        }
        match res2 {
            Message::SubAck(p) => {
                println!("Sub for 01 Topic id: {}", p.topic_id);
                assert_eq!(p.code, ReturnCode::Accepted);
            },
            _ => {
                panic!("Error: didn't receive ack packet");
            } 
        }
        
    }

    #[test]
    fn test_multiple_subs_gwu() {
        let mut broker = MBroker::new();
        // connect
        let id = "1004";
        // Create a Connect packet
        let conn_p = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from(id)
        };
    
        broker.accept_connect(ADDR.to_string(), conn_p);

        // create subscribe packets
        let sub_p1 = Subscribe {
            flags: Flags::default(),
            msg_id: 01,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("seas"))
        };
        let sub_p2 = Subscribe {
            flags: Flags::default(),
            msg_id: 02,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("ccas"))
        };
        let sub_p3 = Subscribe {
            flags: Flags::default(),
            msg_id: 02,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("elliot"))
        };

        let r1 = broker.accept_sub(ADDR.to_string(), sub_p1);
        match r1 {
            Message::SubAck(p) => {
                println!("Sub for seas Topic id: {}", p.topic_id);
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _=> {
                panic!("Error: didn't receive ack packet");
            }
        }
        

        let r2 = broker.accept_sub(ADDR.to_string(), sub_p2);
        match r2 {
            Message::SubAck(p) => {
                println!("Sub for ccas Topic id: {}", p.topic_id);
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _=> {
                panic!("Error: didn't receive ack packet");
            }
        }
        

        let r3 = broker.accept_sub(ADDR.to_string(), sub_p3);
        match r3 {
            Message::SubAck(p) => {
                println!("Sub for elliot Topic id: {}", p.topic_id);
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _=> {
                panic!("Error: didn't receive ack packet");
            }
        }
        // broker.get_sub_list(); 
    }
  
    #[test]
    fn test_multiple_subs_unis() {
        let mut broker = MBroker::new();
        // Create a Connect packet
        let conn_p = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from("1004")
        };
    
        broker.accept_connect(ADDR.to_string(), conn_p);
        // create subscribe packets
        let sub_p1 = Subscribe {
            flags: Flags::default(),
            msg_id: 01,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("gwu"))
        };
        let sub_p2 = Subscribe {
            flags: Flags::default(),
            msg_id: 02,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("udel"))
        };
        let sub_p3 = Subscribe {
            flags: Flags::default(),
            msg_id: 03,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("uwm"))
        };

        let r1 = broker.accept_sub(ADDR.to_string(), sub_p1);
        match r1 {
            Message::SubAck(p) => {
                println!("Sub for gwu Topic id: {}", p.topic_id);
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _=> {
                panic!("Error: didn't receive ack packet");
            }
        }
        

        let r2 = broker.accept_sub(ADDR.to_string(), sub_p2);
        match r2 {
            Message::SubAck(p) => {
                println!("Sub for udel Topic id: {}", p.topic_id);
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _=> {
                panic!("Error: didn't receive ack packet");
            }
        }
        

        let r3 = broker.accept_sub(ADDR.to_string(), sub_p3);
        match r3 {
            Message::SubAck(p) => {
                println!("Sub for uwm Topic id: {}", p.topic_id);
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _=> {
                panic!("Error: didn't receive ack packet");
            }
        }
        
    }
  
    #[test]
    fn test_new_pub() {
        let mut broker = MBroker::new();
        // Create a client's connect packet
        let conn_p = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from("1004")
        };
        broker.accept_connect(ADDR.to_string(), conn_p);

        // create a subscribe packet
        let sub_p = Subscribe {
            flags: Flags::default(),
            msg_id: 01,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("gwu"))
        };
        let sub_ret = broker.accept_sub(ADDR.to_string(), sub_p);
        let topic = match sub_ret {
            Message::SubAck(p) => p.topic_id,
            _ => 0
        };

        // create publish packet
        let pub_p = Publish {
            flags: Flags::default(),
            topic_id: topic,
            msg_id: 02,
            data: PublishData::from("George Washington")
        };

        let res = broker.accept_pub(ADDR.to_string(), pub_p);
        match res.0 {
            Message::PubAck(p) => {
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _=> {
                panic!("Error: didn't receive ack packet");
            }
        }
        
        match res.1 {
            Message::Publish(p) => {
                println!("Published data: {:?}", p.data);
                assert_eq!(p.topic_id, topic);
            }
            _=> {
                panic!("Error didn't receive ")
            }
        }
        println!("{:?}", res.2);  // two expected
    }

    #[test]
    fn test_new_pub_2clients() {
        let mut broker = MBroker::new();
        // Create a client's connect packet
        let conn_p1 = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from("1004")
        };
        broker.accept_connect(ADDR.to_string(), conn_p1);

        // Create a client's connect packet
        let conn_p2 = Connect {
            flags: Flags::default(),
            duration: 30,
            client_id: ClientId::from("1005")
        };
        broker.accept_connect(ADDR2.to_string(), conn_p2);

        // create a subscribe packet
        let sub_p = Subscribe {
            flags: Flags::default(),
            msg_id: 01,
            topic: mqtt_sn::TopicNameOrId::Name(TopicName::from("gwu"))
        };
        let sub_ret = broker.accept_sub(ADDR.to_string(), sub_p);
        let topic = match sub_ret {
            Message::SubAck(p) => p.topic_id,
            _ => 0
        };

        // create publish packet
        let pub_p = Publish {
            flags: Flags::default(),
            topic_id: topic,
            msg_id: 02,
            data: PublishData::from("George Washington University")
        };


        let res = broker.accept_pub(ADDR2.to_string(), pub_p);
        match res.0 {
            Message::PubAck(p) => {
                assert_eq!(p.code, ReturnCode::Accepted);
            }
            _=> {
                panic!("Error: didn't receive ack packet");
            }
        }

        match res.1 {
            Message::Publish(p) => {
                println!("Published data: {:?}", p.data);
                assert_eq!(p.topic_id, topic);
            }
            _=> {
                panic!("Error: publish packet not returned")
            }
        }
        
        println!("{:?}", res.2);  // two expected
    }
}