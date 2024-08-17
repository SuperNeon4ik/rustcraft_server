use crate::{log, network::connection::Connection, LOGGER, CONFIG};
use std::{net::TcpListener, thread};

pub struct MinecraftServer {
    address: String,
}

impl MinecraftServer {
    pub fn new(ip: &str, port: u16) -> Self {
        MinecraftServer {
            address: ip.to_owned() + ":" + &port.to_string(),
        }
    }

    pub fn start_listening(&self) {
        if !CONFIG.server.online_mode { log!(warn, "> Server is running in OFFLINE mode. ") }

        let listener = TcpListener::bind(&self.address).unwrap();

        let server_address = listener.local_addr().unwrap();

        log!(info, "Listening on {}:{}", server_address.ip(), server_address.port());

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let address = stream.peer_addr().unwrap();
                    log!(verbose, "Received a connection: {}:{}", address.ip(), address.port());

                    let conn = Connection::new(stream);
                    thread::spawn(move || { 
                        conn.start_reading();
                    });
                }
                Err(e) => log!(warn, "Failed to read incoming stream: {}", e)
            }
        }
    }
}
