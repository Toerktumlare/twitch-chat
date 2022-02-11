use std::error::Error;
use std::{
    thread::{self, JoinHandle},
    time::Duration,
};

use crossbeam::channel::{unbounded, Receiver, SendError};
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};

pub struct EventHandler {}

impl EventHandler {
    pub fn new() -> Self {
        Self {}
    }
}

pub enum Action {
    Clear,
    Exit,
}

impl EventHandler {
    pub fn run(&self) -> (Receiver<Action>, JoinHandle<()>) {
        let (tx, rx) = unbounded();
        let join_handle = thread::spawn(move || loop {
            if let Ok(event) = read() {
                match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::NONE,
                    }) => tx.send(Action::Exit).unwrap_or(()),
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::NONE,
                    }) => tx.send(Action::Clear).unwrap_or(()),
                    _ => break,
                };
            }
        });
        (rx, join_handle)
    }
}
