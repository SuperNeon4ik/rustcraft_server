use crate::custom_types::known_pack::KnownPack;
use crate::network::packet::{ClientboundPacket, PacketWriter};

pub struct ConfigurationClientboundKnownPacks {
    pub known_packs: Vec<KnownPack>,
}

impl ClientboundPacket for ConfigurationClientboundKnownPacks {
    fn packet_id() -> i32 {
        0x0E
    }

    fn build(&self) -> Vec<u8> {
        let mut writer = PacketWriter::new(Self::packet_id());
        writer.write_varint(self.known_packs.len() as i32);

        for known_pack in &self.known_packs {
            writer.write_string(&known_pack.identifier);
            writer.write_string(&known_pack.id);
            writer.write_string(&known_pack.version);
        }

        writer.build_uncompressed()
    }
}