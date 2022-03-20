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
        self.window.print(screen, "| ", Style::none());
        self.window.print(
            screen,
            message
                .meta_data
                .tmi_sent_ts
                .with_timezone(&Local)
                .format("%H:%M:%S")
                .to_string(),
            Style::none(),
        );
        self.window.print(screen, " | ", Style::none());
        let (r, g, b) = message.meta_data.user_info.color.unwrap_or_else(|| {
            let username = message.meta_data.user_info.display_name.unwrap();
            log.debug(
                format!("no color information found for user: {username}"),
                type_name::<ColorCache>(),
            );
            self.color_cache
                .get(message.meta_data.user_info.display_name.unwrap())
        });

        let display_name = message.meta_data.user_info.display_name.unwrap();
        let display_name = self.padder.add_pad(display_name);

        self.window.print(
            screen,
            display_name,
            Style::fg(Some(Color::Rgb { r, g, b })),
        );
        self.window.print(screen, " | ", Style::none());
        let msg = message.message.replace("Kappa", "\u{1F608}");
        let msg = msg.replace(":)", "\u{1F600}");

        self.window.print(screen, msg.trim(), Style::none());
        self.window.newline(screen);
    }

    pub fn clear(&mut self, screen: &mut Screen<impl Write>) {
        self.padder.reset();
        screen.erase_region(self.pos, self.size);
        self.window.cursor = Pos::zero();
    }
}
