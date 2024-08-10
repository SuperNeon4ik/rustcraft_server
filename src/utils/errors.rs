use core::fmt;

#[derive(Debug)]
pub enum PacketHandleError {
    BadId(i32),
    ReadError(PacketReadError),
}

#[derive(Debug)]
pub enum PacketReadError {
    EmptyBuf,
    BufferUnderflow,
    TooLong,
    Utf8Error,
}

impl fmt::Display for PacketHandleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadId(id) => write!(f, "Bad packet ID 0x{:x?}", id),
            Self::ReadError(e) => write!(f, "ReadError during handling: {}", e)
        }
    }
}

impl fmt::Display for PacketReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::EmptyBuf => "Empty buffer",
            Self::BufferUnderflow => "Buffer underflow",
            Self::TooLong => "Too long",
            Self::Utf8Error => "UTF-8 Error",
        };

        write!(f, "{}", msg)
    }
}