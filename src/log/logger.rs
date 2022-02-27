use std::{
    any::type_name,
    io::Write,
    thread::{self, JoinHandle},
};

use chrono::Utc;
use crossbeam::channel::{unbounded, Receiver, Sender};

use super::{file_appender::FileAppender, get_logger, LogEvent, LogEvents, LogLevel};

struct LogWorker;

impl LogWorker {
    pub(crate) fn run(
        receiver: Receiver<LogEvents>,
        mut appenders: Vec<Box<dyn Write + Send + Sync>>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                match event {
                    LogEvents::LogEvent(data) => {
                        appenders.iter_mut().for_each(|appender| {
                            let event_text = format!("{}{}", data, "\n");
                            let _ = appender
                                .write(event_text.as_bytes())
                                .expect("could not write");

                            appender.flush().unwrap();
                        });
                    }
                }
            }
        })
    }
}

#[derive(Debug)]
pub struct Logger {
    pub log_level: LogLevel,
    sender: Option<Sender<LogEvents>>,
    log_worker: Option<JoinHandle<()>>,
}

impl Logger {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded::<LogEvents>();

        let mut appenders: Vec<Box<dyn Write + Send + Sync>> = Vec::new();
        let appender = FileAppender::new("twitch_chat.log");
        let file_appender = Box::new(appender);
        appenders.push(file_appender);

        let log_worker = LogWorker::run(receiver, appenders);

        Self {
            log_level: LogLevel::Info,
            sender: Some(sender),
            log_worker: Some(log_worker),
        }
    }

    pub fn set_level(&mut self, log_level: LogLevel) {
        self.log_level = log_level;
    }

    #[allow(dead_code)]
    pub fn trace(&self, message: impl Into<String>, type_name: impl Into<String>) {
        self.log(LogLevel::Trace, message.into(), type_name.into())
    }

    #[allow(dead_code)]
    pub fn debug(&self, message: impl Into<String>, type_name: impl Into<String>) {
        self.log(LogLevel::Debug, message.into(), type_name.into())
    }

    #[allow(dead_code)]
    pub fn info(&self, message: impl Into<String>, type_name: impl Into<String>) {
        self.log(LogLevel::Info, message.into(), type_name.into())
    }

    #[allow(dead_code)]
    pub fn warn(&self, message: impl Into<String>, type_name: impl Into<String>) {
        self.log(LogLevel::Warn, message.into(), type_name.into())
    }

    #[allow(dead_code)]
    pub fn error(&self, message: impl Into<String>, type_name: impl Into<String>) {
        self.log(LogLevel::Error, message.into(), type_name.into())
    }

    fn log(&self, log_level: LogLevel, message: String, type_name: impl Into<String>) {
        if log_level > self.log_level {
            return;
        }
        let log_event = LogEvent::new(
            Utc::now(),
            thread::current().name().unwrap().to_string(),
            log_level,
            type_name.into(),
            message,
        );
        if let Some(s) = self.sender.as_ref() {
            s.send(LogEvents::LogEvent(log_event)).unwrap()
        }
    }

    pub fn close(&mut self) {
        let log = get_logger();
        log.debug("Closing down logger", type_name::<Logger>());
        drop(self.sender.take());
        self.log_worker.take().map(JoinHandle::join);
    }
}
