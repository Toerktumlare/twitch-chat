use std::io::Write;

use super::{
    buffer::{Cell, Style},
    screen::Screen,
    Pos, Size,
};

pub struct Window {
    pos: Pos,
    size: Size,
    cursor: Pos,
}

impl Window {
    pub fn new(pos: Pos, size: Size) -> Self {
        Self {
            pos,
            size,
            cursor: Pos::zero(),
        }
    }

    pub fn print(&mut self, screen: &mut Screen<impl Write>, s: impl AsRef<str>, style: Style) {
        for c in s.as_ref().chars() {
            self.put(screen, c, style)
        }
    }

    pub fn newline(&mut self, screen: &mut Screen<impl Write>) {
        if self.cursor.y < screen.size().height - 1 {
            self.cursor.x = 0;
            self.cursor.y += 1;
        } else {
            screen.scroll_up(1);
            self.cursor.x = 0;
        }
    }

    pub fn put(&mut self, screen: &mut Screen<impl Write>, c: char, style: Style) {
        if c == '\n' {
            self.newline(screen);
            return;
        }

        let cell = Cell::new(c, style);
        screen.put(cell, self.pos + self.cursor);

        if self.cursor.x + cell.width() >= self.size.width() {
            self.newline(screen);
        } else {
            self.cursor.x += cell.width();
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::io::stdout;

    #[test]
    pub fn should_place_char_at_zero_zero() {
        let mut screen = Screen::new(stdout(), Size::new(2, 2)).unwrap();
        let mut window = Window::new(Pos::new(0, 0), Size::new(2, 2));
        window.print(&mut screen, "Helo", Style::none());
    }
}
