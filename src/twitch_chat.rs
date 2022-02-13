use std::io::stdout;

use crate::gui::{
    buffer::Style,
    event_handler::{Action, EventHandler},
    screen::Screen,
    window::Window,
    Pos, Size,
};
use chrono::Local;
use crossbeam::channel::select;
use crossterm::{
    execute,
    terminal::{size, EnterAlternateScreen},
};

use crate::chat_message::ChatMessage;
use crate::color_gen;
use crate::twitch_client::{Message, TwitchClient};

pub struct TwitchChat {
    nick: String,
    channel: String,
    token: String,
}

static TWITCH_URL: &str = "ws://irc-ws.chat.twitch.tv:80";

impl TwitchChat {
    pub fn new(
        nick: impl Into<String>,
        channel: impl Into<String>,
        token: impl Into<String>,
    ) -> Self {
        Self {
            nick: nick.into(),
            channel: channel.into(),
            token: token.into(),
        }
    }

    pub fn start(&self) {
        let output = stdout();
        let size = size().expect("Failed to fetch terminal size");

        execute!(stdout(), EnterAlternateScreen).unwrap();

        let mut screen = Screen::new(output, Size::new(size.0, size.1)).unwrap();
        let mut window = Window::new(Pos::new(0, 0), Size::new(size.0, size.1));

        let client = TwitchClient::new(TWITCH_URL, &self.token, &self.channel, &self.nick);
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
                                let msg = message.message.replace("Kappa", "\u{1F608}");
                                let msg = msg.replace(":)", "\u{1F600}");
                                window.print(&mut screen, msg.trim(), Style::none());
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
}
