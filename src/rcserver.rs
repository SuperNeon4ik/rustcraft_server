use crate::logger::Logger;

use std::{io::{BufReader, Read}, net::{Shutdown, TcpListener, TcpStream}, thread};


pub struct MinecraftServer {
    address: String,
    logger: Logger
}

impl MinecraftServer {
    pub fn new(addr: &str) -> MinecraftServer {
        MinecraftServer {
            address: addr.to_owned(),
            logger: Logger::new("MinecraftServer")
        }
    }

    pub fn start_listening(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.address)?;

        let server_address = listener.local_addr()?;

        self.logger.info(&format!("Listening on {}:{}", server_address.ip(), server_address.port()));

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let address = stream.peer_addr().unwrap();
                    self.logger.verbose(&format!("Received a connection: {}:{}", address.ip(), address.port()));
                    let logger = self.logger.clone();
                    thread::spawn(move || { 
                        handle_connection(logger, stream);
                    });
                }
                Err(e) => self.logger.warn(&format!("Failed to read incoming stream: {}", e))
            }
        }

        Ok(())
    }
}

fn handle_connection(logger: Logger, stream: TcpStream) {
    let address = stream.peer_addr().unwrap();

    let mut buf = BufReader::new(stream.try_clone().unwrap());
    loop {
        let mut bytes:Vec<u8> = Vec::new();
        match buf.read_to_end(&mut bytes) {
            Ok(b)  => {
                if b == 0 { break; }
                logger.debug(&format!("Received data ({} bytes): {}", bytes.len(), hex::encode(bytes)));
            }
            Err(e) => logger.warn(&format!("Error receiving data: {}", e))
        }
    }

    logger.verbose(&format!("Client {}:{} dropped", address.ip(), address.port()));
    stream.shutdown(Shutdown::Both).unwrap();
}