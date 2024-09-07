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
    UnexpectedValue,
    ConvertationIssue(String),
}

#[derive(Debug)]
pub enum ObjectResponseError {
    ReqwestError(String),
    SerdeParseError(String),
}

impl fmt::Display for PacketHandleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadId(id) => write!(f, "Bad packet ID 0x{:x?}", id),
            Self::ReadError(e) => write!(f, "ReadError during handling: {}", e)
        }
    }
}

impl From<PacketReadError> for PacketHandleError {
    fn from(err: PacketReadError) -> PacketHandleError {
        PacketHandleError::ReadError(err)
    }
}

impl fmt::Display for PacketReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::EmptyBuf => "Empty buffer",
            Self::BufferUnderflow => "Buffer underflow",
            Self::TooLong => "Too long",
            Self::Utf8Error => "UTF-8 Error",
            Self::UnexpectedValue => "Unexpected value",
            Self::ConvertationIssue(details) => &("Convertation issue: ".to_owned() + details)
        };

        write!(f, "{}", msg)
    }
}

impl fmt::Display for ObjectResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReqwestError(e) => write!(f, "Error while sending request: {}", e),
            Self::SerdeParseError(e) => write!(f, "Failed to parse object: {}", e)
        }
    }
}

impl From<reqwest::Error> for ObjectResponseError {
    fn from(err: reqwest::Error) -> ObjectResponseError {
        ObjectResponseError::ReqwestError(err.to_string())
    }
}