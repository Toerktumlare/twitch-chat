use std::{
    any::type_name,
    error::Error,
    ops::{Deref, DerefMut},
    thread::{self, JoinHandle},
    time::Duration,
};

use crossbeam::channel::{unbounded, Receiver, Sender};
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::log::get_logger;

pub struct EventHandler {
    pub receiver: Receiver<Action>,
    worker: EventWorker,
}

pub enum Action {
    Clear,
    Exit,
}

impl EventHandler {
    pub fn new() -> EventHandler {
        let (sender, receiver) = unbounded();
        let worker = EventWorker::run("event_worker", sender).unwrap();
        Self { receiver, worker }
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        let log = get_logger();
        log.debug("Closing down EventHandler", type_name::<EventHandler>());
        self.sender.send(Action::Exit).unwrap();
        self.thread.take().map(JoinHandle::join);
        log.debug("EventHandler closed", type_name::<EventHandler>());
    }
}

pub struct EventWorker {
    name: String,
    thread: Option<JoinHandle<()>>,
    sender: Sender<Action>,
}

impl EventWorker {
    pub fn run(
        name: impl Into<String>,
        sender: Sender<Action>,
    ) -> Result<EventWorker, Box<dyn Error>> {
        let (tx, rx) = unbounded();
        let name = name.into();
        let handle = thread::Builder::new()
            .name(name.clone())
            .spawn(move || loop {
                if poll(Duration::from_millis(100)).unwrap() {
                    match read().unwrap() {
                        Event::Key(KeyEvent {
                            code: KeyCode::Char('q'),
                            modifiers: KeyModifiers::NONE,
                        }) => sender.send(Action::Exit).unwrap(),
                        Event::Key(KeyEvent {
                            code: KeyCode::Char('c'),
                            modifiers: KeyModifiers::NONE,
                        }) => sender.send(Action::Clear).unwrap_or(()),
                        _ => (),
                    };
                }
                if let Ok(Action::Exit) = rx.try_recv() {
                    break;
                }
            })
            .unwrap();

        Ok(EventWorker {
            name,
            thread: Some(handle),
            sender: tx,
        })
    }
}

impl Deref for EventHandler {
    type Target = EventWorker;

    fn deref(&self) -> &Self::Target {
        &self.worker
    }
}

impl DerefMut for EventHandler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.worker
    }
}
