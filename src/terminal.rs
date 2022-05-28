//! Displaying frames on a terminal.

use std::io::Write;
use std::{io, mem};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::{PrintStyledContent, StyledContent};
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{ExecutableCommand, QueueableCommand};

use crate::buffer::{Buffer, Size};
use crate::frame::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Redraw {
    Required,
    NotRequired,
}

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
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = self.out.execute(LeaveAlternateScreen);
        let _ = self.out.execute(Show);
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
        crossterm::terminal::enable_raw_mode()?;
        result.out.execute(EnterAlternateScreen)?;
        Ok(result)
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

    /// Display the current frame on the screen and prepare the next frame.
    /// Returns `true` if an immediate redraw is required.
    ///
    /// After calling this function, the frame returned by [`Self::frame`] will
    /// be empty again and have no cursor position.
    pub fn present(&mut self) -> io::Result<Redraw> {
        if self.frame.widthdb.measuring_required() {
            self.frame.widthdb.measure_widths(&mut self.out)?;
            // Since we messed up the screen by measuring widths, we'll need to
            // do a full redraw the next time around.
            self.full_redraw = true;
            // Throwing away the current frame because its content were rendered
            // with unconfirmed width data. Also, this function guarantees that
            // after it is called, the frame is empty.
            self.frame.reset();
            return Ok(Redraw::Required);
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

        Ok(Redraw::NotRequired)
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
