use std::thread::{self, JoinHandle};

use crossbeam::channel::{unbounded, Receiver};
use tungstenite::{connect, Message};
use url::Url;

pub struct TwitchClient {
    token: String,
    url: String,
}

impl TwitchClient {
    pub fn new(url: &str, token: String) -> Self {
        TwitchClient {
            url: url.to_string(),
            token,
        }
    }

    pub fn run(&self) -> (Receiver<Message>, JoinHandle<()>) {
        let (s, r) = unbounded();
        let token = &self.token;

        let (mut socket, response) =
            connect(Url::parse(self.url.as_str()).unwrap()).expect("couldn't connect");

        s.send(Message::Text(format!("Connected to {}", self.url)))
            .unwrap();
        s.send(Message::Text(format!(
            "Response https code: {}",
            response.status()
        )))
        .unwrap();
        s.send(Message::Text(format!("Connected to {}", self.url)))
            .unwrap();
        s.send(Message::Text(format!("Connected to {}", self.url)))
            .unwrap();
        for (header, value) in response.headers() {
            s.send(Message::Text(format!("* {}: {:?}", header, value)))
                .unwrap();
        }

        let token = format!("PASS {}", token);

        socket.write_message(Message::Text(token)).unwrap();
        socket
            .write_message(Message::Text("NICK ToerkBot".into()))
            .unwrap();
        socket
            .write_message(Message::Text("JOIN #toerktumlare".into()))
            .unwrap();
        socket
            .write_message(Message::Text("CAP REQ :twitch.tv/tags".into()))
            .unwrap();

        let handle = thread::spawn(move || loop {
            let msg = socket.read_message().expect("error reading msgs");

            if msg.to_text().unwrap().contains("PING :tmi.twitch.tv") {
                socket
                    .write_message(Message::Text("PONG :tmi.twitch.tv".into()))
                    .unwrap();
            }
            s.send(msg).unwrap();
        });
        (r, handle)
    }
}
