use core::str;

use bytes::Buf;

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