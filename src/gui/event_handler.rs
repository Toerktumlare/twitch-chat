use std::thread::{self, JoinHandle};

use crossbeam::channel::{unbounded, Receiver};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::log::get_logger;

pub struct EventHandler {
    pub receiver: Receiver<Action>,
    thread: JoinHandle<()>,
}

pub enum Action {
    Clear,
    Exit,
}

impl EventHandler {
    pub fn new() -> EventHandler {
        let (sender, receiver) = unbounded();
        let _log = get_logger();
        let join_handle = thread::spawn(move || loop {
            if let Ok(event) = read() {
                match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::NONE,
                    }) => sender.send(Action::Exit).unwrap(),
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::NONE,
                    }) => sender.send(Action::Clear).unwrap_or(()),
                    _ => break,
                };
            }
        });
        Self {
            receiver,
            thread: join_handle,
        }
    }
}
