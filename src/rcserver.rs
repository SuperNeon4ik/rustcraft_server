use std::net::TcpListener;

pub struct MinecraftServer {
    address: String
}

impl MinecraftServer {
    pub fn new(addr: &str) -> MinecraftServer {
        MinecraftServer {
            address: addr.to_owned()
        }
    }

    pub fn start_listening(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.address)?;

        println!("Listening on {}:{}", listener.local_addr()?.ip(), listener.local_addr()?.port());

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("Received a connection: {}:{}", stream.peer_addr().unwrap().ip(), stream.peer_addr().unwrap().port());
                    stream.shutdown(std::net::Shutdown::Both).unwrap();
                }
                Err(e) => println!("Failed to read incoming stream: {}", e)
            }
        }

        drop(listener);
        Ok(())
    }
}