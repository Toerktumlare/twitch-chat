use std::{any::type_name, io::Write};

use chrono::Local;
use crossterm::style::Color;

use crate::{chat_message::ChatMessage, color_gen, log::get_logger};

use super::{buffer::Style, screen::Screen, window::Window, Pos, Size};

pub struct ChatWidget<'a> {
    window: &'a mut Window,
    size: Size,
    pos: Pos,
}

impl<'a> ChatWidget<'a> {
    pub fn new(window: &'a mut Window, pos: Pos, size: Size) -> Self {
        Self { window, pos, size }
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
        let (r, g, b) = message
            .meta_data
            .color
            .flatten()
            .unwrap_or_else(color_gen::get_color);
        self.window.print(
            screen,
            message.meta_data.display_name.unwrap(),
            Style::fg(Some(Color::Rgb { r, g, b })),
        );
        self.window.print(screen, " | ", Style::none());
        let msg = message.message.replace("Kappa", "\u{1F608}");
        let msg = msg.replace(":)", "\u{1F600}");

        log.debug(msg.trim(), type_name::<ChatWidget>());
        self.window.print(screen, msg.trim(), Style::none());
        self.window.newline(screen);
    }

    pub fn clear(&mut self, screen: &mut Screen<impl Write>) {
        screen.erase_region(self.pos, self.size);
        self.window.cursor = Pos::zero();
    }
}
