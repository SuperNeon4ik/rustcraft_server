use bytes::BytesMut;
use json::object;

use crate::{log, network::packets::handshaking::serverbound::handshake::{HandshakeNextState, HandshakingServerboundHandshake}, utils::{errors::PacketHandleError, packet_utils::read_varint}, CONFIG, LOGGER};
use core::fmt;
use std::{io::{Read, Write}, net::{Shutdown, TcpStream}, sync::{Arc, Mutex}};

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
    pub connection_info: Arc<Mutex<Option<ConnectionInfo>>>,
}

pub struct ConnectionInfo {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16, 
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection { 
            stream: Arc::new(Mutex::new(stream)),
            state: Arc::new(Mutex::new(ConnectionState::Handshaking)), 
            connection_info: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_reading(&self) {
        let mut stream = self.stream.lock().unwrap();
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
                        let state = self.state.lock().unwrap();
                        log!(debug, "Received packet with ID 0x{:x?} ({})", &packet_id, *state);

                        if let Err(e) = self.handle_packet(reader) {
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

    fn send_packet_bytes(&self, data: &[u8]) {
        let mut stream = self.stream.lock().unwrap();
        let address = stream.peer_addr().unwrap();
        log!(debug, "Sending packet ({} bytes) to {}:{}", data.len(), address.ip(), address.port());
        stream.write_all(data).unwrap();
    }

    fn get_addr(&self) -> String {
        let addr = self.stream.lock().unwrap().peer_addr().unwrap();
        format!("{}:{}", addr.ip(), addr.port())
    }

    fn handle_packet(&self, reader: PacketReader) -> Result<(), PacketHandleError> {
        let state = self.state.lock().unwrap();
        match *state {
            ConnectionState::Handshaking => Ok(self.handle_handshaking_packet(reader)?),
            ConnectionState::Status => Ok(self.handle_status_packet(reader)?),
            ConnectionState::Login => Ok(self.handle_login_packet(reader)?),
        }
    }

    fn handle_handshaking_packet(&self, mut reader: PacketReader) -> Result<(), PacketHandleError> {
        match reader.id() {
            0x00 => {
                log!(debug, "Handshake from {}:", self.get_addr());
                let packet = HandshakingServerboundHandshake::read(&mut reader)?;
                log!(debug, "\tprotocol_version = {}", packet.protocol_version);
                log!(debug, "\tserver_address = {}", packet.server_address);
                log!(debug, "\tserver_port = {}", packet.server_port);
                log!(debug, "\tnext_state = {}", packet.next_state);

                let mut connection_info = self.connection_info.lock().unwrap();
                *connection_info = Some(ConnectionInfo {
                    protocol_version: packet.protocol_version,
                    server_address: packet.server_address,
                    server_port: packet.server_port,
                });

                let mut state = self.state.lock().unwrap();
                match packet.next_state {
                    HandshakeNextState::Status => {
                        *state = ConnectionState::Status;
                    }
                    HandshakeNextState::Login => {
                        *state = ConnectionState::Login;
                    }
                    _ => {
                        log!(warn, "Weird 'next_state' ({}) when handling handshake packet from {}", packet.next_state, self.get_addr());
                    }
                }
            }
            _ => return Err(PacketHandleError::BadId(reader.id()))
        }

        Ok(())
    }

    fn handle_status_packet(&self, mut reader: PacketReader) -> Result<(), PacketHandleError> {
        match reader.id() {
            0x00 => {
                let json_status_response = object! {
                    version: {
                        name: CONFIG.status.version_prefix.clone() + " " + crate::VERSION,
                        protocol: crate::PROTOCOL_VERSION,
                    },
                    players: {
                        max: CONFIG.server.max_players,
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

                self.send_packet_bytes(&status_response_packet.build());
            }
            0x01 => {
                let client_timestamp = StatusServerboundPingRequest::read(&mut reader)?.timestamp;

                let ping_response_packet = StatusClientboundPingResponse {
                    timestamp: client_timestamp
                };

                self.send_packet_bytes(&ping_response_packet.build());
            }
            _ => return Err(PacketHandleError::BadId(reader.id()))
        }

        Ok(())
    }

    fn handle_login_packet(&self, mut reader: PacketReader) -> Result<(), PacketHandleError> {
        match reader.id() {
            0x00 => {
                let packet = LoginServerboundLoginStart::read(&mut reader)?;

                log!(info, "Player {}[uuid = {}; ip = {}] is logging in", packet.name, packet.uuid, self.get_addr());

                let connection_info_ref = self.connection_info.lock().unwrap();
                if let Some(connection_info) = &*connection_info_ref {
                    if connection_info.protocol_version != crate::PROTOCOL_VERSION {
                        self.disconnect(format!("Your protocol version ({}) doesn't match server's protocol version ({})", connection_info.protocol_version, crate::PROTOCOL_VERSION));
                        return Ok(());
                    }
                }

                // DEBUG: disconnect client
                self.disconnect(String::from("Login functionality is not implemented yet."));
            }
            _ => return Err(PacketHandleError::BadId(reader.id()))
        }

        Ok(())
    }

    fn disconnect(&self, reason: String) {
        match *self.state.lock().unwrap() {
            ConnectionState::Login => {
                let login_disconnect_packet = LoginClientboundDisconnect::from_string(reason);
                self.send_packet_bytes(&login_disconnect_packet.build());
            }
            _ => log!(warn, "Invalid state ({}) while sending disconnect packet.", *self.state.lock().unwrap())
        }
    }
}