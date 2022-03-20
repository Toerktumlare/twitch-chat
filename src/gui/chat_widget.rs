use std::{any::type_name, io::Write};

use chrono::Local;
use crossterm::style::Color;

use crate::{
    color_holder::ColorCache, log::get_logger, parser::chat_message::ChatMessage,
    string_padder::StringPadder,
};

use super::{buffer::Style, screen::Screen, window::Window, Pos, Size};

pub struct ChatWidget<'a> {
    window: &'a mut Window,
    size: Size,
    pos: Pos,
    padder: StringPadder,
    color_cache: ColorCache,
}

impl<'a> ChatWidget<'a> {
    pub fn new(window: &'a mut Window, pos: Pos, size: Size) -> Self {
        Self {
            window,
            pos,
            size,
            padder: StringPadder::new(),
            color_cache: ColorCache::new(),
        }
    }

    pub fn print(&mut self, screen: &mut Screen<impl Write>, message: ChatMessage) {
        let log = get_logger();
        let timestamp = message
            .meta_data
            .tmi_sent_ts
            .with_timezone(&Local)
            .format("%H:%M:%S")
            .to_string();
        self.print_timestamp(screen, Some(timestamp));

        let (r, g, b) = message.meta_data.user_info.color.unwrap_or_else(|| {
            let username = message.meta_data.user_info.display_name.unwrap();
            log.debug(
                format!("no color information found for user: {username}"),
                type_name::<ChatWidget>(),
            );
            self.color_cache
                .get(message.meta_data.user_info.display_name.unwrap())
        });

        let display_name = message.meta_data.user_info.display_name.unwrap();
        let display_name = self.padder.add_pad(display_name);

        self.print_display_name(screen, Some(&display_name), Some((r, g, b)));

        let msg = message.message.replace("Kappa", "\u{1F608}");
        let msg = msg.replace(":)", "\u{1F600}");

        let msg = msg.trim();
        let current_width = (self.size.width - (18 + self.padder.current_max) as u16) as usize;
        let msg = textwrap::wrap(msg, current_width);

        log.debug(
            format!(
                "Textwrapping info - chat area width: {}, no of lines: {}, msg: {:?}",
                current_width,
                msg.len(),
                msg
            ),
            type_name::<ChatWidget>(),
        );

        for (i, msg) in msg.iter().enumerate() {
            if i == 0 {
                self.print_msg(screen, msg)
            } else {
                self.print_timestamp(screen, None);
                self.print_display_name(screen, None, None);
                self.print_msg(screen, msg);
            }
            self.window.newline(screen);
        }
    }

    fn print_timestamp(&mut self, screen: &mut Screen<impl Write>, timestamp: Option<String>) {
        self.window.print(screen, "| ", Style::none());
        if let Some(timestamp) = timestamp {
            self.window.print(screen, timestamp, Style::none());
        } else {
            self.window.print(screen, "        ", Style::none());
        }
        self.window.print(screen, " ", Style::none());
    }

    fn print_display_name(
        &mut self,
        screen: &mut Screen<impl Write>,
        display_name: Option<&str>,
        color: Option<(u8, u8, u8)>,
    ) {
        self.window.print(screen, "| ", Style::none());
        if let Some(display_name) = display_name {
            let (r, g, b) = color.unwrap();
            self.window.print(
                screen,
                display_name,
                Style::fg(Some(Color::Rgb { r, g, b })),
            );
        } else {
            let value = format!("{:1$}", " ", self.padder.current_max as usize);
            self.window.print(screen, value, Style::none());
        }
        self.window.print(screen, " ", Style::none());
    }

    fn print_msg(&mut self, screen: &mut Screen<impl Write>, msg: &str) {
        self.window.print(screen, "| ", Style::none());
        self.window.print(screen, msg, Style::none());
    }

    pub fn clear(&mut self, screen: &mut Screen<impl Write>) {
        let log = get_logger();
        log.debug("Clearing chat window", type_name::<ChatWidget>());
        self.padder.reset();
        screen.erase_region(self.pos, self.size);
        self.window.cursor = Pos::zero();
    }
}
