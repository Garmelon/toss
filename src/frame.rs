//! Rendering the next frame.

use crate::buffer::Buffer;
pub use crate::buffer::{Pos, Size};
use crate::styled::Styled;
use crate::widthdb::WidthDB;
use crate::wrap;

#[derive(Debug, Default)]
pub struct Frame {
    pub(crate) widthdb: WidthDB,
    pub(crate) buffer: Buffer,
    cursor: Option<Pos>,
}

impl Frame {
    pub fn push(&mut self, pos: Pos, size: Size) {
        self.buffer.push(pos, size);
    }

    pub fn pop(&mut self) {
        self.buffer.pop();
    }

    pub fn size(&self) -> Size {
        self.buffer.current_frame().size
    }

    pub fn reset(&mut self) {
        self.buffer.reset();
        self.cursor = None;
    }

    pub fn cursor(&self) -> Option<Pos> {
        self.cursor
            .map(|p| self.buffer.current_frame().global_to_local(p))
    }

    pub fn set_cursor(&mut self, pos: Option<Pos>) {
        self.cursor = pos.map(|p| self.buffer.current_frame().local_to_global(p));
    }

    pub fn show_cursor(&mut self, pos: Pos) {
        self.set_cursor(Some(pos));
    }

    pub fn hide_cursor(&mut self) {
        self.set_cursor(None);
    }

    /// Determine the width of a grapheme.
    ///
    /// If the grapheme is a tab, the column is used to determine its width.
    ///
    /// If the width has not been measured yet, it is estimated using the
    /// Unicode Standard Annex #11.
    pub fn grapheme_width(&mut self, grapheme: &str, col: usize) -> u8 {
        self.widthdb.grapheme_width(grapheme, col)
    }

    /// Determine the width of a string based on its graphemes.
    ///
    /// If a grapheme is a tab, its column is used to determine its width.
    ///
    /// If the width of a grapheme has not been measured yet, it is estimated
    /// using the Unicode Standard Annex #11.
    pub fn width(&mut self, s: &str) -> usize {
        self.widthdb.width(s)
    }

    pub fn wrap(&mut self, text: &str, width: usize) -> Vec<usize> {
        wrap::wrap(&mut self.widthdb, text, width)
    }

    pub fn write<S: Into<Styled>>(&mut self, pos: Pos, styled: S) {
        self.buffer.write(&mut self.widthdb, pos, &styled.into());
    }
}
