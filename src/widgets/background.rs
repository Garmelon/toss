use async_trait::async_trait;
use crossterm::style::ContentStyle;

use crate::{AsyncWidget, Frame, Pos, Size, Widget};

pub struct Background<I> {
    inner: I,
    style: ContentStyle,
}

impl<I> Background<I> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            style: ContentStyle::default(),
        }
    }

    pub fn style(mut self, style: ContentStyle) -> Self {
        self.style = style;
        self
    }

    fn fill(&self, frame: &mut Frame) {
        let size = frame.size();
        for dy in 0..size.height {
            for dx in 0..size.width {
                frame.write(Pos::new(dx.into(), dy.into()), (" ", self.style));
            }
        }
    }
}

impl<E, I> Widget<E> for Background<I>
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
        self.fill(frame);
        self.inner.draw(frame)
    }
}

#[async_trait]
impl<E, I> AsyncWidget<E> for Background<I>
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
        self.fill(frame);
        self.inner.draw(frame).await
    }
}