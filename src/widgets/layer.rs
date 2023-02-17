use async_trait::async_trait;

use crate::{AsyncWidget, Frame, Size, Widget};

pub struct Layer<I1, I2> {
    bottom: I1,
    top: I2,
}

impl<I1, I2> Layer<I1, I2> {
    pub fn new(bottom: I1, top: I2) -> Self {
        Self { bottom, top }
    }

    fn size(bottom: Size, top: Size) -> Size {
        Size::new(bottom.width.max(top.width), bottom.height.max(top.height))
    }
}

impl<E, I1, I2> Widget<E> for Layer<I1, I2>
where
    I1: Widget<E>,
    I2: Widget<E>,
{
    fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        let bottom = self.bottom.size(frame, max_width, max_height)?;
        let top = self.top.size(frame, max_width, max_height)?;
        Ok(Self::size(bottom, top))
    }

    fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.bottom.draw(frame)?;
        self.top.draw(frame)?;
        Ok(())
    }
}

#[async_trait]
impl<E, I1, I2> AsyncWidget<E> for Layer<I1, I2>
where
    I1: AsyncWidget<E> + Send + Sync,
    I2: AsyncWidget<E> + Send + Sync,
{
    async fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        let bottom = self.bottom.size(frame, max_width, max_height).await?;
        let top = self.top.size(frame, max_width, max_height).await?;
        Ok(Self::size(bottom, top))
    }

    async fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.bottom.draw(frame).await?;
        self.top.draw(frame).await?;
        Ok(())
    }
}
