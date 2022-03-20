use log::{get_logger_mut, LogLevel};
use std::{env, process::exit};
use twitch_chat::TwitchChat;

mod arg_parser;
mod color_gen;
mod color_holder;
mod gui;
mod log;
mod parser;
mod string_padder;
mod twitch_chat;
mod twitch_client;

fn main() {
    log::init();

    let args: Vec<String> = env::args().collect();
    let mut arg_map = arg_parser::parse(&args);

    let log = get_logger_mut();
    if let Some(value) = arg_map.remove("logging") {
        if value.parse::<bool>().unwrap() {
            log.enabled();
        }
    }

    if let Some(&value) = arg_map.get("log_level") {
        match value {
            "debug" => log.set_level(LogLevel::Debug),
            "info" => log.set_level(LogLevel::Info),
            "warn" => log.set_level(LogLevel::Warn),
            "error" => log.set_level(LogLevel::Error),
            "trace" => log.set_level(LogLevel::Trace),
            _ => (),
        }
    }

    log.debug(format!("logger values: {:#?}", log), "main");
    log.info("Starting application", "main");

    let nick = arg_map.remove("nick").unwrap_or_else(|| {
        eprintln!("ERROR: no nick was provided");
        print_help();
        exit(1);
    });

    let channel = arg_map.remove("channel").unwrap_or_else(|| {
        eprintln!("ERROR: no channel was provided");
        print_help();
        exit(1);
    });

    let token = env::var("TWITCH_BOT_TOKEN").unwrap_or_else(|_| {
        eprintln!("ERROR: TWITCH_BOT_TOKEN env variable not set");
        print_help();
        exit(1);
    });

    log.info("Config:", "main");
    log.info(format!("\t nick: {}", nick), "main");
    log.info(format!("\t channel: {}", channel), "main");

    let twitch_chat = TwitchChat::new(nick, channel, token);
    twitch_chat.start();

    log.close();
}

fn print_help() {
    println!("\u{1F608}");
    println!("NAME");
    println!("\t Twitch-Chat an amazing terminal user interface twitch chat listener");
    println!();
    println!("SYNOPSIS");
    println!("\t twitch-chat [options]");
    println!();
    println!("\t twitch-chat is a terminal based twitch chat listener. It is used to conviniently listen to twitch chat in a terminal window while streaming. It has color support and will display timestamps for each message.");
    println!();
    println!("EXAMPLES");
    println!("\t twitch-chat --nick=foobar --channel=flubber");
    println!();
    println!("OPTIONS");
    println!("\t --nick");
    println!("\t\t nick of the account the provided token is associated with.");
    println!("\t --channel");
    println!("\t\t name of the channel you want to connect to");
}
