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

        // TODO these should be sent through a channel instead, raw mode prevents println
        println!("Connected to the server");
        println!("Response https code is: {}", response.status());
        println!("Response contains the following headers: ");
        for (header, value) in response.headers() {
            println!("* {}: {:?}", header, value);
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
