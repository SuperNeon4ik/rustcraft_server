use aes::cipher::AsyncStreamCipher;
use rand::Rng;
use rand::thread_rng;
use bytes::BytesMut;
use json::object;
use rsa::Pkcs1v15Encrypt;
use rsa::pkcs8::EncodePublicKey;
use uuid::Uuid;

use crate::crypto::aes_util;
use crate::crypto::aes_util::Aes128Cfb8Dec;
use crate::crypto::aes_util::Aes128Cfb8Enc;
use crate::network::packets::login::clientbound::login_success::LoginClientboundLoginSuccess;
use crate::network::packets::login::clientbound::login_success::LoginSuccessProperty;
use crate::utils::mojauth::authenticate_player;
use crate::{log, network::packets::{handshaking::serverbound::handshake::{HandshakeNextState, HandshakingServerboundHandshake}, login::clientbound::encryption_request::LoginClientboundEncryptionRequest}, utils::{errors::PacketHandleError, packet_utils::read_varint}, CONFIG, LOGGER, server::ServerData};
use core::fmt;
use std::{io::{Read, Write}, net::{Shutdown, TcpStream}, sync::{Arc, Mutex}};

use super::packets::login::serverbound::encryption_response::LoginServerboundEncryptionResponse;
use super::{packet::{ClientboundPacket, PacketReader, ServerboundPacket}, packets::{status::{clientbound::{ping_response::StatusClientboundPingResponse, status_response::StatusClientboundStatusResponse}, serverbound::ping_request::StatusServerboundPingRequest}, login::{serverbound::login_start::LoginServerboundLoginStart, clientbound::disconnect::LoginClientboundDisconnect}}};

#[allow(unused)]
#[derive(Clone, PartialEq)]
pub enum ConnectionState {
    Handshaking,
    Status,
    Login,
    Configuration,
    Play,
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = match self {
            Self::Handshaking => "Handshaking",
            Self::Status => "Status",
            Self::Login => "Login",
            Self::Configuration => "Configuration",
            Self::Play => "Play",
        };

        write!(f, "{}", state)
    }
}

pub struct Connection {
    stream: Arc<Mutex<TcpStream>>,
    state: Arc<Mutex<ConnectionState>>,
    server_data: ServerData,
    verify_token: Mutex<Option<Vec<u8>>>,
    encryption_setting: Mutex<EncryptionSetting>,
    name: Mutex<Option<String>>,
    uuid: Mutex<Uuid>,
    pub connection_info: Arc<Mutex<Option<ConnectionInfo>>>,
}

#[allow(unused)]
pub struct ConnectionInfo {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16, 
}

enum EncryptionSetting {
    Disabled,
    Encrypted(Box<Aes128Cfb8Enc>, Box<Aes128Cfb8Dec>),
}

impl Connection {
    pub fn new(stream: TcpStream, server_data: &ServerData) -> Self {
        Connection { 
            stream: Arc::new(Mutex::new(stream)),
            state: Arc::new(Mutex::new(ConnectionState::Handshaking)), 
            server_data: server_data.clone(),
            verify_token: Mutex::new(None),
            encryption_setting: Mutex::new(EncryptionSetting::Disabled),
            name: Mutex::new(None),
            uuid: Mutex::new(Uuid::new_v4()),
            connection_info: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_reading(&self) {
        let stream_binding = Arc::clone(&self.stream);
        let address = self.stream.lock().unwrap().peer_addr().unwrap();

        let mut buf = [0u8; 1024];
        let mut data_accumulator: Vec<u8> = Vec::new();
        
        loop {
            let mut stream = stream_binding.lock().unwrap();
            match stream.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let mut encryption_setting = self.encryption_setting.lock().unwrap();
                    if let EncryptionSetting::Encrypted(_, ref mut decryptor) = *encryption_setting {
                        decryptor.clone().decrypt(&mut buf); // TODO: Check if it's possible to remove cloning here
                    }
                    
                    data_accumulator.extend_from_slice(&buf[..n]);
                    drop(encryption_setting);
                    drop(stream);

                    while let Some(reader) = self.extract_packet_reader(&mut data_accumulator) {
                        let packet_id = reader.id();
                        log!(debug, "Received packet with ID 0x{:x?}", &packet_id);

                        if let Err(e) = self.handle_packet(reader) {
                            log!(warn, "Failed to handle packet 0x{:x?} for {}: {}", packet_id, self.get_addr(), e);
                        }
                    }
                }
                Err(e) => log!(warn, "Error receiving data: {}", e)
            }
        }
    
        log!(verbose, "Client {}:{} dropped", address.ip(), address.port());
        self.stream.lock().unwrap().shutdown(Shutdown::Both).unwrap();
    }

