use json::object;

use crate::network::packet::{ClientboundPacket, PacketWriter};

pub struct LoginClientboundDisconnect {
    pub json_disconnect_reason: String
}

impl LoginClientboundDisconnect {
    pub fn from_string(reason: String) -> Self {
        Self {
            json_disconnect_reason: object! {
                text: reason
            }.dump()
        }
    }
}

impl ClientboundPacket for LoginClientboundDisconnect {
    fn packet_id() -> i32 {
        0x00
    }

    fn build(&self) -> Vec<u8> {
        PacketWriter::new(Self::packet_id())
            .write_string(&self.json_disconnect_reason)
            .build_uncompressed()
    }
}