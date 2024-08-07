use std::{io::{BufReader, Read}, net::{Shutdown, TcpListener, TcpStream}, thread};

pub struct MinecraftServer {
    address: String,
}

impl MinecraftServer {
    pub fn new(addr: &str) -> MinecraftServer {
        MinecraftServer {
            address: addr.to_owned()
        }
    }

    pub fn start_listening(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.address)?;

        let server_address = listener.local_addr()?;

        println!("Listening on {}:{}", server_address.ip(), server_address.port());

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let address = stream.peer_addr().unwrap();
                    println!("Received a connection: {}:{}", address.ip(), address.port());
                    thread::spawn(move || { 
                        handle_connection(stream);
                    });
                }
                Err(e) => println!("Failed to read incoming stream: {}", e)
            }
        }

        Ok(())
    }
}

fn handle_connection(stream: TcpStream) {
    let address = stream.peer_addr().unwrap();

    let mut buf = BufReader::new(stream.try_clone().unwrap());
    loop {
        let mut bytes:Vec<u8> = Vec::new();
        match buf.read_to_end(&mut bytes) {
            Ok(b)  => {
                if b == 0 { break; }
                println!("Received data ({} bytes): {}", bytes.len(), hex::encode(bytes));
            }
            Err(e) => println!("Error receiving data: {}", e)
        }
    }

    println!("Client {}:{} dropped", address.ip(), address.port());
    stream.shutdown(Shutdown::Both).unwrap();
}