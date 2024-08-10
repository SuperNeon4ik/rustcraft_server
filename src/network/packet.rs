use bytes::{BufMut, BytesMut};
use super::packet_utils::{write_string, write_varint};

pub struct PacketWriter {
    packet_id: i32,
    data: BytesMut,
}

impl PacketWriter {
    pub fn new(packet_id: i32) -> Self {
        PacketWriter {
            packet_id,
            data: BytesMut::new()
        }
    }

    pub fn write_varint(mut self, n: i32) -> Self {
        write_varint(&mut self.data, n);
        self
    }

    pub fn write_string(mut self, text: &str) -> Self {
        write_string(&mut self.data, text);
        self
    }

    pub fn write_boolean(mut self, val: bool) -> Self {
        if val { self.data.put_u8(0x01); }
        else { self.data.put_u8(0x00); }
        self
    }

    pub fn write_byte(mut self, n: i8) -> Self {
        self.data.put_i8(n);
        self
    }

    pub fn write_ubyte(mut self, n: u8) -> Self {
        self.data.put_u8(n);
        self
    }

    pub fn write_short(mut self, n: i16) -> Self {
        self.data.put_i16_le(n);
        self
    }

    pub fn write_ushort(mut self, n: u16) -> Self {
        self.data.put_u16_le(n);
        self
    }

    pub fn write_int(mut self, n: i32) -> Self {
        self.data.put_i32_le(n);
        self
    }

    pub fn write_long(mut self, n: i64) -> Self {
        self.data.put_i64_le(n);
        self
    }

    pub fn write_float(mut self, val: f32) -> Self {
        self.data.put_f32_le(val);
        self
    }

    pub fn write_double(mut self, val: f64) -> Self {
        self.data.put_f64_le(val);
        self
    }

    pub fn build_uncompressed(self) -> Vec<u8> {
        let mut packet_buf = BytesMut::with_capacity(7);
        write_varint(&mut packet_buf, self.packet_id);
        packet_buf.put(self.data);
    
        let mut final_buf = BytesMut::new();
        write_varint(&mut final_buf, packet_buf.len() as i32);
        final_buf.put(packet_buf);
    
        final_buf.to_vec()
    }
}