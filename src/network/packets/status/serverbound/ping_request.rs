use crate::{network::packet::{PacketReader, ServerboundPacket}, utils::errors::PacketReadError};

pub struct StatusServerboundPingRequest {
    pub timestamp: i64
}

impl ServerboundPacket for StatusServerboundPingRequest {
    fn packet_id() -> i32 
    where 
        Self: Sized {
            0x01
    }

    fn read(reader: &mut PacketReader) -> Result<Self, PacketReadError>
    where 
        Self: Sized {
            Ok(StatusServerboundPingRequest {
                timestamp: reader.read_long()?
            })
    }
}