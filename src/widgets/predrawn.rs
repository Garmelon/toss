use std::mem;

use async_trait::async_trait;

use crate::buffer::Buffer;
use crate::{AsyncWidget, Frame, Pos, Size, Style, Styled, Widget, WidthDb};

pub struct Predrawn {
    buffer: Buffer,
}

impl Predrawn {
    pub fn new<E, W: Widget<E>>(inner: W, frame: &mut Frame) -> Result<Self, E> {
        let mut tmp_frame = Frame::default();
        tmp_frame.buffer.resize(frame.size());
        mem::swap(&mut frame.widthdb, &mut tmp_frame.widthdb);

        inner.draw(&mut tmp_frame)?;

        mem::swap(&mut frame.widthdb, &mut tmp_frame.widthdb);

        let buffer = tmp_frame.buffer;
        Ok(Self { buffer })
    }

    pub async fn new_async<E, W: AsyncWidget<E>>(inner: W, frame: &mut Frame) -> Result<Self, E> {
        let mut tmp_frame = Frame::default();
        tmp_frame.buffer.resize(frame.size());
        mem::swap(&mut frame.widthdb, &mut tmp_frame.widthdb);

        inner.draw(&mut tmp_frame).await?;

        mem::swap(&mut frame.widthdb, &mut tmp_frame.widthdb);

        let buffer = tmp_frame.buffer;
        Ok(Self { buffer })
    }

    pub fn size(&self) -> Size {
        self.buffer.size()
    }

    fn draw_impl(&self, frame: &mut Frame) {
        for (x, y, cell) in self.buffer.cells() {
            let pos = Pos::new(x.into(), y.into());
            let style = Style {
                content_style: cell.style,
                opaque: true,
            };
            frame.write(pos, Styled::new(&cell.content, style));
        }
    }
}

impl<E> Widget<E> for Predrawn {
    fn size(
        &self,
        _widthdb: &mut WidthDb,
        _max_width: Option<u16>,
        _max_height: Option<u16>,
    ) -> Result<Size, E> {
        Ok(self.buffer.size())
    }

    fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.draw_impl(frame);
        Ok(())
    }
}

#[async_trait]
impl<E> AsyncWidget<E> for Predrawn {
    async fn size(
        &self,
        _widthdb: &mut WidthDb,
        _max_width: Option<u16>,
        _max_height: Option<u16>,
    ) -> Result<Size, E> {
        Ok(self.buffer.size())
    }

    async fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.draw_impl(frame);
        Ok(())
    }
}
