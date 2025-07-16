use rand::Rng;
use rand::thread_rng;
use bytes::{Bytes, BytesMut};
use json::object;
use rsa::Pkcs1v15Encrypt;
use rsa::pkcs8::EncodePublicKey;
use uuid::Uuid;

use crate::crypto::aes_util;
use crate::crypto::aes_util::Aes128Cfb8Dec;
use crate::crypto::aes_util::Aes128Cfb8Enc;
use crate::crypto::aes_util::SimpleDecryptor;
use crate::crypto::aes_util::SimpleEncryptor;
use crate::custom_types::identifier::Identifier;
use crate::network::packets::configuration::clientbound::finish_configuration::ConfigurationClientboundFinishConfiguration;
use crate::network::packets::configuration::clientbound::plugin_message::ConfigurationClientboundPluginMessage;
use crate::network::packets::login::clientbound::login_success::LoginClientboundLoginSuccess;
use crate::network::packets::login::clientbound::login_success::LoginSuccessProperty;
use crate::network::packets::play::clientbound::login::PlayClientboundLogin;
use crate::utils::mojauth::authenticate_player;
use crate::{log, network::packets::{handshaking::serverbound::handshake::{HandshakeNextState, HandshakingServerboundHandshake}, login::clientbound::encryption_request::LoginClientboundEncryptionRequest}, utils::{errors::PacketHandleError, packet_utils::read_varint}, CONFIG, LOGGER, server::ServerData};
use core::fmt;
use std::sync::RwLock;
use std::{io::{Read, Write}, net::{Shutdown, TcpStream}, sync::{Arc, Mutex}};
use crate::utils::errors::PacketReadError;
use crate::utils::packet_utils::write_string;
use super::packets::configuration::clientbound::disconnect::ConfigurationClientboundDisconnect;
use super::packets::configuration::serverbound::client_information::ConfigurationServerboundClientInformation;
use super::packets::configuration::serverbound::plugin_message::ConfigurationServerboundPluginMessage;
use super::packets::login::serverbound::encryption_response::LoginServerboundEncryptionResponse;
use super::{packet::{ClientboundPacket, PacketReader, ServerboundPacket}, packets::{status::{clientbound::{ping_response::StatusClientboundPingResponse, status_response::StatusClientboundStatusResponse}, serverbound::ping_request::StatusServerboundPingRequest}, login::{serverbound::login_start::LoginServerboundLoginStart, clientbound::disconnect::LoginClientboundDisconnect}}};


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

        write!(f, "{state}")
    }
}

pub struct Connection {
    stream: Arc<Mutex<TcpStream>>,
    state: Arc<RwLock<ConnectionState>>,
    server_data: Arc<ServerData>,
    verify_token: Arc<Mutex<Option<Vec<u8>>>>,
    encryption_setting: Arc<Mutex<EncryptionSetting>>,
    name: Arc<RwLock<Option<String>>>,
    uuid: Arc<RwLock<Uuid>>,
    pub connection_info: Arc<RwLock<Option<ConnectionInfo>>>,
}


pub struct ConnectionInfo {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16, 
}

enum EncryptionSetting {
    Disabled,
    Encrypted(Box<Aes128Cfb8Enc>, Box<Aes128Cfb8Dec>),
}

impl EncryptionSetting {
    fn encrypt(&mut self, buf: &mut [u8]) {
        if let EncryptionSetting::Encrypted(encryptor, _) = self {
            buf.copy_from_slice(&encryptor.encrypt(buf));
        }
    }

    fn decrypt(&mut self, buf: &mut [u8]) {
        if let EncryptionSetting::Encrypted(_, decryptor) = self {
            buf.copy_from_slice(&decryptor.decrypt(buf));
        }
    }
}

impl Connection {
    pub fn new(stream: TcpStream, server_data: &ServerData) -> Self {
        Connection { 
            stream: Arc::new(Mutex::new(stream)),
            state: Arc::new(RwLock::new(ConnectionState::Handshaking)), 
            server_data: Arc::new(server_data.clone()),
            verify_token: Arc::new(Mutex::new(None)),
            encryption_setting: Arc::new(Mutex::new(EncryptionSetting::Disabled)),
            name: Arc::new(RwLock::new(None)),
            uuid: Arc::new(RwLock::new(Uuid::new_v4())),
            connection_info: Arc::new(RwLock::new(None)),
        }
    }

