use std::io::{Result, Write};

use crossterm::{
    cursor,
    style::{Print, SetForegroundColor},
    terminal::size as term_size,
    ExecutableCommand, QueueableCommand,
};

use super::{
    buffer::{Buffer, Cell, CellState},
    Pos, Size,
};

pub struct Screen<W: Write> {
    new_buffer: Buffer,
    old_buffer: Buffer,
    size: Size,
    output: W,
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
        output.queue(cursor::Hide)?;
        Ok(Self {
            output,
            new_buffer: Buffer::new(size),
            old_buffer: Buffer::new(size),
            size,
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
            .execute(cursor::Show)
            .expect("Could not show cursor when shutting down");
    }
}
