use std::{
    thread::{self, JoinHandle},
    time::Duration,
};

use crossbeam::channel::{unbounded, Receiver};
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
            if poll(Duration::from_millis(100)).unwrap() {
                match read().unwrap() {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::NONE,
                    }) => tx.send(Action::Exit).unwrap(),
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::NONE,
                    }) => tx.send(Action::Clear).unwrap(),
                    _ => break,
                };
            }
        });
        (rx, join_handle)
    }
}
