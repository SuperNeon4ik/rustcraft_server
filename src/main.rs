use std::thread;

use crossbeam_channel::{bounded, select, Receiver};
use logger::Logger;
use rcserver::MinecraftServer;

mod rcserver;
mod logger;

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

fn main() -> std::io::Result<()> {
    let logger = Logger::new("Main");

    logger.info(&format!("RustCraft Server starting..."));
    logger.info(&format!("Ctrl+C to exit"));

    let server = MinecraftServer::new("127.0.0.1:25565");

    thread::spawn(move || { server.start_listening() });

    let ctrl_c_events = ctrl_channel().unwrap();

    loop {
        select! {
            recv(ctrl_c_events) -> _ => {
                drop(logger);
                print!("");
                println!("Goodbye!");
                break;
            }
        }
    }

    Ok(())
}