use crate::network::packet::{ClientboundPacket, PacketWriter};

pub struct StatusClientboundPingResponse {
    pub timestamp: i64
}

impl ClientboundPacket for StatusClientboundPingResponse {
    fn packet_id() -> i32 {
        0x01
    }

    fn build(&self) -> Vec<u8> {
        PacketWriter::new(Self::packet_id())
            .write_long(self.timestamp)
            .build_uncompressed()
    }
}