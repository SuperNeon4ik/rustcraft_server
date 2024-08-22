use json::object;

use crate::network::packet::{ClientboundPacket, PacketWriter};

pub struct ConfigurationClientboundDisconnect {
    pub json_disconnect_reason: String
}

impl ConfigurationClientboundDisconnect {
    pub fn from_string(reason: String) -> Self {
        Self {
            json_disconnect_reason: object! {
                text: reason
            }.dump()
        }
    }
}

impl ClientboundPacket for ConfigurationClientboundDisconnect {
    fn packet_id() -> i32 {
        0x02
    }

    fn build(&self) -> Vec<u8> {
        PacketWriter::new(Self::packet_id())
            .write_string(&self.json_disconnect_reason)
            .build_uncompressed()
    }
}