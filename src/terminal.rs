//! Displaying frames on a terminal.

use std::io::{self, Write};
use std::mem;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{
    DisableBracketedPaste, EnableBracketedPaste, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::style::{PrintStyledContent, StyledContent};
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{ExecutableCommand, QueueableCommand};

use crate::buffer::Buffer;
use crate::{AsyncWidget, Frame, Size, Widget, WidthDb};

/// Wrapper that manages terminal output.
///
/// This struct wraps around stdout (usually) and handles showing things on the
/// terminal. It cleans up after itself when droppped, so it shouldn't leave the
/// terminal in a weird state even if your program crashes.
pub struct Terminal {
    /// Render target.
    out: Box<dyn Write>,
    /// The frame being currently rendered.
    frame: Frame,
    /// Buffer from the previous frame.
    prev_frame_buffer: Buffer,
    /// When the screen is updated next, it must be cleared and redrawn fully
    /// instead of performing an incremental update.
    full_redraw: bool,
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.suspend();
    }
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        Self::with_target(Box::new(io::stdout()))
    }

    pub fn with_target(out: Box<dyn Write>) -> io::Result<Self> {
        let mut result = Self {
            out,
            frame: Frame::default(),
            prev_frame_buffer: Buffer::default(),
            full_redraw: true,
        };
        result.unsuspend()?;
        Ok(result)
    }

    pub fn suspend(&mut self) -> io::Result<()> {
        crossterm::terminal::disable_raw_mode()?;
        self.out.execute(LeaveAlternateScreen)?;
        #[cfg(not(windows))]
        {
            self.out.execute(DisableBracketedPaste)?;
            self.out.execute(PopKeyboardEnhancementFlags)?;
        }
        self.out.execute(Show)?;
        Ok(())
    }

    pub fn unsuspend(&mut self) -> io::Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        self.out.execute(EnterAlternateScreen)?;
        #[cfg(not(windows))]
        {
            self.out.execute(EnableBracketedPaste)?;
            self.out.execute(PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES,
            ))?;
        }
        self.full_redraw = true;
        Ok(())
    }

    pub fn set_tab_width(&mut self, tab_width: u8) {
        self.frame.widthdb.tab_width = tab_width;
    }

    pub fn tab_width(&self) -> u8 {
        self.frame.widthdb.tab_width
    }

    pub fn set_measuring(&mut self, active: bool) {
        self.frame.widthdb.active = active;
    }

    pub fn measuring(&self) -> bool {
        self.frame.widthdb.active
    }

    /// Resize the frame and other internal buffers if the terminal size has
    /// changed.
    pub fn autoresize(&mut self) -> io::Result<()> {
        let (width, height) = crossterm::terminal::size()?;
        let size = Size { width, height };
        if size != self.frame.size() {
            self.frame.buffer.resize(size);
            self.prev_frame_buffer.resize(size);
            self.full_redraw = true;
        }

        Ok(())
    }

    pub fn frame(&mut self) -> &mut Frame {
        &mut self.frame
    }

    pub fn widthdb(&mut self) -> &mut WidthDb {
        &mut self.frame.widthdb
    }

    /// Display the current frame on the screen and prepare the next frame.
    ///
    /// Before drawing and presenting a frame, [`Self::autoresize`] should be
    /// called. [`Self::present`] does **not** call it automatically.
    ///
    /// If width measurements are turned on, any new graphemes encountered since
    /// the last [`Self::present`] call will be measured. This can lead to the
    /// screen flickering or being mostly blank until measurements complete.
    ///
    /// Returns `true` if any new graphemes were measured. Since their widths
    /// may have changed because of the measurements, the application using this
    /// [`Terminal`] should re-draw and re-present the current frame.
    ///
    /// After calling this function, the frame returned by [`Self::frame`] will
    /// be empty again and have no cursor position.
    pub fn present(&mut self) -> io::Result<bool> {
        let measure = self.frame.widthdb.measuring_required();
        if measure {
            self.frame.widthdb.measure_widths(&mut self.out)?;
            self.full_redraw = true;
        }

        if self.full_redraw {
            io::stdout().queue(Clear(ClearType::All))?;
            self.prev_frame_buffer.reset(); // Because the screen is now empty
            self.full_redraw = false;
        }

        self.draw_differences()?;
        self.update_cursor()?;
        self.out.flush()?;

        mem::swap(&mut self.prev_frame_buffer, &mut self.frame.buffer);
        self.frame.reset();

        Ok(measure)
    }

    /// Display a [`Widget`] on the screen.
    ///
    /// Internally calls [`Self::autoresize`] and [`Self::present`], and passes
    /// on the value returned by [`Self::present`].
    pub fn present_widget<E, W>(&mut self, widget: W) -> Result<bool, E>
    where
        E: From<io::Error>,
        W: Widget<E>,
    {
        self.autoresize()?;
        widget.draw(self.frame())?;
        let dirty = self.present()?;
        Ok(dirty)
    }

    /// Display an [`AsyncWidget`] on the screen.
    ///
    /// Internally calls [`Self::autoresize`] and [`Self::present`], and passes
    /// on the value returned by [`Self::present`].
    pub async fn present_async_widget<E, W>(&mut self, widget: W) -> Result<bool, E>
    where
        E: From<io::Error>,
        W: AsyncWidget<E>,
    {
        self.autoresize()?;
        widget.draw(self.frame()).await?;
        let dirty = self.present()?;
        Ok(dirty)
    }

    fn draw_differences(&mut self) -> io::Result<()> {
        for (x, y, cell) in self.frame.buffer.cells() {
            if self.prev_frame_buffer.at(x, y) == cell {
                continue;
            }

            let content = StyledContent::new(cell.style, &cell.content as &str);
            self.out
                .queue(MoveTo(x, y))?
                .queue(PrintStyledContent(content))?;
        }
        Ok(())
    }

    fn update_cursor(&mut self) -> io::Result<()> {
        if let Some(pos) = self.frame.cursor() {
            let size = self.frame.size();
            let x_in_bounds = 0 <= pos.x && pos.x < size.width as i32;
            let y_in_bounds = 0 <= pos.y && pos.y < size.height as i32;
            if x_in_bounds && y_in_bounds {
                self.out
                    .queue(Show)?
                    .queue(MoveTo(pos.x as u16, pos.y as u16))?;
                return Ok(());
            }
        }

        self.out.queue(Hide)?;
        Ok(())
    }
}
