use std::collections::HashMap;

pub fn parse(args: &[String]) -> HashMap<&str, &str> {
    let mut parsed_args = HashMap::new();
    args.iter().for_each(|input| {
        if input == "--debug" {
            parsed_args.insert("debug", "true");
        }

        if let Some(value) = input.strip_prefix("--nick=") {
            parsed_args.insert("nick", value);
        }

        if let Some(value) = input.strip_prefix("--channel=") {
            parsed_args.insert("channel", value);
        }

        if input == "--debug" {
            parsed_args.insert("log_level", "debug");
        }

        if input == "--info" {
            parsed_args.insert("log_level", "info");
        }

        if input == "--warn" {
            parsed_args.insert("log_level", "warn");
        }

        if input == "--error" {
            parsed_args.insert("log_level", "error");
        }

        if input == "--log" {
            parsed_args.insert("logging", "true");
        }
    });

    parsed_args
}
