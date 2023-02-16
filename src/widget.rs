use async_trait::async_trait;

use crate::widgets::{Border, Float, Padding};
use crate::{Frame, Size};

// TODO Feature-gate these traits

pub trait Widget<E> {
    fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E>;

    fn draw(self, frame: &mut Frame) -> Result<(), E>;
}

#[async_trait]
pub trait AsyncWidget<E> {
    async fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E>;

    async fn draw(self, frame: &mut Frame) -> Result<(), E>;
}

pub trait WidgetExt: Sized {
    fn border(self) -> Border<Self> {
        Border::new(self)
    }

    fn float(self) -> Float<Self> {
        Float::new(self)
    }

    fn padding(self) -> Padding<Self> {
        Padding::new(self)
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
