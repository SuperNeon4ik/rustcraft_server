use crate::{custom_types::identifier::Identifier, network::packet::{PacketReader, ServerboundPacket}, utils::errors::PacketReadError};

pub struct ConfigurationServerboundPluginMessage {
    pub channel: Identifier,
    pub data: Vec<u8>
}

impl ServerboundPacket for ConfigurationServerboundPluginMessage {
    fn packet_id() -> i32 
    where 
        Self: Sized {
        0x04
    }

    fn read(reader: &mut PacketReader) -> Result<Self, PacketReadError>
    where 
        Self: Sized {
            let channel = reader.read_identifier()?;
            let remaining_bytes = reader.remaining();
            let data = reader.read_byte_array(remaining_bytes)?;
            Ok(Self {
                channel,
                data,   
            })
    }
}