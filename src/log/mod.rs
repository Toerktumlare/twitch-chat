use std::{
    fmt::Display,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    sync::Once,
};

use chrono::{DateTime, Local, Utc};

use crate::log::logfactory::Logger;

pub mod file_appender;
pub mod logfactory;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct LogEvent {
    timestamp: DateTime<Utc>,
    log_level: LogLevel,
    thread_name: String,
    logger_name: String,
    message: String,
}

impl Display for LogEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "| {} | {} | {} | {} | {}",
            self.timestamp.with_timezone(&Local).format("%H:%M:%S"),
            self.log_level,
            self.thread_name,
            self.logger_name,
            self.message,
        )
    }
}
impl LogEvent {
    pub(crate) fn new(
        now: DateTime<Utc>,
        thread_name: String,
        log_level: LogLevel,
        logger_name: String,
        message: String,
    ) -> Self {
        Self {
            timestamp: now,
            thread_name,
            log_level,
            logger_name,
            message,
        }
    }
}

pub(crate) enum LogEvents {
    LogEvent(LogEvent),
    Terminate,
}

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
    static ONCE: Once = Once::new();
    static mut LOGGER: MaybeUninit<SingletonLogger> = MaybeUninit::uninit();

    unsafe {
        ONCE.call_once(|| {
            let logger = SingletonLogger {
                inner: Logger::new("root"),
            };
            LOGGER.write(logger);
        });
        LOGGER.assume_init_ref()
    }
}
