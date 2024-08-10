use crate::{log, network::connection::Connection, LOGGER};
use std::{net::TcpListener, thread};

pub struct MinecraftServer {
    address: String,
}

impl MinecraftServer {
    pub fn new(addr: &str) -> Self {
        MinecraftServer {
            address: addr.to_owned(),
        }
    }

    pub fn start_listening(&self) {
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
