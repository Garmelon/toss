use async_trait::async_trait;

use crate::{AsyncWidget, Frame, Pos, Size, Widget};

pub struct Padding<I> {
    inner: I,
    left: u16,
    right: u16,
    top: u16,
    bottom: u16,
}

impl<I> Padding<I> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
        }
    }

    pub fn left(mut self, amount: u16) -> Self {
        self.left = amount;
        self
    }

    pub fn right(mut self, amount: u16) -> Self {
        self.right = amount;
        self
    }

    pub fn top(mut self, amount: u16) -> Self {
        self.top = amount;
        self
    }

    pub fn bottom(mut self, amount: u16) -> Self {
        self.bottom = amount;
        self
    }

    pub fn horizontal(self, amount: u16) -> Self {
        self.left(amount).right(amount)
    }

    pub fn vertical(self, amount: u16) -> Self {
        self.top(amount).bottom(amount)
    }

    pub fn all(self, amount: u16) -> Self {
        self.horizontal(amount).vertical(amount)
    }

    fn pad_size(&self) -> Size {
        Size::new(self.left + self.right, self.top + self.bottom)
    }

    fn push_inner(&self, frame: &mut Frame) {
        let size = frame.size();
        let pad_size = self.pad_size();
        let inner_size = size.saturating_sub(pad_size);
        frame.push(Pos::new(self.left.into(), self.top.into()), inner_size);
    }
}

impl<E, I> Widget<E> for Padding<I>
where
    I: Widget<E>,
{
    fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        let pad_size = self.pad_size();
        let max_width = max_width.map(|w| w.saturating_sub(pad_size.width));
        let max_height = max_height.map(|h| h.saturating_sub(pad_size.height));
        let size = self.inner.size(frame, max_width, max_height)?;
        Ok(size + pad_size)
    }

    fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.push_inner(frame);
        self.inner.draw(frame)?;
        frame.pop();
        Ok(())
    }
}

#[async_trait]
impl<E, I> AsyncWidget<E> for Padding<I>
where
    I: AsyncWidget<E> + Send + Sync,
{
    async fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        let pad_size = self.pad_size();
        let max_width = max_width.map(|w| w.saturating_sub(pad_size.width));
        let max_height = max_height.map(|h| h.saturating_sub(pad_size.height));
        let size = self.inner.size(frame, max_width, max_height).await?;
        Ok(size + pad_size)
    }

    async fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.push_inner(frame);
        self.inner.draw(frame).await?;
        frame.pop();
        Ok(())
    }
}