    pub fn start_reading(&self) {
        let stream_binding = Arc::clone(&self.stream);

        let mut buf = [0u8; 1024];
        let mut data_accumulator: Vec<u8> = Vec::new();

        loop {
            let mut stream = stream_binding.lock().unwrap();
            match stream.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let slice = &mut buf[..n];
                    {
                        self.encryption_setting.lock().unwrap().decrypt(slice);
                    }
                    data_accumulator.extend_from_slice(slice);
                    drop(stream);

                    while let Some(reader) = self.extract_packet_reader(&mut data_accumulator) {
                        let packet_id = reader.id();
                        log!(debug, "Received packet with ID 0x{:x?} from {}", &packet_id, self.get_name());

                        if let Err(e) = self.handle_packet(reader) {
                            log!(warn, "Failed to handle packet 0x{:x?} for {}: {}", packet_id, self.get_name(), e);
                        }
                    }
                }
                Err(e) => log!(warn, "Error receiving data: {}", e)
            }
        }
    
        log!(verbose, "Client {} dropped", self.get_addr());
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

        {
            self.encryption_setting.lock().unwrap().encrypt(&mut data);
        }

        stream.write_all(&data).unwrap();
        drop(stream);
        log!(debug, "Sent packet ({} bytes) to {}", data.len(), self.get_name());
    }

    fn get_addr(&self) -> String {
        let addr = self.stream.lock().unwrap().peer_addr().unwrap();
        format!("{}:{}", addr.ip(), addr.port())
    }

    fn get_name(&self) -> String {
        let name = self.name.read().unwrap().clone();
        match name {
            Some(n) => {
                let uuid = *self.uuid.read().unwrap();
                format!("{n}[{uuid}]")
            },
            None => self.get_addr(),
        }
    }

    fn handle_packet(&self, reader: PacketReader) -> Result<(), PacketHandleError> {
        let state_ref = self.state.read().unwrap();

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
                Ok(self.handle_configuration_packet(reader)?)
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

                let mut connection_info = self.connection_info.write().unwrap();
                *connection_info = Some(ConnectionInfo {
                    protocol_version: packet.protocol_version,
                    server_address: packet.server_address,
                    server_port: packet.server_port,
                });

                let mut state = self.state.write().unwrap();
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
                *self.name.write().unwrap() = Some(packet.name);
                *self.uuid.write().unwrap() = packet.uuid;

                let connection_info_binding = self.connection_info.clone();
                let connection_info = connection_info_binding.read().unwrap();
                if let Some(ref connection_info) = *connection_info {
                    if connection_info.protocol_version != crate::PROTOCOL_VERSION {
                        if connection_info.protocol_version < crate::PROTOCOL_VERSION {
                            self.disconnect(format!("Your protocol version ({}) doesn't match server's protocol version ({}).\nClient out-of-date.", connection_info.protocol_version, crate::PROTOCOL_VERSION));
                        }
                        else {
                            self.disconnect(format!("Your protocol version ({}) doesn't match server's protocol version ({}).\nServer out-of-date.", connection_info.protocol_version, crate::PROTOCOL_VERSION));
                        }
                        
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

                let verify_token = self.verify_token.lock().unwrap().clone();
                match verify_token {
                    Some(verify_token) => {
                        let decrypted_verify_token = self.server_data.private_key.decrypt(Pkcs1v15Encrypt, &packet.verify_token).unwrap();

                        if *verify_token != decrypted_verify_token {
                            log!(warn, "Verify tokens for {} didn't match.", self.get_name());
                            self.disconnect("Verify tokens didn't match.".to_owned());
                            return Ok(());
                        }
                    }
                    None => {
                        log!(error, "{} sent Encryption Response, but a verify token wasn't saved for them.", self.get_name());
                        self.disconnect("Failure to set up encryption".to_owned());
                        return Ok(());
                    }
                };

                let shared_secret = self.server_data.private_key.decrypt(Pkcs1v15Encrypt, &packet.shared_secret).unwrap();

                *self.verify_token.lock().unwrap() = None;

                // turn on encryption
                let (encryptor, decryptor) = aes_util::initialize(&shared_secret);
                *self.encryption_setting.lock().unwrap() = EncryptionSetting::Encrypted(Box::new(encryptor), Box::new(decryptor));

                log!(verbose, "Encryption with {} is set up.", self.get_name());

                if CONFIG.server.online_mode {
                    // Authenticate
                    log!(verbose, "Authenticating {}...", self.get_name());

                    let public_key_der = self.server_data.public_key.to_public_key_der().unwrap();
                    let username = self.name.read().unwrap().clone();

                    if let Some(username) = username {
                        match authenticate_player(username.to_owned(), &shared_secret, public_key_der.as_bytes()) {
                            Ok(response) => {
                                let uuid = Uuid::parse_str(&response.id).unwrap();
                                *self.uuid.write().unwrap() = uuid;

                                log!(verbose, "Authentication for {} succeeded!", self.get_name());

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
                                };

                                self.send_packet_bytes(&login_success_packet.build());
                            },
                            Err(e) => {
                                log!(error, "Failed to authenticate player {}: {}", self.get_name(), e);
                                self.disconnect("Failed to authenticate".to_owned());
                                return Ok(());
                            },
                        }   
                    }
                    else {
                        log!(error, "Client {} sent Encryption Response before Login Start", self.get_name());
                        self.disconnect("Failed to authenticate".to_owned());
                        return Ok(());
                    }
                }
                else {
                    // Authentication skipped (offline mode)
                    let uuid = *self.uuid.read().unwrap();
                    let username = (*self.name.read().unwrap().clone().unwrap()).to_string();

                    let login_success_packet = LoginClientboundLoginSuccess {
                        uuid,
                        username,
                        properties: Vec::new(),
                    };

                    self.send_packet_bytes(&login_success_packet.build());
                }
            }
            0x03 => {
                *self.state.write().unwrap() = ConnectionState::Configuration;
                log!(verbose, "Client {} reached Login Acknowledged!!!", self.get_name());
            }
            _ => return Err(PacketHandleError::BadId(reader.id()))
        }

        Ok(())
    }

    fn handle_configuration_packet(&self, mut reader: PacketReader) -> Result<(), PacketHandleError> {
        match reader.id() {
            0x00 => {
                let packet = ConfigurationServerboundClientInformation::read(&mut reader)?;

                log!(debug, "Client information for {}:", self.get_name());
                log!(debug, "\tLocale: {}", packet.locale);
                log!(debug, "\tView distance: {}", packet.view_distance);
                log!(debug, "\tChat mode: {}", packet.chat_mode);
                log!(debug, "\tChat colors: {}", packet.chat_colors);
                log!(debug, "\tDisplayed skin parts:");
                log!(debug, "\t\tCape: {}", packet.displayed_skin_parts.cape_enabled);
                log!(debug, "\t\tJacket: {}", packet.displayed_skin_parts.jacket_enabled);
                log!(debug, "\t\tLeft sleeve: {}", packet.displayed_skin_parts.left_sleeve_enabled);
                log!(debug, "\t\tRight sleeve: {}", packet.displayed_skin_parts.right_sleeve_enabled);
                log!(debug, "\t\tLeft pants: {}", packet.displayed_skin_parts.left_pants_enabled);
                log!(debug, "\t\tRight pants: {}", packet.displayed_skin_parts.right_pants_enabled);
                log!(debug, "\t\tHat: {}", packet.displayed_skin_parts.hat_enabled);
                log!(debug, "\tMain hand: {}", packet.main_hand);
                log!(debug, "\tEnable text filtering: {}", packet.enable_text_filtering);
                log!(debug, "\tAllow server listings: {}", packet.allow_server_listings);

                // Send server brand to client
                let server_brand = CONFIG.status.version_prefix.clone()
                    + " (https://github.com/SuperNeon4ik/rustcraft_server)";
                let mut server_brand_bytes = BytesMut::new();
                write_string(&mut server_brand_bytes, &server_brand);

                let server_brand_packet = ConfigurationClientboundPluginMessage {
                    channel: Identifier::new(None, "brand").unwrap(),
                    data: server_brand_bytes.to_vec(),
                };

                self.send_packet_bytes(&server_brand_packet.build());

                // Finish configuration
                let finish_configuration_packet = ConfigurationClientboundFinishConfiguration {};
                self.send_packet_bytes(&finish_configuration_packet.build())
            },
            0x02 => {
                let packet = ConfigurationServerboundPluginMessage::read(&mut reader)?;
                log!(debug, "Recieved plugin message at '{}' ({} bytes): {:x?}", packet.channel, packet.data.len(), packet.data);

                if packet.channel.to_string() == "minecraft:brand" {
                    let brand = String::from_utf8(packet.data).unwrap();
                    log!(verbose, "{}'s brand is '{}'", self.get_name(), brand);
                }
            }
            0x03 => {
                *self.state.write().unwrap() = ConnectionState::Play;
                log!(verbose, "Client {} reached Configuration Acknowledged!!!", self.get_name());

                // We reached PLAY!!!
                // now let's find out for how long can we gaslight the client for into thinking
                // that we are a real server
                let login_packet = PlayClientboundLogin {
                    entity_id: 1,
                    is_hardcode: false,
                    dimention_names: vec![Identifier::from_string("minecraft:world").unwrap()],
                    max_players: 1,
                    view_distance: 12,
                    simulation_distance: 12,
                    reduced_debug_info: false,
                    enable_respawn_screen: true,
                    do_limited_crafting: false,
                    dimention_type: todo!(), // what?
                    dimention_name: Identifier::from_string("minecraft:world").unwrap(),
                    hashed_seed: 0,
                    gamemode: 1,
                    previous_gamemode: -1,
                    is_debug: false,
                    is_flat: true,
                    death_location: None,
                    portal_cooldown: 10,
                    enforces_secure_chat: false,
                };

                self.send_packet_bytes(&login_packet.build());
            }
            _ => return Err(PacketHandleError::BadId(reader.id()))
        }

        Ok(())
    }

    fn disconnect(&self, reason: String) {
        let connection_state = self.state.read().unwrap().clone();
        match connection_state {
            ConnectionState::Login => {
                let login_disconnect_packet = LoginClientboundDisconnect::from_string(reason);
                self.send_packet_bytes(&login_disconnect_packet.build());
            },
            ConnectionState::Configuration => {
                let config_disconnect_packet = ConfigurationClientboundDisconnect::from_string(reason);
                self.send_packet_bytes(&config_disconnect_packet.build());
            }
            _ => log!(error, "Invalid state ({}) while sending disconnect packet.", connection_state)
        }
    }

    fn generate_verify_token(size: usize) -> Vec<u8> {
        let mut rng = thread_rng();
        (0..size).map(|_| rng.gen()).collect()
    }
}