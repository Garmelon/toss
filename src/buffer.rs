use crossterm::style::ContentStyle;
use unicode_segmentation::UnicodeSegmentation;

use crate::widthdb::WidthDB;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    pub const ZERO: Self = Self {
        width: 0,
        height: 0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub const ZERO: Self = Self { x: 0, y: 0 };

    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub content: Box<str>,
    pub style: ContentStyle,
    pub width: u8,
    pub offset: u8,
}

impl Cell {
    pub fn empty() -> Self {
        Self {
            content: " ".to_string().into_boxed_str(),
            style: ContentStyle::default(),
            width: 1,
            offset: 0,
        }
    }
}

#[derive(Debug, Default)]
pub struct Buffer {
    size: Size,
    data: Vec<Cell>,
}

impl Buffer {
    fn index(&self, x: u16, y: u16) -> usize {
        assert!(x < self.size.width);
        assert!(y < self.size.height);

        let x: usize = x.into();
        let y: usize = y.into();
        let width: usize = self.size.width.into();

        y * width + x
    }

    pub fn size(&self) -> Size {
        self.size
    }

    /// Resize the buffer and reset its contents.
    ///
    /// The buffer's contents are reset even if the buffer is already the
    /// correct size.
    pub fn resize(&mut self, size: Size) {
        if size == self.size() {
            self.data.fill_with(Cell::empty);
        } else {
            let width: usize = size.width.into();
            let height: usize = size.height.into();
            let len = width * height;

            self.size = size;
            self.data.clear();
            self.data.resize_with(len, Cell::empty);
        }
    }

    /// Reset the contents of the buffer.
    ///
    /// `buf.reset()` is equivalent to `buf.resize(buf.size())`.
    pub fn reset(&mut self) {
        self.data.fill_with(Cell::empty);
    }

    pub fn write(
        &mut self,
        widthdb: &mut WidthDB,
        mut pos: Pos,
        content: &str,
        style: ContentStyle,
    ) {
        if pos.y < 0 || pos.y >= self.size.height as i32 {
            return;
        }

        for grapheme in content.graphemes(true) {
            let width = widthdb.width(grapheme);
            if pos.x >= 0 && pos.x + width as i32 <= self.size.width as i32 {
                // Grapheme fits on buffer in its entirety
                let grapheme = grapheme.to_string().into_boxed_str();

                for offset in 0..width {
                    let i = self.index(pos.x as u16 + offset as u16, pos.y as u16);
                    self.data[i] = Cell {
                        content: grapheme.clone(),
                        style,
                        width,
                        offset,
                    };
                }
            }

            pos.x += width as i32;
        }
    }

    pub fn cells(&self) -> Cells<'_> {
        Cells {
            buffer: self,
            x: 0,
            y: 0,
        }
    }
}

pub struct Cells<'a> {
    buffer: &'a Buffer,
    x: u16,
    y: u16,
}

impl<'a> Cells<'a> {
    fn at(&self, x: u16, y: u16) -> &'a Cell {
        assert!(x < self.buffer.size.width);
        assert!(y < self.buffer.size.height);
        &self.buffer.data[self.buffer.index(x, y)]
    }
}

impl<'a> Iterator for Cells<'a> {
    type Item = (u16, u16, &'a Cell);

    fn next(&mut self) -> Option<Self::Item> {
        if self.y >= self.buffer.size.height {
            return None;
        }

        let (x, y) = (self.x, self.y);
        let cell = self.at(self.x, self.y);
        assert!(cell.offset == 0);

        self.x += cell.width as u16;
        if self.x >= self.buffer.size.width {
            self.x = 0;
            self.y += 1;
        }

        Some((x, y, cell))
    }
}
