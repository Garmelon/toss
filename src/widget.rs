use async_trait::async_trait;

use crate::widgets::{
    Background, Border, Boxed, BoxedAsync, BoxedSendSync, Desync, Either2, Either3, Float,
    JoinSegment, Layer2, Padding, Resize, Title,
};
use crate::{Frame, Size, WidthDb};

// TODO Feature-gate these traits

pub trait Widget<E> {
    fn size(
        &self,
        widthdb: &mut WidthDb,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E>;

    fn draw(self, frame: &mut Frame) -> Result<(), E>;
}

#[async_trait]
pub trait AsyncWidget<E> {
    async fn size(
        &self,
        widthdb: &mut WidthDb,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E>;

    async fn draw(self, frame: &mut Frame) -> Result<(), E>;
}

pub trait WidgetExt: Sized {
    fn background(self) -> Background<Self> {
        Background::new(self)
    }

    fn border(self) -> Border<Self> {
        Border::new(self)
    }

    fn boxed<'a, E>(self) -> Boxed<'a, E>
    where
        Self: Widget<E> + 'a,
    {
        Boxed::new(self)
    }

    fn boxed_send_sync<'a, E>(self) -> BoxedSendSync<'a, E>
    where
        Self: Widget<E> + Send + Sync + 'a,
    {
        BoxedSendSync::new(self)
    }

    fn boxed_async<'a, E>(self) -> BoxedAsync<'a, E>
    where
        Self: AsyncWidget<E> + Send + Sync + 'a,
    {
        BoxedAsync::new(self)
    }

    fn desync(self) -> Desync<Self> {
        Desync(self)
    }

    fn first2<W2>(self) -> Either2<Self, W2> {
        Either2::First(self)
    }

    fn second2<W1>(self) -> Either2<W1, Self> {
        Either2::Second(self)
    }

    fn first3<W2, W3>(self) -> Either3<Self, W2, W3> {
        Either3::First(self)
    }

    fn second3<W1, W3>(self) -> Either3<W1, Self, W3> {
        Either3::Second(self)
    }

    fn third3<W1, W2>(self) -> Either3<W1, W2, Self> {
        Either3::Third(self)
    }

    fn float(self) -> Float<Self> {
        Float::new(self)
    }

    fn segment(self) -> JoinSegment<Self> {
        JoinSegment::new(self)
    }

    fn below<W>(self, above: W) -> Layer2<Self, W> {
        Layer2::new(self, above)
    }

    fn above<W>(self, below: W) -> Layer2<W, Self> {
        Layer2::new(below, self)
    }

    fn padding(self) -> Padding<Self> {
        Padding::new(self)
    }

    fn resize(self) -> Resize<Self> {
        Resize::new(self)
    }

    fn title<S: ToString>(self, title: S) -> Title<Self> {
        Title::new(self, title)
    }
}

// It would be nice if this could be restricted to types implementing Widget.
// However, Widget (and AsyncWidget) have the E type parameter, which WidgetExt
// doesn't have. We sadly can't have unconstrained type parameters like that in
// impl blocks.
//
// If WidgetExt had a type parameter E, we'd need to specify that parameter
// everywhere we use the trait. This is less ergonomic than just constructing
// the types manually.
//
// Blanket-implementing this trait is not great, but usually works fine.
impl<W> WidgetExt for W {}
