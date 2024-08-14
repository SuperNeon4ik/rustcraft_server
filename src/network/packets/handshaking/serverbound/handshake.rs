use std::fmt;

use crate::{network::{connection::ConnectionState, packet::*}, utils::errors::PacketReadError};

pub struct HandshakingServerboundHandshake {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: HandshakeNextState
}

pub enum HandshakeNextState {
    Status,
    Login,
    Transfer,
}

impl fmt::Display for HandshakeNextState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::Status => "Status",
            Self::Login => "Login",
            Self::Transfer => "Transfer",
        };
        
        write!(f, "{}", msg)
    }
}

impl ServerboundPacket for HandshakingServerboundHandshake {
    fn packet_id() -> i32 
    where 
        Self: Sized {
        0x00
    }

    fn read(reader: &mut PacketReader) -> Result<Self, PacketReadError>
    where 
        Self: Sized {
            let protocol_version = reader.read_varint()?;
            let server_address = reader.read_string()?;
            let server_port = reader.read_ushort()?;
            let next_state = match reader.read_varint()? {
                1 => HandshakeNextState::Status,
                2 => HandshakeNextState::Login,
                3 => HandshakeNextState::Transfer,
                _ => return Err(PacketReadError::UnexpectedValue)
            };

            Ok(Self {
                protocol_version,
                server_address,
                server_port,
                next_state,
            })
    }
}