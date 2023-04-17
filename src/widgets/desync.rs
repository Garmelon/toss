use async_trait::async_trait;

use crate::{AsyncWidget, Widget};

pub struct Desync<I>(pub I);

impl<E, I> Widget<E> for Desync<I>
where
    I: Widget<E>,
{
    fn size(
        &self,
        widthdb: &mut crate::WidthDb,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<crate::Size, E> {
        self.0.size(widthdb, max_width, max_height)
    }

    fn draw(self, frame: &mut crate::Frame) -> Result<(), E> {
        self.0.draw(frame)
    }
}

#[async_trait]
impl<E, I> AsyncWidget<E> for Desync<I>
where
    I: Widget<E> + Send + Sync,
{
    async fn size(
        &self,
        widthdb: &mut crate::WidthDb,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<crate::Size, E> {
        self.0.size(widthdb, max_width, max_height)
    }

    async fn draw(self, frame: &mut crate::Frame) -> Result<(), E> {
        self.0.draw(frame)
    }
}
