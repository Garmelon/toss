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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub content: Box<str>,
    pub style: ContentStyle,
    pub width: u8,
    pub offset: u8,
}

impl Default for Cell {
    fn default() -> Self {
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

    pub(crate) fn at(&self, x: u16, y: u16) -> &Cell {
        assert!(x < self.size.width);
        assert!(y < self.size.height);
        let i = self.index(x, y);
        &self.data[i]
    }

    fn at_mut(&mut self, x: u16, y: u16) -> &mut Cell {
        assert!(x < self.size.width);
        assert!(y < self.size.height);
        let i = self.index(x, y);
        &mut self.data[i]
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
            self.data.fill_with(Cell::default);
        } else {
            let width: usize = size.width.into();
            let height: usize = size.height.into();
            let len = width * height;

            self.size = size;
            self.data.clear();
            self.data.resize_with(len, Cell::default);
        }
    }

    /// Reset the contents of the buffer.
    ///
    /// `buf.reset()` is equivalent to `buf.resize(buf.size())`.
    pub fn reset(&mut self) {
        self.data.fill_with(Cell::default);
    }

    /// Remove the grapheme at the specified coordinates from the buffer.
    ///
    /// Removes the entire grapheme, not just the cell at the coordinates.
    /// Preserves the style of the affected cells. Works even if the coordinates
    /// don't point to the beginning of the grapheme.
    fn erase(&mut self, x: u16, y: u16) {
        let cell = self.at(x, y);
        let width: u16 = cell.width.into();
        let offset: u16 = cell.offset.into();
        for x in (x - offset)..(x - offset + width) {
            let cell = self.at_mut(x, y);
            let style = cell.style;
            *cell = Cell::default();
            cell.style = style;
        }
    }

    pub fn write(
        &mut self,
        widthdb: &mut WidthDB,
        mut pos: Pos,
        content: &str,
        style: ContentStyle,
    ) {
        // If we're not even visible, there's nothing to do
        if pos.y < 0 || pos.y >= self.size.height as i32 {
            return;
        }
        let y = pos.y as u16;

        for grapheme in content.graphemes(true) {
            let width = widthdb.width(grapheme);
            self.write_grapheme(pos.x, y, width, grapheme, style);
            pos.x += width as i32;
        }
    }

    /// Assumes that `pos.y` is in range.
    fn write_grapheme(&mut self, x: i32, y: u16, width: u8, grapheme: &str, style: ContentStyle) {
        let min_x = 0;
        let max_x = self.size.width as i32 - 1; // Last possible cell

        let start_x = x;
        let end_x = x + width as i32 - 1; // Coordinate of last cell

        if start_x > max_x || end_x < min_x {
            return; // Not visible
        }

        if start_x >= min_x && end_x <= max_x {
            // Fully visible, write actual grapheme
            for offset in 0..width {
                let x = start_x as u16 + offset as u16;
                self.erase(x, y);
                *self.at_mut(x, y) = Cell {
                    content: grapheme.to_string().into_boxed_str(),
                    style,
                    width,
                    offset,
                };
            }
        } else {
            // Partially visible, write empty cells with correct style
            let start_x = start_x.max(0) as u16;
            let end_x = end_x.min(max_x) as u16;
            for x in start_x..=end_x {
                self.erase(x, y);
                *self.at_mut(x, y) = Cell {
                    style,
                    ..Default::default()
                };
            }
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

impl<'a> Iterator for Cells<'a> {
    type Item = (u16, u16, &'a Cell);

    fn next(&mut self) -> Option<Self::Item> {
        if self.y >= self.buffer.size.height {
            return None;
        }

        let (x, y) = (self.x, self.y);
        let cell = self.buffer.at(self.x, self.y);
        assert!(cell.offset == 0);

        self.x += cell.width as u16;
        if self.x >= self.buffer.size.width {
            self.x = 0;
            self.y += 1;
        }

        Some((x, y, cell))
    }
}
