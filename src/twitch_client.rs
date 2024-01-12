#![allow(dead_code)]
use crate::log::get_logger;
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::{
    any::type_name,
    borrow::Cow,
    error::Error,
    io::ErrorKind,
    net::TcpStream,
    ops::{Deref, DerefMut},
    thread::{self, JoinHandle},
    time::Duration,
};
use tungstenite::{
    connect,
    protocol::{frame::coding::CloseCode, CloseFrame},
    stream::MaybeTlsStream,
    Error as WebClientErr, Message as SocketMessage, WebSocket,
};

pub struct TwitchClient {
    worker: Worker,
    sender: Sender<Message>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Message {
    Text(String),
    Terminate,
}

impl TwitchClient {
    pub fn new(
        url: impl Into<String>,
        token: impl Into<String>,
        channel: impl Into<String>,
        nick: impl Into<String>,
    ) -> Result<TwitchClient, Box<dyn Error>> {
        let log = get_logger();
        log.debug(
            format!("logger values: {:#?}", log),
            type_name::<TwitchClient>(),
        );
        log.info("Starting Twitch-Client", type_name::<TwitchClient>());

        let url = url.into();
        let token = token.into();
        let channel = channel.into();
        let nick = nick.into();

        log.debug(
            format!("Connecting to url: {}", &url),
            type_name::<TwitchClient>(),
        );

        let (mut socket, response) = connect(&url)?;

        if let MaybeTlsStream::Plain(socket) = socket.get_mut() {
            socket.set_nonblocking(true)?;
        }

        log.info(
            format!("Connected to: {}", &url),
            type_name::<TwitchClient>(),
        );
        log.info(
            format!("Response https code: {}", &response.status()),
            type_name::<TwitchClient>(),
        );

        for (header, value) in response.headers() {
            log.info(
                format!("* {}: {:?}", header, value),
                type_name::<TwitchClient>(),
            );
        }

        let token = format!("PASS oauth:{}", &token);
        let nick_message = format!("NICK {}", &nick);
        let join_message = format!("JOIN #{}", &channel);
        let tag_message = "CAP REQ :twitch.tv/tags";

        socket.write_message(SocketMessage::Text(token)).unwrap();
        socket
            .write_message(SocketMessage::Text(nick_message))
            .unwrap();
        socket
            .write_message(SocketMessage::Text(join_message))
            .unwrap();
        socket
            .write_message(SocketMessage::Text(tag_message.to_string()))
            .unwrap();

        log.debug(
            format!("Joined channel: {}, with nick: {}", &channel, &nick),
            type_name::<TwitchClient>(),
        );

        let (sender, receiver) = unbounded();
        let worker = Worker::run("ws-worker", receiver, socket)?;
        Ok(TwitchClient { sender, worker })
    }
}

impl Deref for TwitchClient {
    type Target = Worker;

    fn deref(&self) -> &Self::Target {
        &self.worker
    }
}

impl DerefMut for TwitchClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.worker
    }
}

impl Drop for TwitchClient {
    fn drop(&mut self) {
        self.sender.send(Message::Terminate).unwrap();
        self.thread.take().map(JoinHandle::join);
    }
}

pub struct Worker {
    name: String,
    thread: Option<JoinHandle<()>>,
    pub receiver: Receiver<Message>,
}

impl Worker {
    pub fn run(
        name: impl Into<String>,
        receiver: Receiver<Message>,
        mut socket: WebSocket<MaybeTlsStream<TcpStream>>,
    ) -> Result<Worker, Box<dyn Error>> {
        let (tx, rx) = unbounded();
        let name = name.into();
        let handle = thread::Builder::new().name(name.clone()).spawn(move || {
            let log = get_logger();
            loop {
                thread::sleep(Duration::from_millis(300));
                log.trace("Working", type_name::<Worker>());
                match socket.read_message() {
                    Ok(msg) => {
                        let msg = msg.to_text().unwrap();

                        if msg.contains("PING :tmi.twitch.tv") {
                            socket
                                .write_message(tungstenite::Message::Text(
                                    "PONG :tmi.twitch.tv".into(),
                                ))
                                .unwrap();
                        } else {
                            tx.send(Message::Text(msg.into())).unwrap();
                        }
                    }
                    Err(WebClientErr::Io(ref err)) if err.kind() == ErrorKind::WouldBlock => {}
                    Err(err) => match err {
                        WebClientErr::ConnectionClosed => {
                            log.debug("Connection closed", type_name::<Worker>());
                            break;
                        }
                        _ => log.error(
                            format!("something very unexpected happened: {}", err),
                            type_name::<Worker>(),
                        ),
                    },
                }
                if let Ok(Message::Terminate) = receiver.try_recv() {
                    log.debug("Closing connection to Twitch", type_name::<Worker>());
                    socket
                        .close(Some(CloseFrame {
                            code: CloseCode::Normal,
                            reason: Cow::Borrowed(""),
                        }))
                        .unwrap();
                }
            }
        })?;

        Ok(Worker {
            name,
            thread: Some(handle),
            receiver: rx,
        })
    }
}
