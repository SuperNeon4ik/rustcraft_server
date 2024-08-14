use core::fmt;
use std::{fs::{self, OpenOptions}, io::Write, path::Path, sync::Mutex};

use chrono::Local;
use colored::Colorize;
use serde_derive::{Deserialize, Serialize};

pub struct Logger {
    file: Mutex<fs::File>,
    level: LogLevel
}

#[derive(PartialEq, PartialOrd, Serialize, Deserialize, Clone)]
pub enum LogLevel {
    Error = 4,
    Warn = 3,
    Info = 2,
    Verbose = 1,
    Debug = 0,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let level_str = match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Verbose => "VERBOSE",
            Self::Debug => "DEBUG",
        };

        write!(f, "{}", level_str)
    }
}

impl Logger {
    pub fn new(log_file: &str, level: LogLevel) -> Self {
        let parent = Path::new(log_file).parent().unwrap();
        fs::create_dir_all(parent).unwrap();

        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_file)
            .expect("Unable to open log file");

        Logger {
            file: Mutex::new(file),
            level
        }
    }

    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    pub fn log(&self, level: LogLevel, source: &str, text: &str) {
        if level < self.level { return; }

        let date = Local::now();
        let formatted_text = if self.level <= LogLevel::Verbose {
            format!("[{}] [{}] [{}] {}\n", date.format("%Y-%m-%d %H:%M:%S"), level, source, text)
        } else {
            format!("[{}] [{}] {}\n", date.format("%Y-%m-%d %H:%M:%S"), level, text)
        };

        let mut file_lock = self.file.lock().unwrap();
        file_lock.write_all(formatted_text.as_bytes()).expect("Unable to write to log file");

        let colored_text = match level {
            LogLevel::Error => formatted_text.red(),
            LogLevel::Warn => formatted_text.yellow(),
            LogLevel::Info => formatted_text.white(),
            LogLevel::Verbose => formatted_text.dimmed(),
            LogLevel::Debug => formatted_text.bright_blue(),
        };

        print!("{}", colored_text);
    }
    
    pub fn error(&self, source: &str, text: &str) {
        self.log(LogLevel::Error, source, text)
    }

    pub fn warn(&self, source: &str, text: &str) {
        self.log(LogLevel::Warn, source, text)
    }

    pub fn info(&self, source: &str, text: &str) {
        self.log(LogLevel::Info, source, text)
    }

    pub fn verbose(&self, source: &str, text: &str) {
        self.log(LogLevel::Verbose, source, text)
    }

    pub fn debug(&self, source: &str, text: &str) {
        self.log(LogLevel::Debug, source, text)
    }
}