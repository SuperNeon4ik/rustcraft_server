use crate::network::packet::{ClientboundPacket, PacketWriter};

pub struct ConfigurationClientboundFinishConfiguration {
}

impl ClientboundPacket for ConfigurationClientboundFinishConfiguration {
    fn packet_id() -> i32 {
        0x03
    }

    fn build(&self) -> Vec<u8> {
        PacketWriter::new(Self::packet_id()).build_uncompressed()
    }
}