use std::ops::Range;

use crossterm::style::ContentStyle;

use crate::{Pos, Size, Style, Styled, WidthDb};

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

#[derive(Debug, Clone, Copy)]
struct StackFrame {
    pub pos: Pos,
    pub size: Size,
    pub drawable_area: Option<(Pos, Size)>,
}

impl StackFrame {
    fn intersect_areas(
        a_start: Pos,
        a_size: Size,
        b_start: Pos,
        b_size: Size,
    ) -> Option<(Pos, Size)> {
        // The first row/column that is not part of the area any more
        let a_end = a_start + a_size;
        let b_end = b_start + b_size;

        let x_start = a_start.x.max(b_start.x);
        let x_end = a_end.x.min(b_end.x);
        let y_start = a_start.y.max(b_start.y);
        let y_end = a_end.y.min(b_end.y);

        if x_start < x_end && y_start < y_end {
            let start = Pos::new(x_start, y_start);
            let size = Size::new((x_end - x_start) as u16, (y_end - y_start) as u16);
            Some((start, size))
        } else {
            None
        }
    }

    fn then(&self, pos: Pos, size: Size) -> Self {
        let pos = self.local_to_global(pos);

        let drawable_area = self
            .drawable_area
            .and_then(|(da_pos, da_size)| Self::intersect_areas(da_pos, da_size, pos, size));

        Self {
            pos,
            size,
            drawable_area,
        }
    }

    fn local_to_global(&self, local_pos: Pos) -> Pos {
        local_pos + self.pos
    }

    fn global_to_local(&self, global_pos: Pos) -> Pos {
        global_pos - self.pos
    }

