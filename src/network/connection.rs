use bytes::BytesMut;
use json::object;

use crate::{log, network::{packet::PacketWriter, packet_utils::read_varint}, utils::errors::PacketHandleError, LOGGER};
use core::fmt;
use std::{io::{Read, Write}, net::{Shutdown, TcpStream}, sync::{Arc, Mutex, MutexGuard}};

use super::packet::PacketReader;

#[derive(Clone, PartialEq)]
pub enum ConnectionState {
    Handshaking,
    Status,
    // Login,
    // Configuration,
    // Play,
    Disconnect,
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = match self {
            Self::Handshaking => "Handshaking",
            Self::Status => "Status",
            // Self::Configuration => "Configuration",
            // Self::Login => "Login",
            // Self::Play => "Play",
            Self::Disconnect => "Disconnect",
        };

        write!(f, "{}", state)
    }
}

pub struct Connection {
    stream: Arc<Mutex<TcpStream>>,
    state: Arc<Mutex<ConnectionState>>,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection { 
            stream: Arc::new(Mutex::new(stream)),
            state: Arc::new(Mutex::new(ConnectionState::Handshaking)), 
        }
    }

    pub fn start_reading(&self) {
        let mut stream = self.stream.lock().unwrap();

        let state_binding = Arc::clone(&self.state);
        let mut state = state_binding.lock().unwrap();
        let address = stream.peer_addr().unwrap();

        let mut buf = [0u8; 1024];
        let mut data_accumulator: Vec<u8> = Vec::new();

        loop {
            match stream.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    data_accumulator.extend_from_slice(&buf[..n]);

                    while let Some(packet) = Self::extract_packet(&mut data_accumulator) {
                        let packet_id = packet.id();
                        log!(debug, "Received packet with ID 0x{:x?}", &packet_id);

                        if let Err(e) = match *state {
                            ConnectionState::Handshaking => Self::handle_handshaking_packet(&mut stream, &mut state, packet),
                            ConnectionState::Status => Self::handle_status_packet(&mut stream, packet),
                            _ => todo!()
                        } {
                            log!(warn, "Failed to handle packet 0x{:x?} ({}) for {}:{}: {}", packet_id, *state, address.ip(), address.port(), e);
                        }
                    }
                }
                Err(e) => log!(warn, "Error receiving data: {}", e)
            }

            if *state == ConnectionState::Disconnect { break }
        }
    
        log!(verbose, "Client {}:{} dropped", address.ip(), address.port());
        stream.shutdown(Shutdown::Both).unwrap();
    }

    fn extract_packet(data: &mut Vec<u8>) -> Option<PacketReader> {
        let mut buf = BytesMut::from(&data[..]);
        if let Ok(packet_length) = read_varint(&mut buf) {
            if buf.len() >= packet_length as usize {
                data.drain(..(data.len() - buf.len()));
                let packet: Vec<u8> = data.drain(..packet_length as usize).collect();
                
                if let Ok(packet_reader) = PacketReader::new(&packet) {
                    return Some(packet_reader)    
                }
            }
        }

        None
    }

    fn send_packet_bytes(stream: &mut TcpStream, data: &[u8]) {
        let address = stream.peer_addr().unwrap();
        log!(debug, "Sending packet ({} bytes) to {}:{}: {}", data.len(), address.ip(), address.port(), hex::encode(data));
        stream.write_all(data).unwrap();
    }

    fn handle_handshaking_packet(stream: &mut TcpStream, state: &mut MutexGuard<ConnectionState>, mut packet: PacketReader) -> Result<(), PacketHandleError> {
        let address = stream.peer_addr().unwrap();

        match packet.id() {
            0x00 => {
                log!(debug, "Handshake from {}:{}:", address.ip(), address.port());

                let protocol_version = packet.read_varint().unwrap();
                log!(debug, "\tprotocol_version = {}", protocol_version);

                let server_address = packet.read_string().unwrap();
                log!(debug, "\tserver_address = {}", server_address);

                let server_port = packet.read_ushort().unwrap();
                log!(debug, "\tserver_port = {}", server_port);
                
                let next_state = packet.read_varint().unwrap();
                log!(debug, "\tnext_state = {}", next_state);

                // 1 for Status, 2 for Login, 3 for Transfer
                match next_state {
                    1 => {
                        **state = ConnectionState::Status;
                    }
                    _ => {
                        **state = ConnectionState::Disconnect;
                        log!(warn, "Weird 'next_state' ({}) when handling handshake packet from {}:{}", next_state, address.ip(), address.port());
                    }
                }
            }
            _ => return Err(PacketHandleError::BadId(packet.id()))
        }

        Ok(())
    }

    fn handle_status_packet(stream: &mut TcpStream, mut packet: PacketReader) -> Result<(), PacketHandleError> {
        match packet.id() {
            0x00 => {
                let json_status_response = object! {
                    version: {
                        name: crate::VERSION_NAME,
                        protocol: crate::PROTOCOL_VERSION,
                    },
                    players: {
                        max: 69,
                        online: 69,
                        sample: []
                    },
                    description: {
                        text: "Rusty experimental minecraft server!",
                    },
                    enforcesSecureChat: false,
                };

                let json_status_response_dump = json_status_response.dump();
                log!(debug, "Status response dump: {}", json_status_response_dump);

                let status_response_packet = PacketWriter::new(0x00)
                    .write_string(&json_status_response_dump)
                    .build_uncompressed();

                Self::send_packet_bytes(stream, &status_response_packet);
            }
            0x01 => {
                let client_timestamp = packet.read_long().unwrap();

                let ping_response_packet = PacketWriter::new(0x01)
                    .write_long(client_timestamp)
                    .build_uncompressed();

                Self::send_packet_bytes(stream, &ping_response_packet);
            }
            _ => return Err(PacketHandleError::BadId(packet.id()))
        }

        Ok(())
    }
}