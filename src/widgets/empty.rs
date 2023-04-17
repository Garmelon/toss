use crate::{Frame, Size, Widget, WidthDb};

#[derive(Debug, Default, Clone, Copy)]
pub struct Empty {
    pub size: Size,
}

impl Empty {
    pub fn new() -> Self {
        Self { size: Size::ZERO }
    }

    pub fn with_width(mut self, width: u16) -> Self {
        self.size.width = width;
        self
    }

    pub fn with_height(mut self, height: u16) -> Self {
        self.size.height = height;
        self
    }

    pub fn with_size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }
}

impl<E> Widget<E> for Empty {
    fn size(
        &self,
        _widthdb: &mut WidthDb,
        _max_width: Option<u16>,
        _max_height: Option<u16>,
    ) -> Result<Size, E> {
        Ok(self.size)
    }

    fn draw(self, _frame: &mut Frame) -> Result<(), E> {
        Ok(())
    }
}
