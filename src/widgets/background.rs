use async_trait::async_trait;

use crate::{AsyncWidget, Frame, Pos, Size, Style, Widget, WidthDb};

#[derive(Debug, Clone, Copy)]
pub struct Background<I> {
    pub inner: I,
    pub style: Style,
}

impl<I> Background<I> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            style: Style::new().opaque(),
        }
    }

    pub fn with_style(mut self, style: Style) -> Self {
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
        widthdb: &mut WidthDb,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        self.inner.size(widthdb, max_width, max_height)
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
        widthdb: &mut WidthDb,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        self.inner.size(widthdb, max_width, max_height).await
    }

    async fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.fill(frame);
        self.inner.draw(frame).await
    }
}
