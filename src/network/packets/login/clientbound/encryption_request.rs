use crate::network::packet::{ClientboundPacket, PacketWriter};

pub struct LoginClientboundEncryptionRequest {
    pub public_key: Vec<u8>,
    pub verify_token: Vec<u8>,
    pub should_authenticate: bool,
}

impl ClientboundPacket for LoginClientboundEncryptionRequest {
    fn packet_id() -> i32 {
        0x01
    }

    fn build(&self) -> Vec<u8> {
        let mut writer = PacketWriter::new(Self::packet_id());
        writer.write_string(""); // Server ID (from wiki.vg): Appears to be empty
        writer.write_varint(self.public_key.len() as i32);
        writer.write_byte_array(&self.public_key);
        writer.write_varint(self.verify_token.len() as i32);
        writer.write_byte_array(&self.verify_token);
        writer.write_boolean(self.should_authenticate);
        writer.build_uncompressed()
    }
}