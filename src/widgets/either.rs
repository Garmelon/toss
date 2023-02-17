use async_trait::async_trait;

use crate::{AsyncWidget, Frame, Size, Widget};

pub enum Either<I1, I2> {
    First(I1),
    Second(I2),
}

impl<E, I1, I2> Widget<E> for Either<I1, I2>
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
        match self {
            Self::First(l) => l.size(frame, max_width, max_height),
            Self::Second(r) => r.size(frame, max_width, max_height),
        }
    }

    fn draw(self, frame: &mut Frame) -> Result<(), E> {
        match self {
            Self::First(l) => l.draw(frame),
            Self::Second(r) => r.draw(frame),
        }
    }
}

#[async_trait]
impl<E, I1, I2> AsyncWidget<E> for Either<I1, I2>
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
        match self {
            Self::First(l) => l.size(frame, max_width, max_height).await,
            Self::Second(r) => r.size(frame, max_width, max_height).await,
        }
    }

    async fn draw(self, frame: &mut Frame) -> Result<(), E> {
        match self {
            Self::First(l) => l.draw(frame).await,
            Self::Second(r) => r.draw(frame).await,
        }
    }
}
