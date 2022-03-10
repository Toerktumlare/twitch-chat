use std::{any::type_name, io::stdout};

use crate::{
    gui::{
        chat_widget::ChatWidget,
        event_handler::{Action, EventHandler},
        screen::Screen,
        window::Window,
        Pos, Size,
    },
    log::get_logger,
    parser::chat_message::ChatMessage,
    twitch_client::TwitchClient,
};

use crossbeam::select;
use crossterm::{
    execute,
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
        let mut chat = ChatWidget::new(&mut window, Pos::new(0, 0), Size::new(size.0, size.1));
        screen.enable_raw_mode().expect("could not enable raw mode");

        let client = TwitchClient::new(TWITCH_URL, &self.token, &self.channel, &self.nick).unwrap();
        let event_handler = EventHandler::new();

        loop {
            select! {
                recv(client.receiver) -> chat_event => {
                    if let Ok(chat_event) = chat_event {
                        if let Message::Text(message) = chat_event {

                            match ChatMessage::parse(&message) {
                                Ok(message) => chat.print(&mut screen, message),
                                Err(message) => log.error(format!("{:#?}", message), type_name::<TwitchChat>()),
                            }

                            // if let Ok(message) = ChatMessage::parse(&message) {
                            //     chat.print(&mut screen, message);
                            // } else if !message.starts_with('@') {
                            //     log.error(message.trim(), type_name::<TwitchChat>());
                            //     let _message = format!("| {} | {}", Local::now().format("%H:%M:%S"), message);
                            //     // chat.print(&mut screen, message);
                            // } else {
                            //     log.debug(&message, type_name::<TwitchChat>());
                            // }
                        }
                        screen.render().unwrap();
                    }
                },
                recv(event_handler.receiver) -> action => {
                    if let Ok(action) = action {
                        match action {
                            Action::Clear => {
                                chat.clear(&mut screen);
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
