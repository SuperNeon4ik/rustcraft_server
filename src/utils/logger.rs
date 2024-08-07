use std::{fs::{self, OpenOptions}, io::Write, sync::Mutex};

use chrono::Local;
use colored::Colorize;

pub struct Logger {
    file: Mutex<fs::File>,
}

pub enum LogLevel {
    Error,
    Warn,
    Info,
    Verbose,
    Debug,
}

impl LogLevel {
    pub fn to_string(&self) -> String {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Verbose => "VERBOSE",
            Self::Debug => "DEBUG",
        }.to_owned()
    }
}

impl Logger {
    pub fn new(log_file: &str) -> Self {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_file)
            .expect("Unable to open log file");

        Logger {
            file: Mutex::new(file),
        }
    }

    pub fn log(&self, level: LogLevel, source: &str, text: &str) {
        let date = Local::now();
        let formatted_text = format!("[{}] [{}] [{}] {}\n", date.format("%Y-%m-%d %H:%M:%S"), level.to_string(), source, text);

        let mut file = self.file.lock().unwrap();
        file.write_all(formatted_text.as_bytes()).expect("Unable to write to log file");

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