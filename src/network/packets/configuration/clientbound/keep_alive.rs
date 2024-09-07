use crate::network::packet::{ClientboundPacket, PacketWriter};

pub struct ConfigurationClientboundKeepAlive {
    pub keep_alive_id: i64,
}

impl ClientboundPacket for ConfigurationClientboundKeepAlive {
    fn packet_id() -> i32 {
        0x04
    }

    fn build(&self) -> Vec<u8> {
        PacketWriter::new(Self::packet_id())
            .write_long(self.keep_alive_id)
            .build_uncompressed()
    }
}