    /// Ranges along the x and y axis where drawing is allowed, in global
    /// coordinates.
    fn legal_ranges(&self) -> Option<(Range<i32>, Range<i32>)> {
        if let Some((pos, size)) = self.drawable_area {
            let xrange = pos.x..pos.x + size.width as i32;
            let yrange = pos.y..pos.y + size.height as i32;
            Some((xrange, yrange))
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Buffer {
    size: Size,
    data: Vec<Cell>,
    cursor: Option<Pos>,

    /// A stack of rectangular drawing areas.
    ///
    /// When rendering to the buffer with a nonempty stack, it behaves as if it
    /// was the size of the topmost stack element, and characters are translated
    /// by the position of the topmost stack element. No characters can be
    /// placed outside the area described by the topmost stack element.
    stack: Vec<StackFrame>,
}

impl Buffer {
    /// Index in `data` of the cell at the given position. The position must
    /// be inside the buffer.
    ///
    /// Ignores the stack.
    fn index(&self, x: u16, y: u16) -> usize {
        assert!(x < self.size.width);
        assert!(y < self.size.height);

        let x: usize = x.into();
        let y: usize = y.into();
        let width: usize = self.size.width.into();

        y * width + x
    }

    /// A reference to the cell at the given position. The position must be
    /// inside the buffer.
    ///
    /// Ignores the stack.
    pub fn at(&self, x: u16, y: u16) -> &Cell {
        assert!(x < self.size.width);
        assert!(y < self.size.height);

        let i = self.index(x, y);
        &self.data[i]
    }

    /// A mutable reference to the cell at the given position. The position must
    /// be inside the buffer.
    ///
    /// Ignores the stack.
    fn at_mut(&mut self, x: u16, y: u16) -> &mut Cell {
        assert!(x < self.size.width);
        assert!(y < self.size.height);

        let i = self.index(x, y);
        &mut self.data[i]
    }

    fn current_frame(&self) -> StackFrame {
        self.stack.last().copied().unwrap_or(StackFrame {
            pos: Pos::ZERO,
            size: self.size,
            drawable_area: Some((Pos::ZERO, self.size)),
        })
    }

    pub fn push(&mut self, pos: Pos, size: Size) {
        self.stack.push(self.current_frame().then(pos, size));
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    /// Size of the current drawable area, respecting the stack.
    pub fn size(&self) -> Size {
        self.current_frame().size
    }

    pub fn cursor(&self) -> Option<Pos> {
        self.cursor.map(|p| self.current_frame().global_to_local(p))
    }

    pub fn set_cursor(&mut self, pos: Option<Pos>) {
        self.cursor = pos.map(|p| self.current_frame().local_to_global(p));
    }

    /// Resize the buffer and reset its contents.
    ///
    /// The buffer's contents are reset even if the buffer is already the
    /// correct size. The stack is reset as well.
    pub fn resize(&mut self, size: Size) {
        if size == self.size {
            self.data.fill_with(Cell::default);
        } else {
            let width: usize = size.width.into();
            let height: usize = size.height.into();
            let len = width * height;

            self.size = size;
            self.data.clear();
            self.data.resize_with(len, Cell::default);
        }

        self.cursor = None;

        self.stack.clear();
    }

    /// Reset the contents and stack of the buffer.
    ///
    /// `buf.reset()` is equivalent to `buf.resize(buf.size())`.
    pub fn reset(&mut self) {
        self.resize(self.size);
    }

    /// Remove the grapheme at the specified coordinates from the buffer.
    ///
    /// Removes the entire grapheme, not just the cell at the coordinates.
    /// Preserves the style of the affected cells. Preserves the cursor. Works
    /// even if the coordinates don't point to the beginning of the grapheme.
    ///
    /// Ignores the stack.
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

    /// Write styled text to the buffer, respecting the width of individual
    /// graphemes.
    ///
    /// The initial x position is considered the first column for tab width
    /// calculations.
    pub fn write(&mut self, widthdb: &mut WidthDb, pos: Pos, styled: &Styled) {
        let frame = self.current_frame();
        let (xrange, yrange) = match frame.legal_ranges() {
            Some(ranges) => ranges,
            None => return, // No drawable area
        };
        let pos = frame.local_to_global(pos);
        if !yrange.contains(&pos.y) {
            return; // Outside of drawable area
        }
        let y = pos.y as u16;

        let mut col: usize = 0;
        for (_, style, grapheme) in styled.styled_grapheme_indices() {
            let x = pos.x + col as i32;
            let width = widthdb.grapheme_width(grapheme, col);
            col += width as usize;
            if grapheme == "\t" {
                for dx in 0..width {
                    self.write_grapheme(&xrange, x + dx as i32, y, 1, " ", style);
                }
            } else if width > 0 {
                self.write_grapheme(&xrange, x, y, width, grapheme, style);
            }
        }
    }

    /// Write a single grapheme to the buffer, respecting its width.
    ///
    /// Assumes that `pos.y` is in range.
    fn write_grapheme(
        &mut self,
        xrange: &Range<i32>,
        x: i32,
        y: u16,
        width: u8,
        grapheme: &str,
        style: Style,
    ) {
        let min_x = xrange.start;
        let max_x = xrange.end - 1; // Last possible cell

        let start_x = x;
        let end_x = x + width as i32 - 1; // Coordinate of last cell

        if start_x > max_x || end_x < min_x {
            return; // Not visible
        }

        if start_x >= min_x && end_x <= max_x {
            // Fully visible, write actual grapheme
            let base_style = self.at(start_x as u16, y).style;
            for offset in 0..width {
                let x = start_x as u16 + offset as u16;
                self.erase(x, y);
                *self.at_mut(x, y) = Cell {
                    content: grapheme.to_string().into_boxed_str(),
                    style: style.cover(base_style),
                    width,
                    offset,
                };
            }
        } else {
            // Partially visible, write empty cells with correct style
            let start_x = start_x.max(0) as u16;
            let end_x = end_x.min(max_x) as u16;
            for x in start_x..=end_x {
                let base_style = self.at(x, y).style;
                self.erase(x, y);
                *self.at_mut(x, y) = Cell {
                    style: style.cover(base_style),
                    ..Default::default()
                };
            }
        }

        if let Some(pos) = self.cursor {
            if pos.y == y as i32 && start_x <= pos.x && pos.x <= end_x {
                // The cursor lies within the bounds of the current grapheme and
                self.cursor = None;
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
