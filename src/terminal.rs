//! Displaying frames on a terminal.

use std::io::Write;
use std::{io, mem};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{
    DisableBracketedPaste, EnableBracketedPaste, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::style::{PrintStyledContent, StyledContent};
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{ExecutableCommand, QueueableCommand};

use crate::buffer::{Buffer, Size};
use crate::frame::Frame;
use crate::widthdb::WidthDb;

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

    pub fn measuring_required(&self) -> bool {
        self.frame.widthdb.measuring_required()
    }

    pub fn measure_widths(&mut self) -> io::Result<()> {
        self.frame.widthdb.measure_widths(&mut self.out)?;
        self.full_redraw = true;
        Ok(())
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
    /// Returns `true` if an immediate redraw is required.
    ///
    /// After calling this function, the frame returned by [`Self::frame`] will
    /// be empty again and have no cursor position.
    pub fn present(&mut self) -> io::Result<()> {
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

        Ok(())
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
