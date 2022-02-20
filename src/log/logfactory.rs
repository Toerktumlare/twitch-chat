use std::{
    io::Write,
    thread::{self, JoinHandle},
    time::Duration,
};

use chrono::Utc;
use crossbeam::channel::{unbounded, Receiver, Sender};

use super::{file_appender::FileAppender, LogEvent, LogEvents, LogLevel, SingletonLogger};

struct LogWorker;

impl LogWorker {
    pub(crate) fn run(
        receiver: Receiver<LogEvents>,
        mut appenders: Vec<Box<dyn Write + Send + Sync>>,
    ) -> JoinHandle<()> {
        thread::spawn(move || loop {
            match receiver.recv() {
                Ok(event) => {
                    if let LogEvents::LogEvent(event) = event {
                        appenders.iter_mut().for_each(|appender| {
                            let event_text = format!("{}{}", event, "\n");
                            let _ = appender
                                .write(event_text.as_bytes())
                                .expect("could not write");

                            if appender.flush().is_err() {
                                // log.error("Could not flush");
                            }
                        });
                    } else {
                        break;
                    }
                }
                Err(_) => {
                    // log.error("could not receive message");
                    break;
                }
            }
        })
    }
}

pub struct Logger {
    name: String,
    sender: Sender<LogEvents>,
    log_worker: Option<JoinHandle<()>>,
}

impl Logger {
    pub fn new(name: impl Into<String>) -> Self {
        let (sender, receiver) = unbounded::<LogEvents>();

        let mut appenders: Vec<Box<dyn Write + Send + Sync>> = Vec::new();
        let appender = FileAppender::new("twitch_chat.log");
        let file_appender = Box::new(appender);
        appenders.push(file_appender);

        let log_worker = LogWorker::run(receiver, appenders);

        Self {
            name: name.into(),
            sender,
            log_worker: Some(log_worker),
        }
    }

    #[allow(dead_code)]
    pub fn trace(&self, message: impl Into<String>) {
        self.log(LogLevel::Trace, message.into())
    }

    #[allow(dead_code)]
    pub fn debug(&self, message: impl Into<String>) {
        self.log(LogLevel::Debug, message.into())
    }

    #[allow(dead_code)]
    pub fn info(&self, message: impl Into<String>) {
        self.log(LogLevel::Info, message.into())
    }

    #[allow(dead_code)]
    pub fn warn(&self, message: impl Into<String>) {
        self.log(LogLevel::Warn, message.into())
    }

    #[allow(dead_code)]
    pub fn error(&self, message: impl Into<String>) {
        self.log(LogLevel::Error, message.into())
    }

    fn log(&self, log_level: LogLevel, message: String) {
        let log_event = LogEvent::new(
            Utc::now(),
            thread::current().name().unwrap().to_string(),
            log_level,
            self.name.clone(),
            message,
        );
        self.sender.send(LogEvents::LogEvent(log_event)).unwrap();
    }
}

impl Drop for SingletonLogger {
    fn drop(&mut self) {
        println!("we are dropping");
        self.sender.send(LogEvents::Terminate).unwrap();
        thread::sleep(Duration::from_millis(10));
        if let Some(thread) = self.log_worker.take() {
            thread.join().unwrap();
        }
    }
}
