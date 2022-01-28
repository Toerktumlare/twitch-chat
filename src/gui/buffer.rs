use std::ops::Deref;

use crossterm::style::Color;
use unicode_width::UnicodeWidthChar;

use super::{Pos, Size};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
}

impl Style {
    pub fn new(fg: Option<Color>, bg: Option<Color>) -> Self {
        Self { fg, bg }
    }

    pub fn none() -> Style {
        Style::new(None, None)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Cell {
    pub style: Style,
    pub cell_state: CellState,
}

impl Cell {
    pub fn new(c: char, style: Style) -> Self {
        Self {
            style,
            cell_state: CellState::Occupied(c),
        }
    }

    pub fn continuation(style: Style) -> Self {
        Self {
            style,
            cell_state: CellState::Continuation,
        }
    }

    pub fn empty() -> Cell {
        Self {
            style: Style::new(None, None),
            cell_state: CellState::Empty,
        }
    }

    pub fn width(&self) -> u16 {
        match self.cell_state {
            CellState::Occupied(c) => c.width().unwrap_or(1) as u16,
            _ => 0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CellState {
    Empty,
    Occupied(char),
    Continuation,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Buffer {
    size: Size,
    inner: Vec<Cell>,
}

impl Buffer {
    pub fn new(size: impl Into<Size>) -> Self {
        let size = size.into();
        Self {
            inner: vec![Cell::empty(); (size.width * size.height) as usize],
            size,
        }
    }

    pub fn put(&mut self, cell: Cell, pos: Pos) {
        let index = pos.y * self.size.width + pos.x;

        if let CellState::Occupied(c) = cell.cell_state {
            if pos.x < self.size.width {
                if let Some(2..) = c.width() {
                    self.put(Cell::continuation(cell.style), Pos::new(pos.x + 1, pos.y));
                }
            }
        }

        self.inner[index as usize] = cell;
    }

    pub fn empty(&mut self, pos: Pos) {
        let index = pos.y * self.size.width + pos.x;
        self.inner[index as usize] = Cell::empty();
    }

    pub fn lines(&self) -> impl Iterator<Item = &[Cell]> {
        self.inner.chunks(self.size.width as usize)
    }
}

impl Deref for Buffer {
    type Target = Vec<Cell>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn create_empty_buffer() {
        let buffer = Buffer::new(Size::new(2, 2));
        assert_eq!(
            buffer.inner,
            vec![Cell::empty(), Cell::empty(), Cell::empty(), Cell::empty()]
        )
    }

    #[test]
    pub fn should_set_new_cell_at_pos_in_buffer() {
        let mut buffer = Buffer::new(Size::new(2, 2));
        let cell = Cell::new('a', Style::new(None, None));
        buffer.put(cell, Pos::new(0, 1));
        assert_eq!(
            buffer.inner,
            vec![
                Cell::empty(),
                Cell::empty(),
                Cell::new('a', Style::new(None, None)),
                Cell::empty()
            ]
        );
    }

    #[test]
    pub fn should_add_extra_cell_if_unicode_char() {
        let mut buffer = Buffer::new(Size::new(2, 2));
        let cell = Cell::new('ｏ', Style::new(None, None));
        buffer.put(cell, Pos::new(0, 1));
        assert_eq!(
            buffer.inner,
            vec![
                Cell::empty(),
                Cell::empty(),
                Cell::new('ｏ', Style::new(None, None)),
                Cell::continuation(Style::new(None, None)),
            ]
        );
    }

    #[test]
    pub fn should_return_lines() {
        let buffer = Buffer::new(Size::new(5, 5));
        let mut buff_line_iter = buffer.lines();
        assert_eq!(
            buff_line_iter.next().unwrap(),
            &[
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty()
            ]
        )
    }

    #[test]
    pub fn should_iterate_over_buffer_cells() {
        let buffer = Buffer::new(Size::new(5, 5));
        for cell in buffer.iter() {
            assert_eq!(cell, &Cell::empty());
        }
    }
}
