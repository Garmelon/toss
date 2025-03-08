use crate::{Frame, Size, Widget, WidthDb};

///////////
// State //
///////////

#[derive(Debug, Default, Clone)]
pub struct BellState {
    // Whether the bell should be rung the next time the widget is displayed.
    pub ring: bool,
}

impl BellState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn widget(&mut self) -> Bell<'_> {
        Bell { state: self }
    }
}

////////////
// Widget //
////////////

#[derive(Debug)]
pub struct Bell<'a> {
    state: &'a mut BellState,
}

impl Bell<'_> {
    pub fn state(&mut self) -> &mut BellState {
        self.state
    }
}

impl<E> Widget<E> for Bell<'_> {
    fn size(
        &self,
        _widthdb: &mut WidthDb,
        _max_width: Option<u16>,
        _max_height: Option<u16>,
    ) -> Result<Size, E> {
        Ok(Size::ZERO)
    }

    fn draw(self, frame: &mut Frame) -> Result<(), E> {
        if self.state.ring {
            frame.set_bell(true);
            self.state.ring = false
        }
        Ok(())
    }
}
