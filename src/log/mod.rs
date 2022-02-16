use std::fmt::Display;

use chrono::{DateTime, Local, Utc};

pub mod file_appender;
pub mod logfactory;

pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let log_level = match *self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        };
        write!(f, "{}", log_level)
    }
}

pub struct LogEvent {
    timestamp: DateTime<Utc>,
    log_level: LogLevel,
    thread_name: String,
    message: String,
}

impl Display for LogEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "| {} | {} | {} | {}",
            self.timestamp.with_timezone(&Local).format("%H:%M:%S"),
            self.thread_name,
            self.log_level,
            self.message,
        )
    }
}
impl LogEvent {
    pub(crate) fn new(
        now: DateTime<Utc>,
        thread_name: String,
        log_level: LogLevel,
        message: String,
    ) -> Self {
        // let message = format!("{}{}", message, "\n");
        Self {
            timestamp: now,
            thread_name,
            log_level,
            message,
        }
    }
}
