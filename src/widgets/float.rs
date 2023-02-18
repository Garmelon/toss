use async_trait::async_trait;

use crate::{AsyncWidget, Frame, Pos, Size, Widget};

#[derive(Debug, Clone, Copy)]
pub struct Float<I> {
    inner: I,
    horizontal: Option<f32>,
    vertical: Option<f32>,
}

impl<I> Float<I> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            horizontal: None,
            vertical: None,
        }
    }

    pub fn horizontal(mut self, position: f32) -> Self {
        self.horizontal = Some(position);
        self
    }

    pub fn vertical(mut self, position: f32) -> Self {
        self.vertical = Some(position);
        self
    }

    pub fn all(self, position: f32) -> Self {
        self.horizontal(position).vertical(position)
    }

    pub fn left(self) -> Self {
        self.horizontal(0.0)
    }

    pub fn right(self) -> Self {
        self.horizontal(1.0)
    }

    pub fn top(self) -> Self {
        self.vertical(0.0)
    }

    pub fn bottom(self) -> Self {
        self.vertical(1.0)
    }

    pub fn center_h(self) -> Self {
        self.horizontal(0.5)
    }

    pub fn center_v(self) -> Self {
        self.vertical(0.5)
    }

    pub fn center(self) -> Self {
        self.all(0.5)
    }

    fn push_inner(&self, frame: &mut Frame, size: Size, mut inner_size: Size) {
        let mut inner_pos = Pos::ZERO;

        if let Some(horizontal) = self.horizontal {
            let available = (size.width - inner_size.width) as f32;
            // Biased towards the left if horizontal lands exactly on the
            // boundary between two cells
            inner_pos.x = (horizontal * available).floor().min(available) as i32;
            inner_size.width = inner_size.width.min(size.width);
        } else {
            inner_size.width = size.width;
        }

        if let Some(vertical) = self.vertical {
            let available = (size.height - inner_size.height) as f32;
            // Biased towards the top if vertical lands exactly on the boundary
            // between two cells
            inner_pos.y = (vertical * available).floor().min(available) as i32;
            inner_size.height = inner_size.height.min(size.height);
        } else {
            inner_size.height = size.height;
        }

        frame.push(inner_pos, inner_size);
    }
}

impl<E, I> Widget<E> for Float<I>
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
        let size = frame.size();
        let inner_size = self
            .inner
            .size(frame, Some(size.width), Some(size.height))?;

        self.push_inner(frame, size, inner_size);
        self.inner.draw(frame)?;
        frame.pop();

        Ok(())
    }
}

#[async_trait]
impl<E, I> AsyncWidget<E> for Float<I>
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
        let size = frame.size();
        let inner_size = self
            .inner
            .size(frame, Some(size.width), Some(size.height))
            .await?;

        self.push_inner(frame, size, inner_size);
        self.inner.draw(frame).await?;
        frame.pop();

        Ok(())
    }
}
