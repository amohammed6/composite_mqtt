use serde::Deserialize;

use std::fs::File;
use std::io::{Read};

// Publisher message struct
#[derive(Deserialize, Debug)]
struct PubMsg {
    packet_id: u16,
    topic_name: String,
    qos: u16,
    payload: String,
    dup_flag: bool,
}

// Publisher message struct
#[derive(Deserialize, Debug)]
struct SubMsg {
    packet_id: u16,
    topic_name: String,
    qos: u16,
}

// input: JSON file
// output: prints (temp) => Box ds?
pub fn parse_pub() {
    let mut file = File::open("src/test_pub_msg.json").expect("File doesn't open");

    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("File contents not extracted");

    // let reader = BufReader::new(file);

    let p_msg: PubMsg = serde_json::from_str(&contents).unwrap();
    println!("Publishing packet {:?}, for {:?} of qos {:?}, the message is {:?} with flag{:?}", p_msg.packet_id, p_msg.topic_name, p_msg.qos, p_msg.payload, p_msg.dup_flag)
}

// input: JSON file
// output: prints (temp) => Box ds?
pub fn parse_sub() {
    let mut file = File::open("src/test_sub_msg.json").expect("File doesn't open");

    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("File contents not extracted");

    // let reader = BufReader::new(file);

    let s_msg: SubMsg = serde_json::from_str(&contents).unwrap();
    println!("Subscribing to {:?} of qos {:?} with packet # {:?}", s_msg.topic_name, s_msg.qos,  s_msg.packet_id,)
}