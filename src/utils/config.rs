use crate::LOGGER;
use std::{fs::{self, OpenOptions}, io::Write, path::Path};

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub status: StatusConfig,
}

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    pub ip: String,
    pub port: u16,
}

#[derive(Serialize, Deserialize)]
pub struct StatusConfig {
    pub version_prefix: String,
    pub max_players: i32,
    pub motd: String,
}

pub fn read_config(filename: &str) -> Option<Config> {
    let data = match fs::read_to_string(filename) {
        Ok(d) => d,
        Err(_) => return None
    };

    let config = match toml::from_str(&data) {
        Ok(conf) => conf,
        Err(e) => {
            log!(error, "Failed to parse {} file: {}", filename, e);
            return None;
        }
    };

    Some(config)
}

pub fn write_default_config(filename: &str) -> bool {
    if Path::new(filename).exists() { return true; }

    let default_config = Config {
        server: ServerConfig { 
            ip: String::from("127.0.0.1"), 
            port: 25565,
        },
        status: StatusConfig { 
            version_prefix: String::from("Rusty"),
            motd: String::from("Rusty experimental minecraft server!"), 
            max_players: 69, 
        }
    };

    let data = match toml::to_string_pretty(&default_config) {
        Ok(d) => d,
        Err(e) => {
            log!(error, "Failed to serialize default config: {}", e);
            return false;
        }
    };

    let mut file = match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(filename) {
            Ok(f) => f,
            Err(e) => {
                log!(error, "Failed to open config file: {}", e);
                return false;
            }
        };

    file.write_all(data.as_bytes()).is_ok()
}