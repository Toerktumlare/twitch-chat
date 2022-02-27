use std::{any::type_name, io::stdout};

use crate::{
    chat_message::ChatMessage,
    color_gen,
    gui::{
        buffer::Style,
        event_handler::{Action, EventHandler},
        screen::Screen,
        window::Window,
        Pos, Size,
    },
    log::get_logger,
    twitch_client::TwitchClient,
};
use chrono::Local;
use crossbeam::select;
use crossterm::{
    execute,
    style::Color,
    terminal::{size, EnterAlternateScreen},
};

use crate::twitch_client::Message;

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
        let log = get_logger();
        log.info("starting Twitch Chat", type_name::<TwitchChat>());

        let output = stdout();
        let size = size().expect("Failed to fetch terminal size");

        execute!(stdout(), EnterAlternateScreen).unwrap();

        let mut screen = Screen::new(output, Size::new(size.0, size.1)).unwrap();
        let mut window = Window::new(Pos::new(0, 0), Size::new(size.0, size.1));
        screen.enable_raw_mode().expect("could not enable raw mode");

        let client = TwitchClient::new(TWITCH_URL, &self.token, &self.channel, &self.nick).unwrap();
        let event_handler = EventHandler::new();

        loop {
            select! {
                recv(client.receiver) -> chat_event => {
                    if let Ok(chat_event) = chat_event {
                        match chat_event {
                            Message::Info(message) => {
                                let message = format!("| {} | {}", Local::now().format("%H:%M:%S"), message);
                                log.info(&message, type_name::<TwitchChat>());
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

                                    log.debug(msg.trim(), type_name::<TwitchChat>());
                                    window.print(&mut screen, msg.trim(), Style::none());
                                    window.newline(&mut screen);
                                } else if !message.starts_with('@') {
                                    log.error(message.trim(), type_name::<TwitchChat>());
                                    let message = format!("| {} | {}", Local::now().format("%H:%M:%S"), message);
                                    window.print(&mut screen, message, Style::none());
                                    window.newline(&mut screen);
                                } else {
                                    log.error(&message, type_name::<TwitchChat>());
                                }
                            },
                            Message::Error(message) => {
                                let message = format!("| {} | {}", Local::now().format("%H:%M:%S"), message);
                                window.print(&mut screen, message, Style::fg(Some(Color::Red)));
                                window.newline(&mut screen);
                            },
                            _ => {},
                        }
                        screen.render().unwrap();
                    }
                },
                recv(event_handler.receiver) -> action => {
                    if let Ok(action) = action {
                        match action {
                            Action::Clear => {
                                window.clear(&mut screen);
                                screen.render().unwrap();
                            },
                            Action::Exit => break,
                        }

                    }
                }
            }
        }
        log.info("Exiting twitch chat", type_name::<TwitchChat>());
    }
}
