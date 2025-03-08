//! Rendering the next frame.

use crate::buffer::Buffer;
use crate::{Pos, Size, Styled, WidthDb};

#[derive(Debug, Default)]
pub struct Frame {
    pub(crate) widthdb: WidthDb,
    pub(crate) buffer: Buffer,
    pub(crate) title: Option<String>,
    pub(crate) bell: bool,
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
        self.title = None;
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

    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    pub fn set_bell(&mut self, bell: bool) {
        self.bell = bell;
    }

    pub fn widthdb(&mut self) -> &mut WidthDb {
        &mut self.widthdb
    }

    pub fn write<S: Into<Styled>>(&mut self, pos: Pos, styled: S) {
        self.buffer.write(&mut self.widthdb, pos, &styled.into());
    }
}
