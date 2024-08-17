use crate::{network::packet::{ServerboundPacket, PacketReader}, utils::errors::PacketReadError};

pub struct LoginServerboundLoginAcknowledged {
}

impl ServerboundPacket for LoginServerboundLoginAcknowledged {
    fn packet_id() -> i32 
    where 
        Self: Sized {
        0x03
    }

    fn read(_reader: &mut PacketReader) -> Result<Self, PacketReadError>
    where 
        Self: Sized {
        Ok(Self {})
    }
}