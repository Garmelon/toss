use async_trait::async_trait;

use crate::{AsyncWidget, Frame, Pos, Size, Widget};

#[derive(Debug, Clone, Copy)]
pub struct Float<I> {
    pub inner: I,
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

    pub fn horizontal(&self) -> Option<f32> {
        self.horizontal
    }

    pub fn set_horizontal(&mut self, position: Option<f32>) {
        if let Some(position) = position {
            assert!((0.0..=1.0).contains(&position));
        }
        self.horizontal = position;
    }

    pub fn vertical(&self) -> Option<f32> {
        self.vertical
    }

    pub fn set_vertical(&mut self, position: Option<f32>) {
        if let Some(position) = position {
            assert!((0.0..=1.0).contains(&position));
        }
        self.vertical = position;
    }

    pub fn with_horizontal(mut self, position: f32) -> Self {
        self.set_horizontal(Some(position));
        self
    }

    pub fn with_vertical(mut self, position: f32) -> Self {
        self.set_vertical(Some(position));
        self
    }

    pub fn with_all(self, position: f32) -> Self {
        self.with_horizontal(position).with_vertical(position)
    }

    pub fn with_left(self) -> Self {
        self.with_horizontal(0.0)
    }

    pub fn with_right(self) -> Self {
        self.with_horizontal(1.0)
    }

    pub fn with_top(self) -> Self {
        self.with_vertical(0.0)
    }

    pub fn with_bottom(self) -> Self {
        self.with_vertical(1.0)
    }

    pub fn with_center_h(self) -> Self {
        self.with_horizontal(0.5)
    }

    pub fn with_center_v(self) -> Self {
        self.with_vertical(0.5)
    }

    pub fn with_center(self) -> Self {
        self.with_all(0.5)
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
