use crate::network::packet::{ClientboundPacket, PacketWriter};

pub struct StatusClientboundStatusResponse {
    pub json_response: String
}

impl ClientboundPacket for StatusClientboundStatusResponse {
    fn packet_id() -> i32 {
        0x00
    }

    fn build(&self) -> Vec<u8> {
        PacketWriter::new(Self::packet_id())
            .write_string(&self.json_response)
            .build_uncompressed()
    }
}