#![allow(dead_code)]
#![allow(unused_imports)]
use crossbeam::channel::{select, unbounded};
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
// - Print correct timezone
// - Username colors
// - chat channel argument
// - username argument
//
// autoresize... difficult... maybe next decade
// make sure all messages from twitch are sent through to the gui thread
// check how toogle solves events
// event a want:
//    - scroll up/down
//    - clear the screen
//    - quit application
// create a KeyEventHandler
// EventHandler that takes in a MessageEventHandler, and a KeyEventHandler
// Handle different message type:
//    - PRIVMSG
//    - Meta information from Twitch (headers etc.)
//    - Error message?
// Handle a debug flag, which will print messages to the window
fn main() {
    let output = stdout();
    execute!(stdout(), EnterAlternateScreen).unwrap();
    let mut screen = Screen::new(output, Size::new(96, 10)).unwrap();
    let mut window = Window::new(Pos::new(0, 0), Size::new(96, 10));

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
                }) => {
                    s.send(Message::Ping(vec![1])).unwrap();
                    break;
                }
                _ => break,
            };
        }
    });

    loop {
        select! {
            recv(r1) -> msg => {
                let msg = msg.unwrap();
                if msg.to_text().unwrap().contains("PRIVMSG") {
                    let message = ChatMessage::parse(msg.to_text().unwrap());
                    let message = format!(
                        "{} | {}: {}",
                        message.meta_data.tmi_sent_ts.format("%Y-%m-%d %H:%M:%S").to_string(),
                        message.meta_data.display_name.unwrap(),
                        message.message.trim()
                    );
                    window.newline(&mut screen);
                    window.print(
                        &mut screen,
                        message,
                        Style::none(),
                    );
                    screen.render().unwrap();
                }
            },
            recv(r2) -> _ => {
                std::process::exit(0);
            }
        }
    }
}
