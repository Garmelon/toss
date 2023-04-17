use async_trait::async_trait;

use crate::{AsyncWidget, Frame, Size, Widget, WidthDb};

pub struct Boxed<'a, E>(Box<dyn WidgetWrapper<E> + 'a>);

impl<'a, E> Boxed<'a, E> {
    pub fn new<I>(inner: I) -> Self
    where
        I: Widget<E> + 'a,
    {
        Self(Box::new(inner))
    }
}

impl<E> Widget<E> for Boxed<'_, E> {
    fn size(
        &self,
        widthdb: &mut WidthDb,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        self.0.wrap_size(widthdb, max_width, max_height)
    }

    fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.0.wrap_draw(frame)
    }
}

pub struct BoxedSendSync<'a, E>(Box<dyn WidgetWrapper<E> + Send + Sync + 'a>);

impl<'a, E> BoxedSendSync<'a, E> {
    pub fn new<I>(inner: I) -> Self
    where
        I: Widget<E> + Send + Sync + 'a,
    {
        Self(Box::new(inner))
    }
}

impl<E> Widget<E> for BoxedSendSync<'_, E> {
    fn size(
        &self,
        widthdb: &mut WidthDb,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        self.0.wrap_size(widthdb, max_width, max_height)
    }

    fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.0.wrap_draw(frame)
    }
}

pub struct BoxedAsync<'a, E>(Box<dyn AsyncWidgetWrapper<E> + Send + Sync + 'a>);

impl<'a, E> BoxedAsync<'a, E> {
    pub fn new<I>(inner: I) -> Self
    where
        I: AsyncWidget<E> + Send + Sync + 'a,
    {
        Self(Box::new(inner))
    }
}

#[async_trait]
impl<E> AsyncWidget<E> for BoxedAsync<'_, E> {
    async fn size(
        &self,
        widthdb: &mut WidthDb,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        self.0.wrap_size(widthdb, max_width, max_height).await
    }

    async fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.0.wrap_draw(frame).await
    }
}

trait WidgetWrapper<E> {
    fn wrap_size(
        &self,
        widthdb: &mut WidthDb,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E>;

    fn wrap_draw(self: Box<Self>, frame: &mut Frame) -> Result<(), E>;
}

impl<E, W> WidgetWrapper<E> for W
where
    W: Widget<E>,
{
    fn wrap_size(
        &self,
        widthdb: &mut WidthDb,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        self.size(widthdb, max_width, max_height)
    }

    fn wrap_draw(self: Box<Self>, frame: &mut Frame) -> Result<(), E> {
        (*self).draw(frame)
    }
}

#[async_trait]
trait AsyncWidgetWrapper<E> {
    async fn wrap_size(
        &self,
        widthdb: &mut WidthDb,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E>;

    async fn wrap_draw(self: Box<Self>, frame: &mut Frame) -> Result<(), E>;
}

#[async_trait]
impl<E, W> AsyncWidgetWrapper<E> for W
where
    W: AsyncWidget<E> + Send + Sync,
{
    async fn wrap_size(
        &self,
        widthdb: &mut WidthDb,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        self.size(widthdb, max_width, max_height).await
    }

    async fn wrap_draw(self: Box<Self>, frame: &mut Frame) -> Result<(), E> {
        (*self).draw(frame).await
    }
}
