use core::str;
use bytes::{Buf, BufMut};

use crate::utils::errors::PacketReadError;

pub fn read_varint(buf: &mut dyn Buf) -> Result<i32, PacketReadError> {
    let mut value = 0;
    let mut shift = 0;
    
    while shift < 32 {
        if buf.has_remaining() {
            let byte = buf.get_u8();
            value |= ((byte & 0x7F) as i32) << shift;
            if (byte & 0x80) == 0 {
                return Ok(value);
            }
            shift += 7;
        } else {
            return Err(PacketReadError::EmptyBuf);
        }
    }
    
    Err(PacketReadError::TooLong)
}

pub fn write_varint(buf: &mut dyn BufMut, value: i32) {
    let mut value = value;
    loop {
        if (value & !0x7F) == 0 {
            buf.put_u8(value as u8);
            return;
        }

        buf.put_u8(((value & 0x7F) | 0x80) as u8);
        value = ((value as u32) >> 7) as i32;
    }
}

#[allow(unused)]
pub fn read_varlong(buf: &mut dyn Buf) -> Result<i64, PacketReadError> {
    let mut value = 0;
    let mut shift = 0;
    
    while shift < 64 {
        if buf.has_remaining() {
            let byte = buf.get_u8();
            value |= ((byte & 0x7F) as i64) << shift;
            if (byte & 0x80) == 0 {
                return Ok(value);
            }
            shift += 7;
        } else {
            return Err(PacketReadError::EmptyBuf);
        }
    }
    
    Err(PacketReadError::TooLong)
}

#[allow(unused)]
pub fn write_varlong(buf: &mut dyn BufMut, value: i64) {
    let mut value = value;
    loop {
        if (value & !0x7F) == 0 {
            buf.put_u8(value as u8);
            return;
        }

        buf.put_u8(((value & 0x7F) | 0x80) as u8);
        value = ((value as u64) >> 7) as i64;
    }
}

pub fn read_string(buf: &mut dyn Buf) -> Result<String, PacketReadError> {
    let length = read_varint(buf).unwrap() as usize;

    if buf.remaining() < length {
        return Err(PacketReadError::BufferUnderflow);
    }

    let mut string_bytes = vec![0u8; length];
    buf.copy_to_slice(&mut string_bytes);
    return match str::from_utf8(&string_bytes) {
        Ok(result) => Ok(result.to_owned()),
        Err(_) => Err(PacketReadError::Utf8Error)
    }
}

