use std::{
    fmt::Display,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    sync::Once,
};

use chrono::{DateTime, Local, Utc};

use crate::log::logger::Logger;

pub mod file_appender;
pub mod logger;

pub static ONCE: std::sync::Once = Once::new();
pub static mut LOGGER: MaybeUninit<SingletonLogger> = MaybeUninit::uninit();

#[derive(Debug, Eq, Copy, Clone, PartialEq, PartialOrd)]
pub enum LogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let log_level = match *self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => " INFO",
            LogLevel::Warn => " WARN",
            LogLevel::Error => "ERROR",
        };
        write!(f, "{}", log_level)
    }
}

#[derive(Debug)]
pub struct LogEvent {
    timestamp: DateTime<Utc>,
    log_level: LogLevel,
    thread_name: String,
    type_name: String,
    message: String,
}

impl Display for LogEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "| {} | {} | {} | {} | {}",
            self.timestamp
                .with_timezone(&Local)
                .format("%Y-%m-%d %H:%M:%S%.3f"),
            self.log_level,
            self.thread_name,
            self.type_name,
            self.message,
        )
    }
}
impl LogEvent {
    pub(crate) fn new(
        now: DateTime<Utc>,
        thread_name: String,
        log_level: LogLevel,
        type_name: String,
        message: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: now,
            thread_name,
            log_level,
            type_name,
            message: message.into(),
        }
    }
}

pub(crate) enum LogEvents {
    LogEvent(LogEvent),
}
#[derive(Debug)]
pub struct SingletonLogger {
    inner: Logger,
}

impl Deref for SingletonLogger {
    type Target = Logger;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SingletonLogger {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub fn get_logger() -> &'static SingletonLogger {
    unsafe {
        ONCE.call_once(|| {
            let logger = SingletonLogger {
                inner: Logger::new(),
            };
            LOGGER.write(logger);
        });
        LOGGER.assume_init_ref()
    }
}

pub fn get_logger_mut() -> &'static mut SingletonLogger {
    unsafe {
        ONCE.call_once(|| {
            let logger = SingletonLogger {
                inner: Logger::new(),
            };
            LOGGER.write(logger);
        });
        LOGGER.assume_init_mut()
    }
}
