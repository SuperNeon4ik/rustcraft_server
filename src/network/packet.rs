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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_varint_single_byte() {
        let mut buf = BytesMut::from(&[0x01][..]);
        assert_eq!(read_varint(&mut buf), Some(1));
    }

    #[test]
    fn test_read_varint_multi_byte() {
        let mut buf = BytesMut::from(&[0xAC, 0x02][..]);
        assert_eq!(read_varint(&mut buf), Some(300));
    }

    #[test]
    fn test_read_varint_negative() {
        let mut buf = BytesMut::from(&[0xff, 0xff, 0xff, 0xff, 0x0f][..]);
        assert_eq!(read_varint(&mut buf), Some(-1));
    }

    #[test]
    fn test_read_varint_max_value() {
        let mut buf = BytesMut::from(&[0xFF, 0xFF, 0xFF, 0xFF, 0x07][..]);
        assert_eq!(read_varint(&mut buf), Some(2147483647)); // Maximum value for i32
    }

    #[test]
    fn test_read_varint_incomplete_data() {
        let mut buf = BytesMut::from(&[0xFF, 0xFF][..]);
        assert_eq!(read_varint(&mut buf), None);
    }

    #[test]
    fn test_read_varint_too_long() {
        let mut buf = BytesMut::from(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x02][..]);
        assert_eq!(read_varint(&mut buf), None);
    }

    #[test]
    fn test_read_varint_empty_buffer() {
        let mut buf = BytesMut::new();
        assert_eq!(read_varint(&mut buf), None);
    }

    #[test]
    fn test_read_varint_zero_value() {
        let mut buf = BytesMut::from(&[0x00][..]);
        assert_eq!(read_varint(&mut buf), Some(0));
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
        write_varint(&mut buf, 13); // Length of "Hello, world!"
        write_string(&mut buf, "Hello, world!");

        let result = read_string(&mut buf).unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_read_string_insufficient_length() {
        let mut buf = BytesMut::new();
        buf.put_u8(5); // Set the length to 5, but provide only 3 bytes

        buf.put_slice(b"abc");

        let mut buf = buf.freeze();

        let result = read_string(&mut buf);
        assert!(result.is_none());
    }

    #[test]
    #[should_panic(expected = "from_utf8")]
    fn test_read_string_invalid_utf8() {
        let mut buf = BytesMut::new();
        buf.put_u8(3);
        buf.put_slice(&[0xff, 0xff, 0xff]); // Invalid UTF-8 sequence

        let mut buf = buf.freeze();

        read_string(&mut buf).unwrap();
    }

    #[test]
    fn test_prepare_uncompressed_packet_single_byte_id() {
        let mut buf = BytesMut::from(&b"Hello"[..]); // Example payload
        let packet_id = 1; // Single-byte packet ID

        let result = prepare_uncompressed_packet(&mut buf, packet_id);

        // Expected result:
        // - length: 1 byte for packet ID + 5 bytes of "Hello" = 6 bytes
        // - length varint: 1 byte (for value 6)
        // - packet ID varint: 1 byte (for value 1)
        // - payload: "Hello"
        let expected = vec![6, 1, b'H', b'e', b'l', b'l', b'o'];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_prepare_uncompressed_packet_multi_byte_id() {
        let mut buf = BytesMut::from(&b"World"[..]); // Example payload
        let packet_id = 300; // Multi-byte packet ID

        let result = prepare_uncompressed_packet(&mut buf, packet_id);

        // Expected result:
        // - length: 2 bytes for packet ID (300) + 5 bytes of "World" = 7 bytes
        // - length varint: 1 byte (for value 7)
        // - packet ID varint: 2 bytes (for value 300)
        // - payload: "World"
        let expected = vec![7, 0xac, 0x02, b'W', b'o', b'r', b'l', b'd'];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_prepare_uncompressed_packet_empty_payload() {
        let mut buf = BytesMut::new(); // Empty payload
        let packet_id = 42; // Example packet ID

        let result = prepare_uncompressed_packet(&mut buf, packet_id);

        // Expected result:
        // - length: 1 byte for packet ID = 1 byte
        // - length varint: 1 byte (for value 1)
        // - packet ID varint: 1 byte (for value 42)
        let expected = vec![1, 42];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_prepare_uncompressed_packet_large_payload() {
        let payload = vec![b'a'; 1024]; // Large payload (1KB of 'a')
        let mut buf = BytesMut::from(&payload[..]);
        let packet_id = 12345; // Example packet ID

        let result = prepare_uncompressed_packet(&mut buf, packet_id);

        // Expected result:
        // - length: 3 bytes for packet ID (12345) + 1024 bytes of payload = 1027 bytes
        // - length varint: 2 bytes (for value 1027)
        // - packet ID varint: 3 bytes (for value 12345)
        // - payload: 1024 bytes of 'a'
        let mut expected = vec![0x84, 0x08, 0xb9, 0x60];
        expected.extend_from_slice(&payload);

        assert_eq!(result, expected);
    }
}