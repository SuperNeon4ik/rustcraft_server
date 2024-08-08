use crate::{log, network::connection::Connection, LOGGER};
use std::{io::Read, net::{Shutdown, TcpListener, TcpStream}, thread};

pub struct MinecraftServer {
    address: String,
}

impl MinecraftServer {
    pub fn new(addr: &str) -> Self {
        MinecraftServer {
            address: addr.to_owned(),
        }
    }

    pub fn start_listening(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.address)?;

        let server_address = listener.local_addr()?;

        log!(info, "Listening on {}:{}", server_address.ip(), server_address.port());

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let address = stream.peer_addr().unwrap();
                    log!(info, "Received a connection: {}:{}", address.ip(), address.port());

                    let conn = Connection::new(stream);
                    thread::spawn(move || { 
                        conn.start_reading();
                    });
                }
                Err(e) => log!(warn, "Failed to read incoming stream: {}", e)
            }
        }

        Ok(())
    }
}
