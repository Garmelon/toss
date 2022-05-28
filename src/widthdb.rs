use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use crossterm::QueueableCommand;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Measures and stores the with (in terminal coordinates) of graphemes.
#[derive(Debug, Default)]
pub struct WidthDB {
    known: HashMap<String, u8>,
    requested: HashSet<String>,
}

impl WidthDB {
    /// Determine the width of a string based on its graphemes.
    ///
    /// If the width of a grapheme has not been measured yet, it is estimated
    /// using the Unicode Standard Annex #11.
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

    /// Whether any new graphemes have been seen since the last time
    /// [`Self::measure_widths`] was called.
    pub fn measuring_required(&self) -> bool {
        !self.requested.is_empty()
    }

    /// Measure the width of all new graphemes that have been seen since the
    /// last time this function was called.
    ///
    /// This function measures the actual width of graphemes by writing them to
    /// the terminal. After it finishes, the terminal's contents should be
    /// assumed to be garbage and a full redraw should be performed.
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
