use async_trait::async_trait;

use crate::{AsyncWidget, Frame, Size, Widget};

#[derive(Debug, Clone, Copy)]
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
            Self::First(w) => w.size(frame, max_width, max_height),
            Self::Second(w) => w.size(frame, max_width, max_height),
        }
    }

    fn draw(self, frame: &mut Frame) -> Result<(), E> {
        match self {
            Self::First(w) => w.draw(frame),
            Self::Second(w) => w.draw(frame),
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
            Self::First(w) => w.size(frame, max_width, max_height).await,
            Self::Second(w) => w.size(frame, max_width, max_height).await,
        }
    }

    async fn draw(self, frame: &mut Frame) -> Result<(), E> {
        match self {
            Self::First(w) => w.draw(frame).await,
            Self::Second(w) => w.draw(frame).await,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Either3<I1, I2, I3> {
    First(I1),
    Second(I2),
    Third(I3),
}

impl<E, I1, I2, I3> Widget<E> for Either3<I1, I2, I3>
where
    I1: Widget<E>,
    I2: Widget<E>,
    I3: Widget<E>,
{
    fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        match self {
            Self::First(w) => w.size(frame, max_width, max_height),
            Self::Second(w) => w.size(frame, max_width, max_height),
            Self::Third(w) => w.size(frame, max_width, max_height),
        }
    }

    fn draw(self, frame: &mut Frame) -> Result<(), E> {
        match self {
            Self::First(w) => w.draw(frame),
            Self::Second(w) => w.draw(frame),
            Self::Third(w) => w.draw(frame),
        }
    }
}

#[async_trait]
impl<E, I1, I2, I3> AsyncWidget<E> for Either3<I1, I2, I3>
where
    I1: AsyncWidget<E> + Send + Sync,
    I2: AsyncWidget<E> + Send + Sync,
    I3: AsyncWidget<E> + Send + Sync,
{
    async fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        match self {
            Self::First(w) => w.size(frame, max_width, max_height).await,
            Self::Second(w) => w.size(frame, max_width, max_height).await,
            Self::Third(w) => w.size(frame, max_width, max_height).await,
        }
    }

    async fn draw(self, frame: &mut Frame) -> Result<(), E> {
        match self {
            Self::First(w) => w.draw(frame).await,
            Self::Second(w) => w.draw(frame).await,
            Self::Third(w) => w.draw(frame).await,
        }
    }
}
