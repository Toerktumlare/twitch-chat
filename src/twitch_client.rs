#![allow(dead_code)]
use std::{
    any::type_name,
    io::ErrorKind,
    thread::{self, JoinHandle},
};

use crossbeam::channel::{unbounded, Receiver};
use tungstenite::{connect, Error, Message as SockMessage};
use url::Url;

use crate::log::get_logger;

pub struct TwitchClient {
    token: String,
    url: String,
    channel: String,
    nick: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Message {
    Info(String),
    PrivMsg(String),
    Error(String),
}

impl Message {
    pub fn info(text: impl Into<String>) -> Self {
        Message::Info(text.into())
    }

    pub fn privmsg(text: impl Into<String>) -> Self {
        Message::PrivMsg(text.into())
    }

    pub fn error(text: impl Into<String>) -> Self {
        Message::Error(text.into())
    }
}

impl TwitchClient {
    pub fn new(
        url: impl Into<String>,
        token: impl Into<String>,
        channel: impl Into<String>,
        nick: impl Into<String>,
    ) -> Self {
        TwitchClient {
            url: url.into(),
            token: token.into(),
            channel: channel.into(),
            nick: nick.into(),
        }
    }

    pub fn run(&self) -> (Receiver<Message>, Option<JoinHandle<()>>) {
        let log = get_logger();
        log.info("Starting Twitch-Client", type_name::<TwitchClient>());
        let (tx, rx) = unbounded();
        let token = &self.token;

        log.debug(
            format!("Connecting to url: {}", self.url),
            type_name::<TwitchClient>(),
        );

        let (mut socket, response) = connect(Url::parse(self.url.as_str()).unwrap()).unwrap();

        tx.send(Message::Info(format!("Connected to {}", self.url)))
            .unwrap();
        tx.send(Message::Info(format!(
            "Response https code: {}",
            response.status()
        )))
        .unwrap();
        for (header, value) in response.headers() {
            tx.send(Message::Info(format!("* {}: {:?}", header, value)))
                .unwrap();
        }

        let token = format!("PASS {}", token);
        let nick_message = format!("NICK {}", self.nick);
        let join_message = format!("JOIN #{}", self.channel);
        let tag_message = "CAP REQ :twitch.tv/tags".to_string();

        socket.write_message(SockMessage::Text(token)).unwrap();
        socket
            .write_message(SockMessage::Text(nick_message))
            .unwrap();
        socket
            .write_message(SockMessage::Text(join_message))
            .unwrap();
        socket
            .write_message(SockMessage::Text(tag_message))
            .unwrap();

        tx.send(Message::Info(format!("Joined channel: {}", &self.channel)))
            .unwrap();

        let builder = thread::Builder::new().name("twitch-client".to_string());

        let handle = builder
            .spawn(move || loop {
                match socket.read_message() {
                    Ok(msg) => {
                        log.debug("received a message", type_name::<TwitchClient>());
                        let msg = msg.to_text().unwrap();

                        if msg.contains("PING :tmi.twitch.tv") {
                            socket
                                .write_message(SockMessage::Text("PONG :tmi.twitch.tv".into()))
                                .unwrap();
                        } else {
                            let msg = if msg.contains("PRIVMSG") {
                                Message::privmsg(msg)
                            } else {
                                Message::info(msg)
                            };
                            tx.send(msg).unwrap();
                        }
                    }
                    Err(Error::Io(ref err)) if err.kind() == ErrorKind::WouldBlock => break,
                    Err(_) => {} // Don't look here
                }
            })
            .unwrap();
        (rx, Some(handle))
    }
}
