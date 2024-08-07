use chrono::Local;
use colored::Colorize;

#[derive(Clone)]
pub struct Logger {
    name: String,
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
    pub fn new(name: &str) -> Logger {
        Logger {
            name: name.to_owned()
        }
    }

    pub fn log(&self, level: LogLevel, text: &str) {
        let date = Local::now();
        let formatted_text = format!("[{}] [{}] [{}] {}", date.format("%x %X"), level.to_string(), self.name, text);

        let colored_text = match level {
            LogLevel::Error => formatted_text.red(),
            LogLevel::Warn => formatted_text.yellow(),
            LogLevel::Info => formatted_text.white(),
            LogLevel::Verbose => formatted_text.dimmed(),
            LogLevel::Debug => formatted_text.bright_blue(),
        };

        println!("{}", colored_text);
    }
    
    pub fn error(&self, text: &str) {
        self.log(LogLevel::Error, text)
    }

    pub fn warn(&self, text: &str) {
        self.log(LogLevel::Warn, text)
    }

    pub fn info(&self, text: &str) {
        self.log(LogLevel::Info, text)
    }

    pub fn verbose(&self, text: &str) {
        self.log(LogLevel::Verbose, text)
    }

    pub fn debug(&self, text: &str) {
        self.log(LogLevel::Debug, text)
    }
}