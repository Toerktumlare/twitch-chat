#![allow(dead_code)]
#![allow(unused_imports)]
use chrono::Local;
use crossbeam::channel::{select, unbounded};
use crossterm::style::Color;
use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{size, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use gui::{
    buffer::{Cell, Style},
    event_handler::{Action, EventHandler},
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

use crate::chat_message::ChatMessage;
use crate::twitch_client::{Message, TwitchClient};

mod chat_message;
mod color_gen;
mod gui;
mod twitch_client;

static CHANNEL: &str = "bmkibler";
static NICK: &str = "ToerkBot";
static TWITCH_URL: &str = "ws://irc-ws.chat.twitch.tv:80";

fn main() {
    let args: Vec<String> = env::args().collect();

    let channel = if args.len() > 1 {
        let value = &args[1];
        if value.is_empty() {
            CHANNEL
        } else {
            value
        }
    } else {
        CHANNEL
    };

    let token = env::var("TWITCH_BOT_TOKEN").unwrap_or_else(|_| {
        eprintln!("TWITCH_BOT_TOKEN env variable not set");
        exit(1);
    });

    let output = stdout();
    let size = size().expect("Failed to fetch terminal size");

    execute!(stdout(), EnterAlternateScreen).unwrap();

    let mut screen = Screen::new(output, Size::new(size.0, size.1)).unwrap();
    let mut window = Window::new(Pos::new(0, 0), Size::new(size.0, size.1));

    let client = TwitchClient::new(TWITCH_URL, token, channel, NICK);
    let (client_receiver, _join_handle) = client.run();

    screen.enable_raw_mode().expect("could not enable raw mode");

    let event_handler = EventHandler::new();
    let (event_rx, _join_handle) = event_handler.run();

    loop {
        select! {
            recv(client_receiver) -> msg => {
                match msg.unwrap() {
                    Message::Info(message) => {
                        let message = format!("| {} | {}", Local::now().format("%H:%M:%S"), message);
                        window.print(&mut screen, message, Style::none());
                        window.newline(&mut screen);
                    },
                    Message::PrivMsg(message) => {
                        if let Ok(message) = ChatMessage::parse(&message) {
                            window.print(&mut screen, "| ", Style::none());
                            window.print(&mut screen,
                                message.meta_data.tmi_sent_ts.with_timezone(&Local)
                                    .format("%H:%M:%S")
                                    .to_string(),
                                Style::none());
                            window.print(&mut screen, " | ", Style::none());
                            let (r, g, b) = message.meta_data.color.flatten().unwrap_or_else(color_gen::get_color);
                            window.print(&mut screen, message.meta_data.display_name.unwrap(), Style::fg(Some(Color::Rgb {r, g, b})));
                            window.print(&mut screen, " | ", Style::none());
                            window.print(&mut screen, message.message.trim(), Style::none());
                            window.newline(&mut screen);
                        } else if !message.starts_with('@') {
                            let message = format!("| {} | {}", Local::now().format("%H:%M:%S"), message);
                            window.print(&mut screen, message, Style::none());
                            window.newline(&mut screen);
                        }
                    },
                    Message::Error(message) => {
                        let message = format!("| {} | {}", Local::now().format("%H:%M:%S"), message);
                        window.print(&mut screen, message, Style::fg(Some(Color::Red)));
                        window.newline(&mut screen);
                    },

                }
                screen.render().unwrap();
            },
            recv(event_rx) -> msg => {
                let msg = msg.unwrap();
                match msg {
                    Action::Clear => {
                        window.clear(&mut screen);
                        screen.render().unwrap();
                    },
                    Action::Exit => {
                        std::process::exit(0);
                    }
                }
            }
        }
    }
}
