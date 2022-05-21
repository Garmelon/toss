use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use crossterm::QueueableCommand;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Default)]
pub struct WidthDB {
    known: HashMap<String, u8>,
    requested: HashSet<String>,
}

impl WidthDB {
    pub fn width(&mut self, s: &str) -> u8 {
        let mut total = 0;
        for grapheme in s.graphemes(true) {
            total += if let Some(width) = self.known.get(grapheme) {
                *width
            } else {
                self.requested.insert(grapheme.to_string());
                grapheme.width() as u8
            };
        }
        total
    }

    pub fn measuring_required(&self) -> bool {
        !self.requested.is_empty()
    }

    pub fn measure_widths(&mut self, out: &mut impl Write) -> io::Result<()> {
        for grapheme in self.requested.drain() {
            out.queue(Clear(ClearType::All))?
                .queue(MoveTo(0, 0))?
                .queue(Print(&grapheme))?;
            out.flush()?;
            let width = crossterm::cursor::position()?.0.max(1) as u8;
            self.known.insert(grapheme, width);
        }
        Ok(())
    }
}
