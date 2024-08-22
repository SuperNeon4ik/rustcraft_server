use crate::{custom_types::identifier::Identifier, network::packet::{ClientboundPacket, PacketReader, PacketWriter, ServerboundPacket}, utils::errors::PacketReadError};

pub struct ConfigurationClientboundFeatureFlags {
    pub feature_flags: Vec<Identifier>,
}

impl ClientboundPacket for ConfigurationClientboundFeatureFlags {
    fn packet_id() -> i32 {
        0x0C
    }

    fn build(&self) -> Vec<u8> {
        let mut writer = PacketWriter::new(Self::packet_id());
        writer.write_varint(self.feature_flags.len() as i32);

        for feature_flag in &self.feature_flags {
            writer.write_identifier(feature_flag);
        }
        
        writer.build_uncompressed()
    }
}