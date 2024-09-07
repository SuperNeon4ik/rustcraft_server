use crate::{custom_types::identifier::Identifier, network::packet::{ClientboundPacket, PacketWriter}};

pub struct ConfigurationClientboundPluginMessage {
    pub channel: Identifier,
    pub data: Vec<u8>,
}

impl ClientboundPacket for ConfigurationClientboundPluginMessage {
    fn packet_id() -> i32 {
        0x01
    }

    fn build(&self) -> Vec<u8> {
        let mut packet = PacketWriter::new(Self::packet_id());
        packet.write_identifier(&self.channel);
        packet.write_byte_array(&self.data);
        packet.build_uncompressed()
    }
}