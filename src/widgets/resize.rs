use async_trait::async_trait;

use crate::{AsyncWidget, Frame, Size, Widget};

pub struct Resize<I> {
    inner: I,
    min_width: Option<u16>,
    min_height: Option<u16>,
    max_width: Option<u16>,
    max_height: Option<u16>,
}

impl<I> Resize<I> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
        }
    }

    pub fn min_width(mut self, width: u16) -> Self {
        self.min_width = Some(width);
        self
    }

    pub fn min_height(mut self, height: u16) -> Self {
        self.min_height = Some(height);
        self
    }

    pub fn max_width(mut self, width: u16) -> Self {
        self.max_width = Some(width);
        self
    }

    pub fn max_height(mut self, height: u16) -> Self {
        self.max_height = Some(height);
        self
    }

    fn resize(&self, size: Size) -> Size {
        let mut width = size.width;
        let mut height = size.height;

        if let Some(min_width) = self.min_width {
            width = width.max(min_width);
        }
        if let Some(min_height) = self.min_height {
            height = height.max(min_height);
        }

        if let Some(max_width) = self.max_width {
            width = width.min(max_width);
        }
        if let Some(max_height) = self.max_height {
            height = height.min(max_height);
        }

        Size::new(width, height)
    }
}

impl<E, I> Widget<E> for Resize<I>
where
    I: Widget<E>,
{
    fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        let size = self.inner.size(frame, max_width, max_height)?;
        Ok(self.resize(size))
    }

    fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.inner.draw(frame)
    }
}

#[async_trait]
impl<E, I> AsyncWidget<E> for Resize<I>
where
    I: AsyncWidget<E> + Send + Sync,
{
    async fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        let size = self.inner.size(frame, max_width, max_height).await?;
        Ok(self.resize(size))
    }

    async fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.inner.draw(frame).await
    }
}
