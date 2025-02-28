use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use crossterm::QueueableCommand;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::wrap;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum WidthEstimationMethod {
    /// Estimate the width of a grapheme using legacy methods.
    ///
    /// Different terminal emulators all use different approaches to determine
    /// grapheme widths, so this method will never be able to give a fully
    /// correct solution. For that, the only possible approach is measuring the
    /// actual grapheme width.
    #[default]
    Legacy,

    /// Estimate the width of a grapheme using the unicode standard in a
    /// best-effort manner.
    Unicode,
}

/// Measures and stores the with (in terminal coordinates) of graphemes.
#[derive(Debug)]
pub struct WidthDb {
    pub(crate) estimate: WidthEstimationMethod,
    pub(crate) measure: bool,
    pub(crate) tab_width: u8,
    known: HashMap<String, u8>,
    requested: HashSet<String>,
}

impl Default for WidthDb {
    fn default() -> Self {
        Self {
            estimate: WidthEstimationMethod::default(),
            measure: false,
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
    /// If the width has not been measured yet or measurements are turned off,
    /// it is estimated using the Unicode Standard Annex #11.
    pub fn grapheme_width(&mut self, grapheme: &str, col: usize) -> u8 {
        assert_eq!(Some(grapheme), grapheme.graphemes(true).next());
        if grapheme == "\t" {
            return self.tab_width_at_column(col);
        }

        if self.measure {
            if let Some(width) = self.known.get(grapheme) {
                return *width;
            }
            self.requested.insert(grapheme.to_string());
        }

        match self.estimate {
            // A character-wise width calculation is a simple and obvious
            // approach to compute character widths. The idea is that dumb
            // terminal emulators tend to do something roughly like this, and
            // smart terminal emulators try to emulate dumb ones for
            // compatibility. In practice, this approach seems to be fairly
            // robust.
            WidthEstimationMethod::Legacy => grapheme
                .chars()
                .filter(|c| !c.is_ascii_control())
                .flat_map(|c| c.width())
                .sum::<usize>()
                .try_into()
                .unwrap_or(u8::MAX),

            // The unicode width crate considers control chars to have a width
            // of 1 even though they usually have a width of 0 when displayed.
            WidthEstimationMethod::Unicode => grapheme
                .split(|c: char| c.is_ascii_control())
                .map(|s| s.width())
                .sum::<usize>()
                .try_into()
                .unwrap_or(u8::MAX),
        }
    }

    /// Determine the width of a string based on its graphemes.
    ///
    /// If a grapheme is a tab, its column is used to determine its width.
    ///
    /// If the width of a grapheme has not been measured yet or measurements are
    /// turned off, it is estimated using the Unicode Standard Annex #11.
    pub fn width(&mut self, s: &str) -> usize {
        let mut total: usize = 0;
        for grapheme in s.graphemes(true) {
            total += self.grapheme_width(grapheme, total) as usize;
        }
        total
    }

    /// Perform primitive word wrapping with the specified maximum width.
    ///
    /// Returns the byte offsets at which the string should be split into lines.
    /// An offset of 1 would mean the first line contains only a single byte.
    /// These offsets lie on grapheme boundaries.
    ///
    /// This function does not support bidirectional script. It assumes the
    /// entire text has the same direction.
    pub fn wrap(&mut self, text: &str, width: usize) -> Vec<usize> {
        wrap::wrap(self, text, width)
    }

    /// Whether any new graphemes have been seen since the last time
    /// [`Self::measure_widths`] was called.
    pub(crate) fn measuring_required(&self) -> bool {
        self.measure && !self.requested.is_empty()
    }

    /// Measure the width of all new graphemes that have been seen since the
    /// last time this function was called.
    ///
    /// This function measures the actual width of graphemes by writing them to
    /// the terminal. After it finishes, the terminal's contents should be
    /// assumed to be garbage and a full redraw should be performed.
    pub(crate) fn measure_widths(&mut self, out: &mut impl Write) -> io::Result<()> {
        if !self.measure {
            return Ok(());
        }
        for grapheme in self.requested.drain() {
            if grapheme.chars().any(|c| c.is_ascii_control()) {
                // ASCII control characters like the escape character or the
                // bell character tend to be interpreted specially by terminals.
                // This may break width measurements. To avoid this, we just
                // assign each control character a with of 0.
                self.known.insert(grapheme, 0);
                continue;
            }

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