    fn extract_packet_reader(&self, data: &mut Vec<u8>) -> Option<PacketReader> {
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
        let mut data: Vec<u8> = data.to_vec();
        let mut encryption_setting = self.encryption_setting.lock().unwrap();
        if let EncryptionSetting::Encrypted(ref mut encryptor, _) = *encryption_setting {
            encryptor.clone().encrypt(&mut data); // TODO: Check if it's possible to remove cloning here
        }

        stream.write_all(&data).unwrap();
        drop(stream);
        log!(debug, "Sent packet ({} bytes) to {}", data.len(), self.get_addr());
    }

    fn get_addr(&self) -> String {
        let addr = self.stream.lock().unwrap().peer_addr().unwrap();
        format!("{}:{}", addr.ip(), addr.port())
    }

    fn handle_packet(&self, reader: PacketReader) -> Result<(), PacketHandleError> {
        let state_ref = self.state.lock().unwrap();

        match *state_ref {
            ConnectionState::Handshaking => {
                drop(state_ref);
                Ok(self.handle_handshaking_packet(reader)?)
            },
            ConnectionState::Status => {
                drop(state_ref);
                Ok(self.handle_status_packet(reader)?)
            },
            ConnectionState::Login => {
                drop(state_ref);
                Ok(self.handle_login_packet(reader)?)
            },
            ConnectionState::Configuration => {
                drop(state_ref);
                Ok(())
            },
            ConnectionState::Play => {
                drop(state_ref);
                Ok(())
            }
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
                    HandshakeNextState::Status => *state = ConnectionState::Status,
                    HandshakeNextState::Login => *state = ConnectionState::Login,
                    _ => log!(warn, "Weird 'next_state' ({}) when handling handshake packet from {}", packet.next_state, self.get_addr()),
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
                *self.name.lock().unwrap() = Some(packet.name);
                *self.uuid.lock().unwrap() = packet.uuid;

                if let Some(connection_info) = &*self.connection_info.lock().unwrap() {
                    if connection_info.protocol_version != crate::PROTOCOL_VERSION {
                        self.disconnect(format!("Your protocol version ({}) doesn't match server's protocol version ({})", connection_info.protocol_version, crate::PROTOCOL_VERSION));
                        return Ok(());
                    }
                }

                let public_key_der = self.server_data.public_key.to_public_key_der().unwrap();
                let verify_token = Self::generate_verify_token(4);

                *self.verify_token.lock().unwrap() = Some(verify_token.clone());
                let encryption_request_packet = LoginClientboundEncryptionRequest {
                    public_key: public_key_der.to_vec(),
                    verify_token,
                    should_authenticate: CONFIG.server.online_mode,
                };

                self.send_packet_bytes(&encryption_request_packet.build());
            }
            0x01 => {
                let packet = LoginServerboundEncryptionResponse::read(&mut reader)?;

                match &*self.verify_token.lock().unwrap() {
                    Some(verify_token) => {
                        let decrypted_verify_token = self.server_data.private_key.decrypt(Pkcs1v15Encrypt, &packet.verify_token).unwrap();

                        if *verify_token != decrypted_verify_token {
                            log!(warn, "Verify tokens for {} didn't match.", self.get_addr());
                            self.disconnect("Verify tokens didn't match.".to_owned());
                            return Ok(());
                        }
                    }
                    None => {
                        log!(error, "Client sent Encryption Response, but a verify token wasn't saved for them.");
                        self.disconnect("Failure to set up encryption".to_owned());
                        return Ok(());
                    }
                };

                let shared_secret = self.server_data.private_key.decrypt(Pkcs1v15Encrypt, &packet.shared_secret).unwrap();

                *self.verify_token.lock().unwrap() = None;
                let (encryptor, decryptor) = aes_util::initialize(&shared_secret); // turn on encryption
                *self.encryption_setting.lock().unwrap() = EncryptionSetting::Encrypted(Box::new(encryptor), Box::new(decryptor));

                log!(verbose, "Encryption with {} is set up.", self.get_addr());

                if CONFIG.server.online_mode {
                    // Authenticate
                    log!(verbose, "Authenticating {}...", self.get_addr());

                    let public_key_der = self.server_data.public_key.to_public_key_der().unwrap();
                    let username = self.name.lock().unwrap().clone();

                    if let Some(username) = username {
                        match authenticate_player(username.to_owned(), &shared_secret, public_key_der.as_bytes()) {
                            Ok(response) => {
                                let uuid = Uuid::parse_str(&response.id).unwrap();
                                *self.uuid.lock().unwrap() = uuid;

                                log!(verbose, "Authentication for {}[{}] succeeded!", self.get_addr(), uuid);

                                let mut properties: Vec<LoginSuccessProperty> = Vec::new();

                                for property in response.properties {
                                    properties.push(LoginSuccessProperty { 
                                        name: property.name, 
                                        value: property.value, 
                                        signature: Some(property.signature) 
                                    });
                                }

                                let login_success_packet = LoginClientboundLoginSuccess {
                                    uuid,
                                    username,
                                    properties,
                                    strict_error_handling: false,
                                };

                                self.send_packet_bytes(&login_success_packet.build());
                            },
                            Err(e) => {
                                log!(error, "Failed to authenticate player {}[{}]: {}", username, self.get_addr(), e);
                                self.disconnect("Failed to authenticate".to_owned());
                                return Ok(());
                            },
                        }   
                    }
                    else {
                        log!(error, "Client {} sent Encryption Response before Login Start", self.get_addr());
                        self.disconnect("Failed to authenticate".to_owned());
                        return Ok(());
                    }
                }
                else {
                    // Authentication skipped (offline mode)
                    let uuid = *self.uuid.lock().unwrap();
                    let username = (*self.name.lock().unwrap().clone().unwrap()).to_string();

                    let login_success_packet = LoginClientboundLoginSuccess {
                        uuid,
                        username,
                        properties: Vec::new(),
                        strict_error_handling: false,
                    };

                    self.send_packet_bytes(&login_success_packet.build());
                }
            }
            0x03 => {
                *self.state.lock().unwrap() = ConnectionState::Configuration;
                log!(verbose, "Client {} reached Login Acknowledged!!!", self.get_addr());
            }
            _ => return Err(PacketHandleError::BadId(reader.id()))
        }

        Ok(())
    }

    fn disconnect(&self, reason: String) {
        let connection_state = self.state.lock().unwrap();
        match *connection_state {
            ConnectionState::Login => {
                let login_disconnect_packet = LoginClientboundDisconnect::from_string(reason);
                self.send_packet_bytes(&login_disconnect_packet.build());
            }
            _ => log!(warn, "Invalid state ({}) while sending disconnect packet.", *connection_state)
        }
    }

    fn generate_verify_token(size: usize) -> Vec<u8> {
        let mut rng = thread_rng();
        (0..size).map(|_| rng.gen()).collect()
    }
}