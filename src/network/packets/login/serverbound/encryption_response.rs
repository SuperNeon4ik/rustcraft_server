use crate::{network::packet::{ServerboundPacket, PacketReader}, utils::errors::PacketReadError};

pub struct LoginServerboundEncryptionResponse {
    pub shared_secret: Vec<u8>,
    pub verify_token: Vec<u8>,
}

impl ServerboundPacket for LoginServerboundEncryptionResponse {
    fn packet_id() -> i32 
    where 
        Self: Sized {
        0x01
    }

    fn read(reader: &mut PacketReader) -> Result<Self, PacketReadError>
    where 
        Self: Sized {
        let shared_secret_length = reader.read_varint()? as usize;
        let shared_secret = reader.read_byte_array(shared_secret_length)?;
        let verify_token_length = reader.read_varint()? as usize;
        let verify_token = reader.read_byte_array(verify_token_length)?;

        Ok(Self {
            shared_secret,
            verify_token,
        })
    }
}