//! Rendering the next frame.

use crossterm::style::ContentStyle;

use crate::buffer::Buffer;
pub use crate::buffer::{Pos, Size};
use crate::widthdb::WidthDB;

#[derive(Debug, Default)]
pub struct Frame {
    pub(crate) widthdb: WidthDB,
    pub(crate) buffer: Buffer,
    cursor: Option<Pos>,
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

    pub fn write(&mut self, pos: Pos, content: &str, style: ContentStyle) {
        self.buffer.write(&mut self.widthdb, pos, content, style);
    }
}
