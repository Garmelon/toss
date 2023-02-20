use async_trait::async_trait;

use crate::{AsyncWidget, Frame, Size, Widget};

#[derive(Debug, Clone, Copy)]
pub struct Layer<I1, I2> {
    pub below: I1,
    pub above: I2,
}

impl<I1, I2> Layer<I1, I2> {
    pub fn new(below: I1, above: I2) -> Self {
        Self { below, above }
    }

    fn size(below: Size, above: Size) -> Size {
        Size::new(below.width.max(above.width), below.height.max(above.height))
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
        let bottom = self.below.size(frame, max_width, max_height)?;
        let top = self.above.size(frame, max_width, max_height)?;
        Ok(Self::size(bottom, top))
    }

    fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.below.draw(frame)?;
        self.above.draw(frame)?;
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
        let bottom = self.below.size(frame, max_width, max_height).await?;
        let top = self.above.size(frame, max_width, max_height).await?;
        Ok(Self::size(bottom, top))
    }

    async fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.below.draw(frame).await?;
        self.above.draw(frame).await?;
        Ok(())
    }
}
