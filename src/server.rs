use rsa::RsaPublicKey;
use rsa::RsaPrivateKey;
use crate::crypto::rsa_util::generate_rsa_keypair;
use crate::{log, network::connection::Connection, LOGGER, CONFIG};
use std::{net::TcpListener, thread};

pub struct MinecraftServer {
    address: String,
    server_data: ServerData,
}

#[derive(Clone)]
pub struct ServerData {
    pub private_key: RsaPrivateKey,
    pub public_key: RsaPublicKey,
}

impl MinecraftServer {
    pub fn new(ip: &str, port: u16) -> Self {
        log!(info, "Generating RSA keypair...");
        let keypair = generate_rsa_keypair();

        MinecraftServer {
            address: ip.to_owned() + ":" + &port.to_string(),
            server_data: ServerData { 
                private_key: keypair.0, 
                public_key: keypair.1
            }
        }
    }

    pub fn start_listening(&self) {
        if CONFIG.server.online_mode { log!(verbose, "SESSION_HOST = '{}'", crate::SESSION_HOST) }
        else { log!(warn, "> Server is running in OFFLINE mode. ") }

        let listener = TcpListener::bind(&self.address).unwrap();
        let server_address = listener.local_addr().unwrap();

        log!(info, "Listening on {}:{}", server_address.ip(), server_address.port());

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let address = stream.peer_addr().unwrap();
                    log!(verbose, "Received a connection: {}:{}", address.ip(), address.port());

                    let mut conn = Connection::new(stream, &self.server_data);
                    thread::spawn(move || { 
                        conn.start_reading();
                    });
                }
                Err(e) => log!(warn, "Failed to read incoming stream: {}", e)
            }
        }
    }
}
