use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use crossterm::QueueableCommand;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::wrap;

/// Measures and stores the with (in terminal coordinates) of graphemes.
#[derive(Debug)]
pub struct WidthDb {
    pub(crate) active: bool,
    pub(crate) tab_width: u8,
    known: HashMap<String, u8>,
    requested: HashSet<String>,
}

impl Default for WidthDb {
    fn default() -> Self {
        Self {
            active: false,
            tab_width: 8,
            known: Default::default(),
            requested: Default::default(),
        }
    }
}

impl WidthDb {
    /// Determine the width of a tab character starting at the specified column.
    fn tab_width_at_column(&self, col: usize) -> u8 {
        self.tab_width - (col % self.tab_width as usize) as u8
    }

    /// Determine the width of a grapheme.
    ///
    /// If the grapheme is a tab, the column is used to determine its width.
    ///
    /// If the width has not been measured yet, it is estimated using the
    /// Unicode Standard Annex #11.
    pub fn grapheme_width(&mut self, grapheme: &str, col: usize) -> u8 {
        assert_eq!(Some(grapheme), grapheme.graphemes(true).next());
        if grapheme == "\t" {
            return self.tab_width_at_column(col);
        }
        if !self.active {
            return grapheme.width() as u8;
        }
        if let Some(width) = self.known.get(grapheme) {
            *width
        } else {
            self.requested.insert(grapheme.to_string());
            grapheme.width() as u8
        }
    }

    /// Determine the width of a string based on its graphemes.
    ///
    /// If a grapheme is a tab, its column is used to determine its width.
    ///
    /// If the width of a grapheme has not been measured yet, it is estimated
    /// using the Unicode Standard Annex #11.
    pub fn width(&mut self, s: &str) -> usize {
        let mut total: usize = 0;
        for grapheme in s.graphemes(true) {
            total += self.grapheme_width(grapheme, total) as usize;
        }
        total
    }

    pub fn wrap(&mut self, text: &str, width: usize) -> Vec<usize> {
        wrap::wrap(self, text, width)
    }

    /// Whether any new graphemes have been seen since the last time
    /// [`Self::measure_widths`] was called.
    pub(crate) fn measuring_required(&self) -> bool {
        self.active && !self.requested.is_empty()
    }

    /// Measure the width of all new graphemes that have been seen since the
    /// last time this function was called.
    ///
    /// This function measures the actual width of graphemes by writing them to
    /// the terminal. After it finishes, the terminal's contents should be
    /// assumed to be garbage and a full redraw should be performed.
    pub(crate) fn measure_widths(&mut self, out: &mut impl Write) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }
        for grapheme in self.requested.drain() {
            out.queue(Clear(ClearType::All))?
                .queue(MoveTo(0, 0))?
                .queue(Print(&grapheme))?;
            out.flush()?;
            let width = crossterm::cursor::position()?.0 as u8;
            self.known.insert(grapheme, width);
        }
        Ok(())
    }
}
