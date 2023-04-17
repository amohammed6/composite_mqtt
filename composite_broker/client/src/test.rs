#[cfg(test)]
mod test {
    use std::net::UdpSocket; 
    use byte::BytesExt;
    use mqtt_sn::{Message};
    // use std::time::{Duration}; // Instant
    use chrono::{prelude::*}; // DurationRound

    const NUM_CLIENTS: usize = 64;

    #[test]
    fn some_test() { 
        let addr: String = "127.0.0.1".to_string();          // address doesn't change
        let mut port = 7777;                            // port increases with each client
        let mut clients: Vec<UdpSocket> =Vec::with_capacity(NUM_CLIENTS);
        // let mut sock: UdpSocket;

        // make num_clients number of ports
        for _i in 0..NUM_CLIENTS {
            let sock = UdpSocket::bind(addr.clone()+":"+&port.to_string()).expect("Couldn't bind client socket");
            clients.push(sock);
            port+=1;
        }

        // connect them all
        for socket in &clients {
            crate::send_connect(&socket, addr.clone()+":8888");
            // receive the ack packet
            let mut buf = [0u8; 1500];
            socket.recv_from(buf.as_mut()).expect("Could not read into buffer");
            let p: Message = buf.read(&mut 0).unwrap();
            match p {
                Message::ConnAck(_m) => {
                    // println!("Connect Ack received {:?}", m.code)
                }
                _ => {panic!("ConnAck not received for {:?}", socket.local_addr())}
            }
        }

        // create a topic and subscribe to it
        for socket in &clients {
            // create a topic and subscribe to it
            crate::send_sub("sub 1\n".to_string(), &300, &socket, addr.clone()+":8888");
            // receive the ack packet
            let mut buf = [0u8; 1500];
            socket.recv_from(buf.as_mut()).expect("Could not read into buffer");
            let p: Message = buf.read(&mut 0).unwrap();
            match p {
                Message::SubAck(_m) => {
                    // println!("Subscribe Ack received {:?}", m.code)
                }
                _ => {panic!("Subscribe Ack not received for {:?}", socket.local_addr())}
            }
        }

        // publish to that topic
        for socket in &clients {
            // publish the time to the topic
            let now = Local::now();       // get time it's sent
            crate::send_pub("pub 1:".to_owned()+&now.to_rfc3339(), &300, &socket, addr.clone()+":8888");
            // receive the ack packet
            let mut buf = [0u8; 1500];
            socket.recv_from(buf.as_mut()).expect("Could not read into buffer");
            let p: Message = buf.read(&mut 0).unwrap();
            match p {
                Message::PubAck(_m) => {
                    // println!("Subscribe Ack received {:?}", m.code)
                }
                Message::Publish(_m) => {
                    let new_now = Local::now().signed_duration_since(now);
                    println!("{:?}", new_now.num_nanoseconds().unwrap());
                }
                _ => {panic!("Subscribe Ack not received for {:?}", socket.local_addr())}
            }
        }
        
    }
}
