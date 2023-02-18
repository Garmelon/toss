use async_trait::async_trait;

use crate::{AsyncWidget, Frame, Pos, Size, Style, Widget};

#[derive(Debug, Clone, Copy)]
pub struct BorderLook {
    pub top_left: &'static str,
    pub top_right: &'static str,
    pub bottom_left: &'static str,
    pub bottom_right: &'static str,
    pub top: &'static str,
    pub bottom: &'static str,
    pub left: &'static str,
    pub right: &'static str,
}

impl BorderLook {
    /// ```text
    /// +-------+
    /// | Hello |
    /// +-------+
    /// ```
    pub const ASCII: Self = Self {
        top_left: "+",
        top_right: "+",
        bottom_left: "+",
        bottom_right: "+",
        top: "-",
        bottom: "-",
        left: "|",
        right: "|",
    };

    /// ```text
    /// ┌───────┐
    /// │ Hello │
    /// └───────┘
    /// ```
    pub const LINE: Self = Self {
        top_left: "┌",
        top_right: "┐",
        bottom_left: "└",
        bottom_right: "┘",
        top: "─",
        bottom: "─",
        left: "│",
        right: "│",
    };

    /// ```text
    /// ┏━━━━━━━┓
    /// ┃ Hello ┃
    /// ┗━━━━━━━┛
    /// ```
    pub const LINE_HEAVY: Self = Self {
        top_left: "┏",
        top_right: "┓",
        bottom_left: "┗",
        bottom_right: "┛",
        top: "━",
        bottom: "━",
        left: "┃",
        right: "┃",
    };

    /// ```text
    /// ╔═══════╗
    /// ║ Hello ║
    /// ╚═══════╝
    /// ```
    pub const LINE_DOUBLE: Self = Self {
        top_left: "╔",
        top_right: "╗",
        bottom_left: "╚",
        bottom_right: "╝",
        top: "═",
        bottom: "═",
        left: "║",
        right: "║",
    };
}

impl Default for BorderLook {
    fn default() -> Self {
        Self::LINE
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Border<I> {
    inner: I,
    look: BorderLook,
    style: Style,
}

impl<I> Border<I> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            look: BorderLook::default(),
            style: Style::default(),
        }
    }

    pub fn look(mut self, look: BorderLook) -> Self {
        self.look = look;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    fn draw_border(&self, frame: &mut Frame) {
        let size = frame.size();
        let right = size.width.saturating_sub(1).into();
        let bottom = size.height.saturating_sub(1).into();

        for y in 1..bottom {
            frame.write(Pos::new(right, y), (self.look.right, self.style));
            frame.write(Pos::new(0, y), (self.look.left, self.style));
        }

        for x in 1..right {
            frame.write(Pos::new(x, bottom), (self.look.bottom, self.style));
            frame.write(Pos::new(x, 0), (self.look.top, self.style));
        }

        frame.write(
            Pos::new(right, bottom),
            (self.look.bottom_right, self.style),
        );
        frame.write(Pos::new(0, bottom), (self.look.bottom_left, self.style));
        frame.write(Pos::new(right, 0), (self.look.top_right, self.style));
        frame.write(Pos::new(0, 0), (self.look.top_left, self.style));
    }

    fn push_inner(&self, frame: &mut Frame) {
        let mut size = frame.size();
        size.width = size.width.saturating_sub(2);
        size.height = size.height.saturating_sub(2);

        frame.push(Pos::new(1, 1), size);
    }
}

impl<E, I> Widget<E> for Border<I>
where
    I: Widget<E>,
{
    fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        let max_width = max_width.map(|w| w.saturating_sub(2));
        let max_height = max_height.map(|h| h.saturating_sub(2));
        let size = self.inner.size(frame, max_width, max_height)?;
        Ok(size + Size::new(2, 2))
    }

    fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.draw_border(frame);

        self.push_inner(frame);
        self.inner.draw(frame)?;
        frame.pop();

        Ok(())
    }
}

#[async_trait]
impl<E, I> AsyncWidget<E> for Border<I>
where
    I: AsyncWidget<E> + Send + Sync,
{
    async fn size(
        &self,
        frame: &mut Frame,
        max_width: Option<u16>,
        max_height: Option<u16>,
    ) -> Result<Size, E> {
        let max_width = max_width.map(|w| w.saturating_sub(2));
        let max_height = max_height.map(|h| h.saturating_sub(2));
        let size = self.inner.size(frame, max_width, max_height).await?;
        Ok(size + Size::new(2, 2))
    }

    async fn draw(self, frame: &mut Frame) -> Result<(), E> {
        self.draw_border(frame);

        self.push_inner(frame);
        self.inner.draw(frame).await?;
        frame.pop();

        Ok(())
    }
}
