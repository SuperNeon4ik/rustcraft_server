#![allow(unused)]
mod crypto;
mod custom_types;
mod network;
mod utils;
mod server;

use std::thread;

use chrono::Local;
use crossbeam_channel::{bounded, select, Receiver};
use once_cell::sync::Lazy;
use utils::{config::{read_config, write_default_config, Config}, logger::{LogLevel, Logger}};
use server::MinecraftServer;

pub const VERSION: &str = "1.21";
pub const PROTOCOL_VERSION: i32 = 767;
pub const SESSION_HOST: &str = "https://sessionserver.mojang.com";

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    println!("Loading config.toml...");
    if write_default_config("config.toml") {
        println!("Created default config file!");
    }

    read_config("config.toml").expect("Config file missing.")
});

pub static LOGGER: Lazy<Logger> = Lazy::new(|| {
    let mut logger = Logger::new(&format!("logs/{}.log", Local::now().format("%Y-%m-%d-%H-%M-%S")), LogLevel::Info);
    let level = CONFIG.misc.log_level.clone();
    logger.set_level(level);
    logger
});

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

fn main() -> std::io::Result<()> {
    log!(info, "RustCraft Server ({} {}; Protocol {}) starting...", CONFIG.status.version_prefix, VERSION, PROTOCOL_VERSION);
    log!(info, "Ctrl+C to exit");

    let server = MinecraftServer::new(&CONFIG.server.ip, CONFIG.server.port);

    thread::spawn(move || { server.start_listening() });

    let ctrl_c_events = ctrl_channel().unwrap();

    select! {
        recv(ctrl_c_events) -> _ => {
            println!();
            log!(info, "Server closing...");
        }
    }

    println!("Goodbye!");

    Ok(())
}