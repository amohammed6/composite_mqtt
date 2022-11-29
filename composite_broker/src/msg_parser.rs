pub mod msg_parser {
    use bytes::{BytesMut};
    use mqtt_v5::{
        decoder, encoder,
        types::{ProtocolVersion},
    };

    #[allow(dead_code)]
    pub fn cm_encode(
        packet: mqtt_v5::types::Packet,
        buffer: &mut BytesMut,
    ) -> Result<&mut BytesMut, String> {
        encoder::encode_mqtt(&packet, buffer, ProtocolVersion::V500);
        if buffer.is_empty() {
            Err("Packet wasn't encoded".to_string())
        } else {
            Ok(buffer)
        }
    }

    // Decode function
    //  input: bytes of encoded packet
    //  output: packet
    pub fn cm_decode(buffer: & [u8]) -> Result<mqtt_v5::types::Packet, String> {
        let mut b = BytesMut::from(buffer);
        if buffer.is_empty() {
            Err("Buffer was empty".to_string())
        } else {
            Ok(decoder::decode_mqtt(&mut b, ProtocolVersion::V500)
                .unwrap()
                .unwrap())
        }
    }
}
