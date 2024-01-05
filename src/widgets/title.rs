use async_trait::async_trait;

use crate::{AsyncWidget, Frame, Size, Widget, WidthDb};

#[derive(Debug, Clone)]
pub struct Title<I> {
    pub inner: I,
    pub title: String,
}

impl<I> Title<I> {
    pub fn new<S: ToString>(inner: I, title: S) -> Self {
        Self {
            inner,
            title: title.to_string(),
        }
    }
}

impl<E, I> Widget<E> for Title<I>
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
        self.inner.draw(frame)?;
        frame.set_title(Some(self.title));
        Ok(())
    }
}

#[async_trait]
impl<E, I> AsyncWidget<E> for Title<I>
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
        self.inner.draw(frame).await?;
        frame.set_title(Some(self.title));
        Ok(())
    }
}
