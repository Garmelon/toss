use async_trait::async_trait;

use crate::{AsyncWidget, Frame, Pos, Size, Styled, Widget, WidthDb};

#[derive(Debug, Clone)]
pub struct Text {
    styled: Styled,
    wrap: bool,
}

impl Text {
    pub fn new<S: Into<Styled>>(styled: S) -> Self {
        Self {
            styled: styled.into(),
            wrap: true,
        }
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    fn wrapped(&self, widthdb: &mut WidthDb, max_width: Option<u16>) -> Vec<Styled> {
        let max_width = max_width
            .filter(|_| self.wrap)
            .map(|w| w as usize)
            .unwrap_or(usize::MAX);

        let indices = widthdb.wrap(self.styled.text(), max_width);
        self.styled.clone().split_at_indices(&indices)
    }

    fn size(&self, widthdb: &mut WidthDb, max_width: Option<u16>) -> Size {
        let lines = self.wrapped(widthdb, max_width);

        let min_width = lines
            .iter()
            .map(|l| widthdb.width(l.text().trim_end()))
            .max()
            .unwrap_or(0);
        let min_height = lines.len();

        let min_width: u16 = min_width.try_into().unwrap_or(u16::MAX);
        let min_height: u16 = min_height.try_into().unwrap_or(u16::MAX);
        Size::new(min_width, min_height)
    }

    fn draw(self, frame: &mut Frame) {
        let size = frame.size();
        for (i, line) in self
            .wrapped(frame.widthdb(), Some(size.width))
            .into_iter()
            .enumerate()
        {
            let i: i32 = i.try_into().unwrap_or(i32::MAX);
            frame.write(Pos::new(0, i), line);
        }
    }
}

impl<E> Widget<E> for Text {
    fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        _max_height: Option<u16>,
    ) -> Result<Size, E> {
        Ok(self.size(frame.widthdb(), max_width))
    }

    fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.draw(frame);
        Ok(())
    }
}

#[async_trait]
impl<E> AsyncWidget<E> for Text {
    async fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        _max_height: Option<u16>,
    ) -> Result<Size, E> {
        Ok(self.size(frame.widthdb(), max_width))
    }

    async fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.draw(frame);
        Ok(())
    }
}
