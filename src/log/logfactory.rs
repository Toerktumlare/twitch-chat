#![allow(dead_code)]

use std::{io::Write, thread};

use chrono::Utc;
use crossbeam::channel::{unbounded, Receiver, Sender};

use super::{file_appender::FileAppender, LogEvent, LogLevel};
pub struct LogFactory;

impl LogFactory {
    pub fn get_logger(name: impl Into<String>) -> Logger {
        let (sender, receiver) = unbounded::<LogEvent>();

        let mut appenders: Vec<Box<dyn Write + Send + Sync>> = Vec::new();
        let appender = FileAppender::new("twitch_chat.log");
        let file_appender = Box::new(appender);
        appenders.push(file_appender);

        LogWorker::start(receiver, appenders);

        Logger::new(name, sender)
    }
}

struct LogWorker;

impl LogWorker {
    pub(crate) fn start(
        receiver: Receiver<LogEvent>,
        mut appenders: Vec<Box<dyn Write + Send + Sync>>,
    ) {
        thread::spawn(move || {
            let log_event = receiver.recv().unwrap();
            appenders.iter_mut().for_each(|a| {
                let _ = a.write(format!("{}", log_event).as_bytes()).unwrap();
                a.flush().unwrap();
            });
        });
    }
}

pub struct Logger {
    name: String,
    sender: Sender<LogEvent>,
}

impl Logger {
    pub fn new(name: impl Into<String>, sender: Sender<LogEvent>) -> Self {
        Self {
            name: name.into(),
            sender,
        }
    }

    pub fn trace(&self, message: impl Into<String>) {
        self.log(LogLevel::Trace, message.into())
    }

    pub fn debug(&self, message: impl Into<String>) {
        self.log(LogLevel::Debug, message.into())
    }
    pub fn info(&self, message: impl Into<String>) {
        self.log(LogLevel::Info, message.into())
    }
    pub fn warn(&self, message: impl Into<String>) {
        self.log(LogLevel::Warn, message.into())
    }
    pub fn error(&self, message: impl Into<String>) {
        self.log(LogLevel::Error, message.into())
    }

    fn log(&self, log_level: LogLevel, message: String) {
        let log_event = LogEvent::new(
            Utc::now(),
            thread::current().name().unwrap().to_string(),
            log_level,
            message,
        );
        self.sender.send(log_event).unwrap();
    }
}
