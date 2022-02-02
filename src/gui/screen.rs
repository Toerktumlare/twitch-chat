use std::io::{Result, Write};

use crossterm::{
    cursor::{self, MoveTo},
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, size as term_size, Clear, ClearType,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
    ExecutableCommand, QueueableCommand,
};

use super::{
    buffer::{Buffer, Cell, CellState},
    Pos, Size,
};

#[derive(Debug, Eq, PartialEq)]
pub struct Screen<W: Write> {
    new_buffer: Buffer,
    old_buffer: Buffer,
    size: Size,
    output: W,
    alt_screen: bool,
}

impl Screen<std::io::Stdout> {
    pub fn stdout() -> Result<Self> {
        let stdout = std::io::stdout();
        let size: Size = term_size()?.into();
        Self::new(stdout, size)
    }
}

impl<W: Write> Screen<W> {
    pub fn new(mut output: W, size: impl Into<Size>) -> Result<Self> {
        let size: Size = size.into();
        output.queue(cursor::MoveTo(0, 0))?;
        // output.queue(cursor::Hide)?;
        Ok(Self {
            output,
            new_buffer: Buffer::new(size),
            old_buffer: Buffer::new(size),
            size,
            alt_screen: false,
        })
    }

    pub fn size(&self) -> Size {
        self.size
    }

    fn contains(&self, pos: Pos) -> bool {
        pos.x < self.size.width && pos.y < self.size.height
    }

    pub fn put(&mut self, cell: Cell, pos: Pos) {
        if self.contains(pos) {
            self.new_buffer.put(cell, pos)
        }
    }

    pub fn alternate_screen(mut self, value: bool) -> Self {
        self.alt_screen = value;
        if self.alt_screen {
            self.output.execute(EnterAlternateScreen).unwrap();
        }
        self
    }

    pub fn enable_raw_mode(&self) -> Result<()> {
        enable_raw_mode()?;
        Ok(())
    }

    pub fn disable_raw_mode(&self) -> Result<()> {
        disable_raw_mode()?;
        Ok(())
    }

    pub fn clear_all(&mut self) -> Result<()> {
        self.output.execute(MoveTo(0, 0))?;
        self.output.execute(SetForegroundColor(Color::Reset))?;
        self.output.execute(SetBackgroundColor(Color::Reset))?;
        self.output.execute(Clear(ClearType::All))?;
        Ok(())
    }

    pub fn erase_region(&mut self, pos: Pos, size: Size) {
        let to_x = (size.width + pos.x).min(self.size.width);
        let to_y = (size.height + pos.y).min(self.size.height);

        for x in pos.x.min(to_x)..to_x {
            for y in pos.y.min(to_y)..to_y {
                self.new_buffer.empty(Pos::new(x, y));
            }
        }
    }

    pub fn scroll_up(&mut self, lines: usize) {
        let count = lines * self.size.width as usize;
        self.new_buffer.inner.drain(0..count);
        let mut empty_line = vec![Cell::empty(); count];
        self.new_buffer.inner.append(&mut empty_line);
    }

    pub fn render(&mut self) -> Result<()> {
        for (y, column) in self.new_buffer.lines().enumerate() {
            for (x, cell) in column.iter().enumerate() {
                self.output.queue(cursor::MoveTo(x as u16, y as u16))?;

                if let Some(fg) = cell.style.fg {
                    self.output.queue(SetForegroundColor(fg))?;
                }

                if let Some(bg) = cell.style.bg {
                    self.output.queue(SetForegroundColor(bg))?;
                }

                let _ = match cell.cell_state {
                    CellState::Empty => self.output.queue(Print(' '))?,
                    CellState::Occupied(c) => self.output.queue(Print(c))?,
                    CellState::Continuation => continue,
                };
            }
        }
        self.output.flush()?;
        self.old_buffer = self.new_buffer.clone();
        Ok(())
    }
}

impl<W: Write> Drop for Screen<W> {
    fn drop(&mut self) {
        self.output
            .queue(cursor::Show)
            .expect("Could not show cursor when shutting down");
        if self.alt_screen {
            self.output.queue(LeaveAlternateScreen).expect(
                "Could not leave alternate screen, you are forever damned to live in darkness",
            );
        }
        disable_raw_mode().expect("Could not disable raw mode");
        self.output.flush().expect("Could not flush the toilet");
    }
}
