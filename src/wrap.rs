//! Word wrapping for text.

use unicode_linebreak::BreakOpportunity;
use unicode_segmentation::UnicodeSegmentation;

use crate::widthdb::WidthDB;

// TODO Handle tabs separately?
// TODO Convert into an iterator?
pub fn wrap(text: &str, width: usize, widthdb: &mut WidthDB) -> Vec<usize> {
    let mut breaks = vec![];

    let mut break_options = unicode_linebreak::linebreaks(text).peekable();

    // The last valid break point encountered and its width
    let mut valid_break = None;
    let mut valid_break_width = 0;

    // Width of the line at the current grapheme
    let mut current_width = 0;

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
                }
                BreakOpportunity::Allowed => {
                    valid_break = Some(bi);
                    valid_break_width = current_width;
                }
            }
        }

        let grapheme_width: usize = widthdb.grapheme_width(g).into();
        if current_width + grapheme_width > width {
            if current_width == 0 {
                // The grapheme is wider than the maximum width, so we'll allow
                // it, thereby forcing the following grapheme to break no matter
                // what (either because of a mandatory or allowed break, or via
                // a forced break).
            } else if let Some(bi) = valid_break {
                // We can't fit the grapheme onto the current line, so we'll
                // just break at the last valid break point.
                breaks.push(bi);
                current_width -= valid_break_width;
                valid_break = None;
                valid_break_width = 0;
            } else {
                // Forced break in the midde of a normally non-breakable chunk
                // because there have been no valid break points yet.
                breaks.push(gi);
                valid_break = None;
                valid_break_width = 0;
                current_width = 0;
            }
        }

        current_width += grapheme_width;
    }

    breaks
}

pub fn split_at_indices<'a>(s: &'a str, indices: &[usize]) -> Vec<&'a str> {
    let mut slices = vec![];

    let mut rest = s;
    let mut offset = 0;

    for i in indices {
        let (left, right) = rest.split_at(i - offset);
        slices.push(left);
        rest = right;
        offset = *i;
    }

    slices.push(rest);

    slices
}
