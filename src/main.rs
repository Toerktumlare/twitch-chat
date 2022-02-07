#![allow(dead_code)]
#![allow(unused_imports)]
use chrono::Local;
use crossbeam::channel::{select, unbounded};
use crossterm::style::Color;
use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use gui::{
    buffer::{Cell, Style},
    screen::Screen,
    window::Window,
    Pos, Size,
};
use std::{
    env,
    process::exit,
    thread::{self, Thread},
};
use std::{io::stdout, time::Duration};
use tungstenite::Message;

use crate::chat_message::ChatMessage;
use crate::twitch_client::TwitchClient;

mod chat_message;
mod gui;
mod twitch_client;

//TODO:
// - Borders around the window
// - chat channel argument
// - username argument
//
// autoresize... difficult... maybe next decade
// check how toggle solves events
// event a want:
//    - scroll up/down
//    - clear the screen
//    - quit application, more gracefully
// create a KeyEventHandler
// create proper key events (enum)
// EventHandler that takes in a MessageEventHandler, and a KeyEventHandler
// Handle different message type:
//    - PRIVMSG
//    - Meta information from Twitch (headers etc.)
//    - Error message?
// Handle a debug flag, which will print messages to the window
fn main() {
    let output = stdout();
    execute!(stdout(), EnterAlternateScreen).unwrap();
    let mut screen = Screen::new(output, Size::new(96, 32)).unwrap();
    let mut window = Window::new(Pos::new(0, 0), Size::new(96, 32));

    let token = env::var("TWITCH_BOT_TOKEN").unwrap_or_else(|_| {
        eprintln!("TWITCH_BOT_TOKEN env variable not set");
        exit(1);
    });

    let client = TwitchClient::new("ws://irc-ws.chat.twitch.tv:80", token);
    let (r1, _join_handle) = client.run();

    screen.enable_raw_mode().expect("could not enable raw mode");

    let (s, r2) = unbounded();
    let _join_handle2 = thread::spawn(move || loop {
        if poll(Duration::from_millis(100)).unwrap() {
            match read().unwrap() {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE,
                }) => s.send(Message::Ping(vec![1])).unwrap(),
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::NONE,
                }) => s.send(Message::Text("#clear".to_string())).unwrap(),
                _ => break,
            };
        }
    });

    loop {
        select! {
            recv(r1) -> msg => {
                let msg = msg.unwrap();
                if msg.to_text().unwrap().contains("PRIVMSG") {
                    if let Ok(message) = ChatMessage::parse(msg.to_text().unwrap()) {
                        window.print(&mut screen,
                            message.meta_data.tmi_sent_ts.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string(),
                            Style::none());
                        window.print(&mut screen, " | ", Style::none());
                        let (r, g, b) = message.meta_data.color.flatten().unwrap_or((15, 15, 15));
                        window.print(&mut screen, message.meta_data.display_name.unwrap(), Style::fg(Some(Color::Rgb {r, g, b})));
                        window.print(&mut screen, " | ", Style::none());
                        window.print(&mut screen, message.message.trim(), Style::none());
                        window.newline(&mut screen);
                    } else {
                        let message = msg.to_text().unwrap().to_string();
                        if !message.starts_with('@') {
                            let message = format!("{} | {}", Local::now().format("%Y-%m-%d %H:%M:%S"), message);
                            window.print(&mut screen, message, Style::none());
                            window.newline(&mut screen);
                        }
                    }
                }
                screen.render().unwrap();
            },
            recv(r2) -> msg => {
                let msg = msg.unwrap();
                match msg {
                    Message::Ping(_) => {
                        // Can't join handle since connection to twitch is not closed
                        // join_handle.join().unwrap();
                        // join_handle2.join().unwrap();
                        std::process::exit(0);
                    }
                    Message::Text(_) => {
                        window.clear(&mut screen);
                        screen.render().unwrap();
                    },
                    _ => break,
                }
            }
        }
    }
}
