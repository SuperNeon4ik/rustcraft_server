use bytes::{Buf, BufMut, BytesMut};
use uuid::Uuid;
use crate::utils::errors::PacketReadError;
use crate::utils::packet_utils::{read_string, read_varint, write_string, write_varint};

pub struct PacketReader {
    packet_id: i32,
    data: BytesMut,
}

pub struct PacketWriter {
    packet_id: i32,
    data: BytesMut,
}

#[allow(unused)]
impl PacketReader {
    pub fn new(data: &[u8]) -> Result<Self, PacketReadError> {
        let mut buf = BytesMut::from(data);
        let packet_id = read_varint(&mut buf)?;

        Ok(PacketReader {
            packet_id,
            data: buf,
        })
    }

    pub fn id(&self) -> i32 {
        self.packet_id
    }

    pub fn data(&self) -> BytesMut {
        self.data.clone()
    }

    pub fn read_varint(&mut self) -> Result<i32, PacketReadError> {
        read_varint(&mut self.data)
    }

    pub fn read_string(&mut self) -> Result<String, PacketReadError> {
        read_string(&mut self.data)
    }

    pub fn read_uuid(&mut self) -> Result<Uuid, PacketReadError> {
        if self.data.remaining() < 16 { return Err(PacketReadError::BufferUnderflow); }
        let encoded_uuid = self.data.get_u128_le();
        Ok(Uuid::from_u128_le(encoded_uuid))
    }

    pub fn read_byte(&mut self) -> Result<i8, PacketReadError> {
        if self.data.remaining() < 1 { Err(PacketReadError::BufferUnderflow) }
        else { Ok(self.data.get_i8()) }
    }

    pub fn read_ubyte(&mut self) -> Result<u8, PacketReadError> {
        if self.data.remaining() < 1 { Err(PacketReadError::BufferUnderflow) }
        else { Ok(self.data.get_u8()) }
    }

    pub fn read_byte_array(&mut self, size: usize) -> Result<Vec<u8>, PacketReadError> {
        if self.data.remaining() < size { return Err(PacketReadError::BufferUnderflow); }
        let mut bytes = vec![0u8; size];
        self.data.copy_to_slice(&mut bytes);
        Ok(bytes)
    }
    
    pub fn read_short(&mut self) -> Result<i16, PacketReadError> {
        if self.data.remaining() < 2 { Err(PacketReadError::BufferUnderflow) }
        else { Ok(self.data.get_i16_le()) }
    }

    pub fn read_ushort(&mut self) -> Result<u16, PacketReadError> {
        if self.data.remaining() < 2 { Err(PacketReadError::BufferUnderflow) }
        else { Ok(self.data.get_u16_le()) } // TODO: This reads wrong value on my Windows laptop
    }

    pub fn read_int(&mut self) -> Result<i32, PacketReadError> {
        if self.data.remaining() < 4 { Err(PacketReadError::BufferUnderflow) }
        else { Ok(self.data.get_i32_le()) }
    }

    pub fn read_long(&mut self) -> Result<i64, PacketReadError> {
        if self.data.remaining() < 8 { Err(PacketReadError::BufferUnderflow) }
        else { Ok(self.data.get_i64_le()) }
    }

    pub fn read_float(&mut self) -> Result<f32, PacketReadError> {
        if self.data.remaining() < 4 { Err(PacketReadError::BufferUnderflow) }
        else { Ok(self.data.get_f32_le()) }
    }

    pub fn read_double(&mut self) -> Result<f64, PacketReadError> {
        if self.data.remaining() < 8 { Err(PacketReadError::BufferUnderflow) }
        else { Ok(self.data.get_f64_le()) }
    }
}

#[allow(unused)]
impl PacketWriter {
    pub fn new(packet_id: i32) -> Self {
        PacketWriter {
            packet_id,
            data: BytesMut::new()
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn write_varint(&mut self, n: i32) -> &Self {
        write_varint(&mut self.data, n);
        self
    }

    pub fn write_string(&mut self, text: &str) -> &Self {
        write_string(&mut self.data, text);
        self
    }

    pub fn write_uuid(&mut self, uuid: Uuid) -> &Self {
        self.data.put_u128_le(uuid.as_u128());
        self
    }

    pub fn write_boolean(&mut self, val: bool) -> &Self {
        if val { self.data.put_u8(0x01); }
        else { self.data.put_u8(0x00); }
        self
    }

    pub fn write_byte(&mut self, n: i8) -> &Self {
        self.data.put_i8(n);
        self
    }

    pub fn write_ubyte(&mut self, n: u8) -> &Self {
        self.data.put_u8(n);
        self
    }

    pub fn write_byte_array(&mut self, val: &[u8]) -> &Self {
        self.data.put_slice(val);
        self
    }

    pub fn write_short(&mut self, n: i16) -> &Self {
        self.data.put_i16_le(n);
        self
    }

    pub fn write_ushort(&mut self, n: u16) -> &Self {
        self.data.put_u16_le(n);
        self
    }

    pub fn write_int(&mut self, n: i32) -> &Self {
        self.data.put_i32_le(n);
        self
    }

    pub fn write_long(&mut self, n: i64) -> &Self {
        self.data.put_i64_le(n);
        self
    }

    pub fn write_float(&mut self, val: f32) -> &Self {
        self.data.put_f32_le(val);
        self
    }

    pub fn write_double(&mut self, val: f64) -> &Self {
        self.data.put_f64_le(val);
        self
    }

    pub fn build_uncompressed(&self) -> Vec<u8> {
        let mut packet_buf = BytesMut::with_capacity(7);
        write_varint(&mut packet_buf, self.packet_id);
        packet_buf.put(self.data());
    
        let mut final_buf = BytesMut::new();
        write_varint(&mut final_buf, packet_buf.len() as i32);
        final_buf.put(packet_buf);
    
        final_buf.to_vec()
    }
}

pub trait ClientboundPacket {
    fn packet_id() -> i32;
    fn build(&self) -> Vec<u8>;
}

pub trait ServerboundPacket {
    fn packet_id() -> i32 
    where 
        Self: Sized;

    fn read(reader: &mut PacketReader) -> Result<Self, PacketReadError>
    where 
        Self: Sized;
}

// pub struct PacketFactory {
//     creators: HashMap<i32, fn(&mut PacketReader) -> Result<Box<dyn ServerboundPacket>, PacketReadError>>,
// }

// impl PacketFactory {
//     pub fn new(state: &ConnectionState) -> Self {
//         let mut factory = PacketFactory {
//             creators: HashMap::new(),
//         };

//         // Register packets
//         match state {
//             ConnectionState::Handshaking => {
//                 factory.register::<HandshakingServerboundHandshake>();
//             }
//             ConnectionState::Status => {
//                 factory.register::<StatusServerboundPingRequest>();
//                 factory.register::<StatusServerboundStatusRequest>();
//             }
//             _ => {}
//         }

//         factory
//     }

//     pub fn register<T: ServerboundPacket + 'static>(&mut self) {
//         self.creators.insert(T::packet_id(), |reader| {
//             Ok(Box::new(T::read(reader)?) as Box<dyn ServerboundPacket>)
//         });
//     }

//     pub fn read_packet(&self, reader: &mut PacketReader) -> Result<Box<dyn ServerboundPacket>, PacketReadError> {
//         if let Some(creator) = self.creators.get(&reader.packet_id) {
//             creator(reader)
//         } else {
//             Err(PacketReadError::UnknownPacketId)
//         }
//     }
// }