use async_trait::async_trait;

use crate::widgets::{Border, Padding};
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

    fn padding(self) -> Padding<Self> {
        Padding::new(self)
    }
}

impl<W> WidgetExt for W {}

pub trait AsyncWidgetExt: Sized {
    fn border(self) -> Border<Self> {
        Border::new(self)
    }

    fn padding(self) -> Padding<Self> {
        Padding::new(self)
    }
}

impl<W> AsyncWidgetExt for W {}
