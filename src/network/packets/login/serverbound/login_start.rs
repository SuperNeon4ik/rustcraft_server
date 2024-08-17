use uuid::Uuid;

use crate::{network::packet::ServerboundPacket, utils::errors::PacketReadError};

pub struct LoginServerboundLoginStart {
    pub name: String,
    pub uuid: Uuid,
}

impl ServerboundPacket for LoginServerboundLoginStart {
    fn packet_id() -> i32 
    where 
        Self: Sized {
        0x00
    }

    fn read(reader: &mut crate::network::packet::PacketReader) -> Result<Self, PacketReadError>
    where 
        Self: Sized {
        let name = reader.read_string()?;
        let uuid = reader.read_uuid()?;

        Ok(Self {
            name,
            uuid
        })
    }
}