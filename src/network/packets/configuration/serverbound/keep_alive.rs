use crate::{network::packet::{ServerboundPacket, PacketReader}, utils::errors::PacketReadError};

pub struct ConfigurationServerboundKeepAlive {
    pub keep_alive_id: i64,
}

impl ServerboundPacket for ConfigurationServerboundKeepAlive {
    fn packet_id() -> i32 
    where 
        Self: Sized {
        0x04
    }

    fn read(reader: &mut PacketReader) -> Result<Self, PacketReadError>
    where 
        Self: Sized {
            Ok(Self {
                keep_alive_id: reader.read_long()?
            })
    }
}