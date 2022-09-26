//! Rendering the next frame.

use crate::buffer::Buffer;
pub use crate::buffer::{Pos, Size};
use crate::styled::Styled;
use crate::widthdb::WidthDb;

#[derive(Debug, Default)]
pub struct Frame {
    pub(crate) widthdb: WidthDb,
    pub(crate) buffer: Buffer,
}

impl Frame {
    pub fn push(&mut self, pos: Pos, size: Size) {
        self.buffer.push(pos, size);
    }

    pub fn pop(&mut self) {
        self.buffer.pop();
    }

    pub fn size(&self) -> Size {
        self.buffer.size()
    }

    pub fn reset(&mut self) {
        self.buffer.reset();
    }

    pub fn cursor(&self) -> Option<Pos> {
        self.buffer.cursor()
    }

    pub fn set_cursor(&mut self, pos: Option<Pos>) {
        self.buffer.set_cursor(pos);
    }

    pub fn show_cursor(&mut self, pos: Pos) {
        self.set_cursor(Some(pos));
    }

    pub fn hide_cursor(&mut self) {
        self.set_cursor(None);
    }

    pub fn widthdb(&mut self) -> &mut WidthDb {
        &mut self.widthdb
    }

    pub fn write<S: Into<Styled>>(&mut self, pos: Pos, styled: S) {
        self.buffer.write(&mut self.widthdb, pos, &styled.into());
    }
}
