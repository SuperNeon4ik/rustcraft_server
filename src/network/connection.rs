use bytes::{buf::Reader, Buf, Bytes, BytesMut};

use crate::{log, network::packet::read_string, LOGGER};
use std::{io::{Error, Read}, net::{Shutdown, SocketAddr, TcpStream}};

use super::packet::read_varint;

pub enum ConnectionState {
    Handshaking,
    Status,
    Login,
    Configuration,
    Play
}

pub enum ServerboundPacket {
    // 1. Handshaking
    HandshakingHandshake(i32, String, u16, i32),
    HandshakingLegacyServerListPing(u8),

    // 2. Status
    StatusStatusRequest(),
    StatusPingRequest(i64),
}

pub enum ClientboundPacket {
    // 2. Status
    StatusStatusResponse(String),
    StatusPingResponse(i64)
}

pub struct Connection {
    stream: TcpStream,
    state: ConnectionState,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection { 
            stream,
            state: ConnectionState::Handshaking, 
        }
    }

    pub fn start_reading(&self) {
        let mut stream = &self.stream;
        let address = stream.peer_addr().unwrap();

        loop {
            let mut bytes:Vec<u8> = Vec::new();
            match stream.read_to_end(&mut bytes) {
                Ok(b)  => {
                    if b == 0 { break; }
                    log!(debug, "Received data ({} bytes): 0x{}", bytes.len(), hex::encode(&bytes));

                    match self.state {
                        ConnectionState::Handshaking => self.handle_handshaking_packet(bytes.clone()),
                        _ => todo!()
                    }
                }
                Err(e) => log!(warn, "Error receiving data: {}", e)
            }
        }
    
        log!(verbose, "Client {}:{} dropped", address.ip(), address.port());
        stream.shutdown(Shutdown::Both).unwrap();
    }

    fn handle_handshaking_packet(&self, data: Vec<u8>) {
        let address = self.stream.peer_addr().unwrap();

        let mut buf = BytesMut::from(&data[..]);
        let length = read_varint(&mut buf).unwrap();
        let packet_id = read_varint(&mut buf).unwrap();

        log!(debug, "Handling handshake packet 0x{} ({} bytes)", hex::encode(packet_id.to_le_bytes()), length);

        match packet_id {
            0x00 => {
                log!(debug, "Handshake from {}:{}:", address.ip(), address.port());

                let protocol_version = read_varint(&mut buf).unwrap();
                log!(debug, "\tprotocol_version = {}", protocol_version);

                let server_address = read_string(&mut buf).unwrap();
                log!(debug, "\tserver_address = {}", server_address);

                let server_port = buf.get_u16_le();
                log!(debug, "\tserver_port = {}", server_port);
                
                let next_state = read_varint(&mut buf).unwrap();
                log!(debug, "\tnext_state = {}", next_state);
            }
            _ => log!(warn, "Unexpected packet during HANDSHAKING with ID 0x{}", hex::encode(packet_id.to_le_bytes()))
        }
    }
}