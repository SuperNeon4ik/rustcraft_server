use bytes::BytesMut;
use json::object;

use crate::{log, network::{packet_utils::read_varint, packets::handshaking::serverbound::handshake::{HandshakeNextState, HandshakingServerboundHandshake}}, utils::errors::PacketHandleError, CONFIG, LOGGER};
use core::fmt;
use std::{io::{Read, Write}, net::{Shutdown, TcpStream}, sync::{Arc, Mutex, MutexGuard}};

use super::{packet::{ClientboundPacket, PacketReader, ServerboundPacket}, packets::{status::{clientbound::{ping_response::StatusClientboundPingResponse, status_response::StatusClientboundStatusResponse}, serverbound::ping_request::StatusServerboundPingRequest}, login::{serverbound::login_start::LoginServerboundLoginStart, clientbound::disconnect::LoginClientboundDisconnect}}};

#[derive(Clone, PartialEq)]
pub enum ConnectionState {
    Handshaking,
    Status,
    Login,
    // Configuration,
    // Play,
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = match self {
            Self::Handshaking => "Handshaking",
            Self::Status => "Status",
            Self::Login => "Login",
            // Self::Configuration => "Configuration",
            // Self::Play => "Play",
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

                    while let Some(reader) = Self::extract_packet_reader(&mut data_accumulator) {
                        let packet_id = reader.id();
                        log!(debug, "Received packet with ID 0x{:x?} ({})", &packet_id, *state);

                        if let Err(e) = match *state {
                            ConnectionState::Handshaking => Self::handle_handshaking_packet(&mut stream, &mut state, reader),
                            ConnectionState::Status => Self::handle_status_packet(&mut stream, reader),
                            ConnectionState::Login => Self::handle_login_packet(&mut stream, &mut state, reader),
                        } {
                            log!(warn, "Failed to handle packet 0x{:x?} ({}) for {}:{}: {}", packet_id, *state, address.ip(), address.port(), e);
                        }
                    }
                }
                Err(e) => log!(warn, "Error receiving data: {}", e)
            }
        }
    
        log!(verbose, "Client {}:{} dropped", address.ip(), address.port());
        stream.shutdown(Shutdown::Both).unwrap();
    }

    fn extract_packet_reader(data: &mut Vec<u8>) -> Option<PacketReader> {
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
        log!(debug, "Sending packet ({} bytes) to {}:{}", data.len(), address.ip(), address.port());
        stream.write_all(data).unwrap();
    }

    fn handle_handshaking_packet(stream: &mut TcpStream, state: &mut MutexGuard<ConnectionState>, mut reader: PacketReader) -> Result<(), PacketHandleError> {
        let address = stream.peer_addr().unwrap();

        match reader.id() {
            0x00 => {
                log!(debug, "Handshake from {}:{}:", address.ip(), address.port());
                let packet = HandshakingServerboundHandshake::read(&mut reader)?;
                log!(debug, "\tprotocol_version = {}", packet.protocol_version);
                log!(debug, "\tserver_address = {}", packet.server_address);
                log!(debug, "\tserver_port = {}", packet.server_port);
                log!(debug, "\tnext_state = {}", packet.next_state);

                match packet.next_state {
                    HandshakeNextState::Status => {
                        **state = ConnectionState::Status;
                    }
                    HandshakeNextState::Login => {
                        **state = ConnectionState::Login;
                    }
                    _ => {
                        log!(warn, "Weird 'next_state' ({}) when handling handshake packet from {}:{}", packet.next_state, address.ip(), address.port());
                    }
                }
            }
            _ => return Err(PacketHandleError::BadId(reader.id()))
        }

        Ok(())
    }

    fn handle_status_packet(stream: &mut TcpStream, mut reader: PacketReader) -> Result<(), PacketHandleError> {
        match reader.id() {
            0x00 => {
                let json_status_response = object! {
                    version: {
                        name: CONFIG.status.version_prefix.clone() + " " + crate::VERSION,
                        protocol: crate::PROTOCOL_VERSION,
                    },
                    players: {
                        max: CONFIG.status.max_players,
                        online: 69,
                        sample: []
                    },
                    description: {
                        text: CONFIG.status.motd.clone(),
                    },
                    enforcesSecureChat: false,
                };

                let status_response_packet = StatusClientboundStatusResponse {
                    json_response: json_status_response.dump(),
                };

                Self::send_packet_bytes(stream, &status_response_packet.build());
            }
            0x01 => {
                let client_timestamp = StatusServerboundPingRequest::read(&mut reader)?.timestamp;

                let ping_response_packet = StatusClientboundPingResponse {
                    timestamp: client_timestamp
                };

                Self::send_packet_bytes(stream, &ping_response_packet.build());
            }
            _ => return Err(PacketHandleError::BadId(reader.id()))
        }

        Ok(())
    }

    fn handle_login_packet(stream: &mut TcpStream, state: &mut MutexGuard<ConnectionState>, mut reader: PacketReader) -> Result<(), PacketHandleError> {
        let addr = stream.peer_addr().unwrap();
        let addr_str = format!("{}:{}", addr.ip(), addr.port());

        match reader.id() {
            0x00 => {
                let packet = LoginServerboundLoginStart::read(&mut reader)?;

                log!(info, "Player {}[uuid = {}; ip = {}] sent login_packet", packet.name, packet.uuid, addr_str);

                // DEBUG: disconnect client
                Self::disconnect(stream, state, String::from("Disconnected.\nLogin functionality is not implemented yet."));
            }
            _ => return Err(PacketHandleError::BadId(reader.id()))
        }

        Ok(())
    }

    fn disconnect(stream: &mut TcpStream, state: &mut MutexGuard<ConnectionState>, reason: String) {
        match **state {
            ConnectionState::Login => {
                let login_disconnect_packet = LoginClientboundDisconnect::from_string(reason);
                Self::send_packet_bytes(stream, &login_disconnect_packet.build());
            }
            _ => log!(warn, "Invalid state ({}) while sending disconnect packet.", **state)
        }
    }
}