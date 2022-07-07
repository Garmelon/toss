//! Rendering the next frame.

use crate::buffer::Buffer;
pub use crate::buffer::{Pos, Size};
use crate::styled::Styled;
use crate::widthdb::WidthDB;
use crate::wrap;

#[derive(Debug)]
pub struct Frame {
    pub(crate) widthdb: WidthDB,
    pub(crate) buffer: Buffer,
    cursor: Option<Pos>,
    pub(crate) tab_width: u8,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            widthdb: Default::default(),
            buffer: Default::default(),
            cursor: None,
            tab_width: 8,
        }
    }
}

impl Frame {
    pub fn size(&self) -> Size {
        self.buffer.size()
    }

    pub fn reset(&mut self) {
        self.buffer.reset();
        self.cursor = None;
    }

    pub fn cursor(&self) -> Option<Pos> {
        self.cursor
    }

    pub fn set_cursor(&mut self, pos: Option<Pos>) {
        self.cursor = pos;
    }

    pub fn show_cursor(&mut self, pos: Pos) {
        self.set_cursor(Some(pos));
    }

    pub fn hide_cursor(&mut self) {
        self.set_cursor(None);
    }

    /// Determine the width of a grapheme.
    ///
    /// If the width has not been measured yet, it is estimated using the
    /// Unicode Standard Annex #11.
    pub fn grapheme_width(&mut self, grapheme: &str) -> u8 {
        self.widthdb.grapheme_width(grapheme)
    }

    /// Determine the width of a string based on its graphemes.
    ///
    /// If the width of a grapheme has not been measured yet, it is estimated
    /// using the Unicode Standard Annex #11.
    pub fn width(&mut self, s: &str) -> usize {
        self.widthdb.width(s)
    }

    pub fn tab_width_at_column(&self, col: usize) -> u8 {
        wrap::tab_width_at_column(self.tab_width, col)
    }

    pub fn wrap(&mut self, text: &str, width: usize) -> Vec<usize> {
        wrap::wrap(&mut self.widthdb, self.tab_width, text, width)
    }

    pub fn write<S: Into<Styled>>(&mut self, pos: Pos, styled: S) {
        self.buffer
            .write(&mut self.widthdb, self.tab_width, pos, &styled.into());
    }
}
