//! Word wrapping for text.

use unicode_linebreak::BreakOpportunity;
use unicode_segmentation::UnicodeSegmentation;

use crate::widthdb::WidthDb;

pub fn wrap(widthdb: &mut WidthDb, text: &str, width: usize) -> Vec<usize> {
    let mut breaks = vec![];

    let mut break_options = unicode_linebreak::linebreaks(text).peekable();

    // The last valid break point encountered and its width
    let mut valid_break = None;

    // Starting index and width of the line at the current grapheme (with and
    // without trailing whitespace)
    let mut current_start = 0;
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
                    current_start = bi;
                    current_width = 0;
                    current_width_trimmed = 0;
                }
                BreakOpportunity::Allowed => {
                    valid_break = Some(bi);
                }
            }
        }

        // Calculate widths after current grapheme
        let g_is_whitespace = g.chars().all(|c| c.is_whitespace());
        let g_width = widthdb.grapheme_width(g, current_width) as usize;
        current_width += g_width;
        if !g_is_whitespace {
            current_width_trimmed = current_width;
        }

        // Wrap at last break point if necessary
        if current_width_trimmed > width {
            if let Some(bi) = valid_break {
                let new_line = &text[bi..gi + g.len()];

                breaks.push(bi);
                valid_break = None;
                current_start = bi;
                current_width = widthdb.width(new_line);
                current_width_trimmed = widthdb.width(new_line.trim_end());
            }
        }

        // Perform a forced break if still necessary
        if current_width_trimmed > width {
            if current_start == gi {
                // The grapheme is the only thing on the current line and it is
                // wider than the maximum width, so we'll allow it, thereby
                // forcing the following grapheme to break no matter what
                // (either because of a mandatory or allowed break, or via a
                // forced break).
            } else {
                // Forced break in the middle of a normally non-breakable chunk
                // because there are no valid break points.
                breaks.push(gi);
                valid_break = None;
                current_start = gi;
                current_width = widthdb.grapheme_width(g, 0).into();
                current_width_trimmed = if g_is_whitespace { 0 } else { current_width };
            }
        }
    }

    breaks
}
