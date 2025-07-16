use crate::custom_types::known_pack::KnownPack;
use crate::network::packet::{PacketReader, ServerboundPacket};
use crate::utils::errors::PacketReadError;

pub struct ConfigurationServerboundKnownPacks {
    pub known_packs: Vec<KnownPack>,
}

impl ServerboundPacket for ConfigurationServerboundKnownPacks {
    fn packet_id() -> i32
    where
        Self: Sized
    {
        0x07
    }

    fn read(reader: &mut PacketReader) -> Result<Self, PacketReadError>
    where
        Self: Sized
    {
        let len = reader.read_varint()?;
        let mut known_packs = Vec::with_capacity(len as usize);
        for i in 0..len {
            known_packs.push(KnownPack {
                identifier: reader.read_string()?,
                id: reader.read_string()?,
                version: reader.read_string()?,
            });
        }

        Ok(Self {
            known_packs
        })
    }
}