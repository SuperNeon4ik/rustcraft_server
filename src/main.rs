#[macro_use]
extern crate lazy_static;

mod server;
mod utils;

use std::thread;

use crossbeam_channel::{bounded, select, Receiver};
use utils::logger::Logger;
use server::MinecraftServer;

lazy_static! {
    static ref LOGGER: Logger = Logger::new_with_file("server.log");
}

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

fn main() -> std::io::Result<()> {
    log!(info, "RustCraft Server starting...");
    log!(info, "Ctrl+C to exit");

    let server = MinecraftServer::new("127.0.0.1:25565");

    thread::spawn(move || { server.start_listening() });

    let ctrl_c_events = ctrl_channel().unwrap();

    loop {
        select! {
            recv(ctrl_c_events) -> _ => {
                println!("");
                log!(info, "Server closing...");
                break;
            }
        }
    }

    println!("Goodbye!");

    Ok(())
}