pub fn write_string(buf: &mut dyn BufMut, data: &str) {
    let bytes = data.as_bytes();
    let length = bytes.len() as i32;

    write_varint(buf, length);
    buf.put_slice(bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_read_varint_single_byte() {
        let mut buf = BytesMut::from(&[0x01][..]);
        assert_eq!(read_varint(&mut buf).unwrap(), 1);
    }

    #[test]
    fn test_read_varint_multi_byte() {
        let mut buf = BytesMut::from(&[0xAC, 0x02][..]);
        assert_eq!(read_varint(&mut buf).unwrap(), 300);
    }

    #[test]
    fn test_read_varint_many_bytes_1() {
        let mut buf = BytesMut::from(&[0x83, 0x08][..]);
        assert_eq!(read_varint(&mut buf).unwrap(), 1027);
    }

    #[test]
    fn test_read_varint_many_bytes_2() {
        let mut buf = BytesMut::from(&[0xb9, 0x60][..]);
        assert_eq!(read_varint(&mut buf).unwrap(), 12345);
    }

    #[test]
    fn test_read_varint_negative() {
        let mut buf = BytesMut::from(&[0xff, 0xff, 0xff, 0xff, 0x0f][..]);
        assert_eq!(read_varint(&mut buf).unwrap(), -1);
    }

    #[test]
    fn test_read_varint_max_value() {
        let mut buf = BytesMut::from(&[0xFF, 0xFF, 0xFF, 0xFF, 0x07][..]);
        assert_eq!(read_varint(&mut buf).unwrap(), 2147483647); // Maximum value for i32
    }

    #[test]
    fn test_read_varint_min_value() {
        let mut buf = BytesMut::from(&[0x80, 0x80, 0x80, 0x80, 0x08][..]);
        assert_eq!(read_varint(&mut buf).unwrap(), -2147483648); // Minimum value for i32
    }

    #[test]
    #[should_panic(expected = "EmptyBuf")]
    fn test_read_varint_incomplete_data() {
        let mut buf = BytesMut::from(&[0xFF, 0xFF][..]);
        read_varint(&mut buf).unwrap();
    }

    #[test]
    #[should_panic(expected = "TooLong")]
    fn test_read_varint_too_long() {
        let mut buf = BytesMut::from(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x02][..]);
        read_varint(&mut buf).unwrap();
    }

    #[test]
    #[should_panic(expected = "EmptyBuf")]
    fn test_read_varint_empty_buffer() {
        let mut buf = BytesMut::new();
        read_varint(&mut buf).unwrap();
    }

    #[test]
    fn test_read_varint_zero_value() {
        let mut buf = BytesMut::from(&[0x00][..]);
        assert_eq!(read_varint(&mut buf).unwrap(), 0);
    }

    #[test]
    fn test_multi_read_varint() -> Result<(), PacketReadError> {
        let mut buf = BytesMut::from(&[0x83, 0x08, 0xb9, 0x60][..]);
        assert_eq!(read_varint(&mut buf)?, 1027);
        assert_eq!(read_varint(&mut buf)?, 12345);
        Ok(())
    }

    #[test]
    fn test_write_varint_single_byte() {
        let mut buf = BytesMut::new();
        write_varint(&mut buf, 1);
        assert_eq!(buf, BytesMut::from(&[0x01][..]));
    }

    #[test]
    fn test_write_varint_multi_byte() {
        let mut buf = BytesMut::new();
        write_varint(&mut buf, 300);
        assert_eq!(buf, BytesMut::from(&[0xAC, 0x02][..]));
    }

    #[test]
    fn test_write_varint_many_bytes_1() {
        let mut buf = BytesMut::new();
        write_varint(&mut buf, 1027);
        assert_eq!(buf, BytesMut::from(&[0x83, 0x08][..]));
    }

    #[test]
    fn test_write_varint_many_bytes_2() {
        let mut buf = BytesMut::new();
        write_varint(&mut buf, 12345);
        assert_eq!(buf, BytesMut::from(&[0xb9, 0x60][..]));
    }

    #[test]
    fn test_write_varint_max_value() {
        let mut buf = BytesMut::new();
        write_varint(&mut buf, 2147483647); // Maximum value for i32
        assert_eq!(buf, BytesMut::from(&[0xFF, 0xFF, 0xFF, 0xFF, 0x07][..]));
    }

    #[test]
    fn test_write_varint_zero_value() {
        let mut buf = BytesMut::new();
        write_varint(&mut buf, 0);
        assert_eq!(buf, BytesMut::from(&[0x00][..]));
    }

    #[test]
    fn test_write_varint_negative_value() {
        let mut buf = BytesMut::new();
        write_varint(&mut buf, -1);
        assert_eq!(buf, BytesMut::from(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F][..]));
    }

    #[test]
    fn test_write_varint_min_value() {
        let mut buf = BytesMut::new();
        write_varint(&mut buf, -2147483648); // Minimum value for i32
        assert_eq!(buf, BytesMut::from(&[0x80, 0x80, 0x80, 0x80, 0x08][..]));
    }

    #[test]
    fn test_write_string() {
        let mut buf = BytesMut::new();
        write_string(&mut buf, "Hello, world!");

        let mut expected = BytesMut::new();
        expected.put_u8(13); // Length of "Hello, world!"
        expected.put_slice(b"Hello, world!");

        assert_eq!(buf, expected);
    }

    #[test]
    fn test_read_string_success() {
        let mut buf = BytesMut::new();
        buf.put_u8(13); // Length of "Hello, world!"
        buf.put_slice("Hello, world!".as_bytes());

        let result = read_string(&mut buf).unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    #[should_panic(expected = "BufferUnderflow")]
    fn test_read_string_insufficient_length() {
        let mut buf = BytesMut::new();
        buf.put_u8(5); // Set the length to 5, but provide only 3 bytes

        buf.put_slice(b"abc");

        let mut buf = buf.freeze();

        read_string(&mut buf).unwrap();
    }

    #[test]
    #[should_panic(expected = "Utf8Error")]
    fn test_read_string_invalid_utf8() {
        let mut buf = BytesMut::new();
        buf.put_u8(3);
        buf.put_slice(&[0xff, 0xff, 0xff]); // Invalid UTF-8 sequence

        let mut buf = buf.freeze();

        read_string(&mut buf).unwrap();
    }
}