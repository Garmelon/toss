use std::io::Write;
use std::{io, mem};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::{PrintStyledContent, StyledContent};
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{ExecutableCommand, QueueableCommand};

use crate::buffer::{Buffer, Size};

pub struct Terminal {
    out: Box<dyn Write>,
    /// Currently visible on screen.
    prev_buffer: Buffer,
    /// Buffer to render to.
    curr_buffer: Buffer,
    /// When the screen is updated next, it must be cleared and redrawn fully
    /// instead of performing an incremental update.
    full_redraw: bool,
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.out.execute(LeaveAlternateScreen);
        let _ = crossterm::terminal::disable_raw_mode();
    }
}

impl Terminal {
    pub fn new(out: Box<dyn Write>) -> io::Result<Self> {
        let mut result = Self {
            out,
            prev_buffer: Buffer::new(),
            curr_buffer: Buffer::new(),
            full_redraw: true,
        };
        crossterm::terminal::enable_raw_mode()?;
        result.out.execute(EnterAlternateScreen)?;
        Ok(result)
    }

    pub fn buffer(&mut self) -> &mut Buffer {
        &mut self.curr_buffer
    }

    pub fn autoresize(&mut self) -> io::Result<()> {
        let (width, height) = crossterm::terminal::size()?;
        let size = Size { width, height };
        if size != self.curr_buffer.size() {
            self.prev_buffer.resize(size);
            self.curr_buffer.resize(size);
            self.full_redraw = true;
        }

        Ok(())
    }

    /// Display the contents of the buffer on the screen and prepare rendering
    /// the next frame.
    pub fn present(&mut self) -> io::Result<()> {
        if self.full_redraw {
            io::stdout().queue(Clear(ClearType::All))?;
            self.prev_buffer.reset();
            self.full_redraw = false;
        }

        self.draw_differences()?;
        self.update_cursor()?;
        self.out.flush()?;

        mem::swap(&mut self.prev_buffer, &mut self.curr_buffer);
        self.curr_buffer.reset();

        Ok(())
    }

    fn draw_differences(&mut self) -> io::Result<()> {
        // TODO Only draw the differences
        for (x, y, cell) in self.curr_buffer.cells() {
            let content = StyledContent::new(cell.style, &cell.content as &str);
            self.out
                .queue(MoveTo(x, y))?
                .queue(PrintStyledContent(content))?;
        }
        Ok(())
    }

    fn update_cursor(&mut self) -> io::Result<()> {
        if let Some(pos) = self.curr_buffer.cursor() {
            let size = self.curr_buffer.size();
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
