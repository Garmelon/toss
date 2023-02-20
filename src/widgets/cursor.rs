use async_trait::async_trait;

use crate::{AsyncWidget, Frame, Pos, Size, Widget};

#[derive(Debug, Clone, Copy)]
pub struct Cursor<I> {
    pub inner: I,
    pub position: Pos,
}

impl<I> Cursor<I> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            position: Pos::ZERO,
        }
    }

    pub fn with_position(mut self, position: Pos) -> Self {
        self.position = position;
        self
    }

    pub fn with_position_xy(self, x: i32, y: i32) -> Self {
        self.with_position(Pos::new(x, y))
    }
}

impl<E, I> Widget<E> for Cursor<I>
where
    I: Widget<E>,
{
    fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        self.inner.size(frame, max_width, max_height)
    }

    fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.inner.draw(frame)?;
        frame.show_cursor(self.position);
        Ok(())
    }
}

#[async_trait]
impl<E, I> AsyncWidget<E> for Cursor<I>
where
    I: AsyncWidget<E> + Send + Sync,
{
    async fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        self.inner.size(frame, max_width, max_height).await
    }

    async fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.inner.draw(frame).await?;
        frame.show_cursor(self.position);
        Ok(())
    }
}
