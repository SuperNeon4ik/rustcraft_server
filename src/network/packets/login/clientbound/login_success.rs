use uuid::Uuid;

use crate::network::packet::{ClientboundPacket, PacketWriter};

pub struct LoginClientboundLoginSuccess {
    uuid: Uuid,
    username: String,
    properties: Vec<LoginSuccessProperty>,
    strict_error_handling: bool,
}

pub struct LoginSuccessProperty {
    name: String,
    value: String,
    signature: Option<String>,
}

impl ClientboundPacket for LoginClientboundLoginSuccess {
    fn packet_id() -> i32 {
        0x02
    }

    fn build(&self) -> Vec<u8> {
        let mut writer = PacketWriter::new(Self::packet_id());
        writer.write_uuid(self.uuid);
        writer.write_string(&self.username);

        writer.write_varint(self.properties.len() as i32);
        for property in &self.properties {
            writer.write_string(&property.name);
            writer.write_string(&property.value);

            match &property.signature {
                Some(s) => {
                    writer.write_boolean(true);
                    writer.write_string(s);
                }
                None => { 
                    writer.write_boolean(false);
                }
            };
        }

        writer.write_boolean(self.strict_error_handling);

        writer.build_uncompressed()
    }
}