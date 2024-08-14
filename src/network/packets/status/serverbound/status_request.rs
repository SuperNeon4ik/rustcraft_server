use crate::{network::packet::{PacketReader, ServerboundPacket}, utils::errors::PacketReadError};

pub struct StatusServerboundStatusRequest {
}

impl ServerboundPacket for StatusServerboundStatusRequest {
    fn packet_id() -> i32 
    where 
        Self: Sized {
            0x00
    }

    fn read(_reader: &mut PacketReader) -> Result<Self, PacketReadError>
    where 
        Self: Sized {
            Ok(StatusServerboundStatusRequest {})
    }
}