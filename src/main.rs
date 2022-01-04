#![allow(dead_code)]
use std::{env, process::exit, sync::mpsc, thread};
use tungstenite::{connect, Message};
use url::Url;

fn main() {
    println!("Thread: {:?}", thread::current().id());
    let token = env::var("TWITCH_BOT_TOKEN").unwrap_or_else(|_| {
        eprintln!("TWITCH_BOT_TOKEN env variable not set");
        exit(1);
    });

    let (tx, rx) = mpsc::channel();

    let _handle = thread::spawn(move || {
        println!("Thread: {:?}", thread::current().id());

        let (mut socket, response) = connect(Url::parse("ws://irc-ws.chat.twitch.tv:80").unwrap())
            .expect("couldn't connect");

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
            .write_message(Message::Text("JOIN #mewtru".into()))
            .unwrap();
        socket
            .write_message(Message::Text("CAP REQ :twitch.tv/tags".into()))
            .unwrap();

        loop {
            let msg = socket.read_message().expect("error reading msgs");

            if msg.to_text().unwrap().contains("PING :tmi.twitch.tv") {
                println!("PONG back at ya, twitcherino!");
                socket
                    .write_message(Message::Text("PONG :tmi.twitch.tv".into()))
                    .unwrap();
            }
            tx.send(msg).unwrap();
        }
    });

    loop {
        let received = rx.recv().unwrap();
        println!("Received message: {}", received);
    }

    //handle.join().unwrap();
}
