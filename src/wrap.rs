//! Word wrapping for text.

use unicode_linebreak::BreakOpportunity;
use unicode_segmentation::UnicodeSegmentation;

use crate::widthdb::WidthDB;

pub fn wrap(widthdb: &mut WidthDB, text: &str, width: usize) -> Vec<usize> {
    let mut breaks = vec![];

    let mut break_options = unicode_linebreak::linebreaks(text).peekable();

    // The last valid break point encountered and its width
    let mut valid_break = None;
    let mut valid_break_width = 0;

    // Width of the line at the current grapheme (with and without trailing
    // whitespace)
    let mut current_width = 0;
    let mut current_width_trimmed = 0;

    for (gi, g) in text.grapheme_indices(true) {
        // Advance break options
        let (bi, b) = loop {
            let (bi, b) = break_options.peek().expect("not at end of string yet");
            if *bi < gi {
                break_options.next();
            } else {
                break (*bi, b);
            }
        };

        // Evaluate break options at the current position
        if bi == gi {
            match b {
                BreakOpportunity::Mandatory => {
                    breaks.push(bi);
                    valid_break = None;
                    valid_break_width = 0;
                    current_width = 0;
                    current_width_trimmed = 0;
                }
                BreakOpportunity::Allowed => {
                    valid_break = Some(bi);
                    valid_break_width = current_width;
                }
            }
        }

        // Calculate widths after current grapheme
        let g_is_whitespace = g.chars().all(|c| c.is_whitespace());
        let g_width = if g == "\t" {
            widthdb.tab_width_at_column(current_width) as usize
        } else {
            widthdb.grapheme_width(g) as usize
        };
        let g_width_trimmed = if g_is_whitespace { 0 } else { g_width };
        let mut new_width = current_width + g_width;
        let mut new_width_trimmed = if g_is_whitespace {
            current_width_trimmed
        } else {
            new_width
        };

        // Wrap at last break point if necessary
        if new_width_trimmed > width {
            if let Some(bi) = valid_break {
                breaks.push(bi);
                new_width -= valid_break_width;
                new_width_trimmed = new_width_trimmed.saturating_sub(valid_break_width);
                valid_break = None;
                valid_break_width = 0;
            }
        }

        // Perform a forced break if still necessary
        if new_width_trimmed > width {
            if new_width == g_width {
                // The grapheme is the only thing on the current line and it is
                // wider than the maximum width, so we'll allow it, thereby
                // forcing the following grapheme to break no matter what
                // (either because of a mandatory or allowed break, or via a
                // forced break).
            } else {
                // Forced break in the midde of a normally non-breakable chunk
                // because there are no valid break points.
                breaks.push(gi);
                new_width = g_width;
                new_width_trimmed = g_width_trimmed;
                valid_break = None;
                valid_break_width = 0;
            }
        }

        // Update current width
        current_width = new_width;
        current_width_trimmed = new_width_trimmed;
    }

    breaks
}
