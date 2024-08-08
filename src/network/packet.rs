use core::str;
use bytes::{Buf, BufMut, BytesMut};

pub fn read_varint(buf: &mut dyn Buf) -> Option<i32> {
    let mut value = 0;
    let mut shift = 0;
    
    while shift < 35 {
        if buf.has_remaining() {
            let byte = buf.get_u8();
            value |= ((byte & 0x7F) as i32) << shift;
            if (byte & 0x80) == 0 {
                return Some(value);
            }
            shift += 7;
        } else {
            return None;
        }
    }
    
    None // VarInt is too long
}

pub fn write_varint(buf: &mut dyn BufMut, mut value: i32) {
    while value & !0x7F != 0 {
        buf.put_u8((value as u8 & 0x7F) | 0x80);
        value >>= 7;
    }
    buf.put_u8(value as u8 & 0x7F);
}

pub fn read_string(buf: &mut dyn Buf) -> Option<String> {
    let length = read_varint(buf).unwrap() as usize;

    if buf.remaining() < length {
        return None;
    }

    let mut string_bytes = vec![0u8; length];
    buf.copy_to_slice(&mut string_bytes);
    let result = str::from_utf8(&string_bytes).unwrap();
    
    Some(result.to_owned())
}

pub fn write_string(buf: &mut dyn BufMut, data: &str) {
    let bytes = data.as_bytes();
    let length = bytes.len();

    write_varint(buf, length as i32);
    buf.put_slice(bytes);
}

pub fn prepare_uncompressed_packet(buf: &mut BytesMut, packet_id: i32) -> Vec<u8> {
    let mut packet_id_buf = BytesMut::with_capacity(5);
    write_varint(&mut packet_id_buf, packet_id);

    let len = packet_id_buf.len() + buf.len();

    let mut length_buf = BytesMut::with_capacity(3);
    write_varint(&mut length_buf, len as i32);

    let mut final_buf = BytesMut::with_capacity(length_buf.len() + len);
    final_buf.put(length_buf);
    final_buf.put(packet_id_buf);
    final_buf.put(buf);

    final_buf.to_vec()
}