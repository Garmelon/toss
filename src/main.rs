use std::collections::HashMap;
use std::{io, slice};

use crossterm::cursor::{self, MoveTo};
use crossterm::execute;
use crossterm::style::Print;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use unicode_blocks::UnicodeBlock;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

struct WidthDB(HashMap<String, u8>);

impl WidthDB {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn measure(&mut self, s: &str) -> anyhow::Result<()> {
        let mut stdout = io::stdout();
        for grapheme in s.graphemes(true) {
            execute!(stdout, EnterAlternateScreen, MoveTo(0, 0), Print(grapheme))?;
            let width = cursor::position()?.0 as u8;
            if width != grapheme.width() as u8 {
                self.0.insert(grapheme.to_string(), width);
            }
            execute!(stdout, LeaveAlternateScreen)?;
        }
        Ok(())
    }

    fn measure_block(&mut self, block: UnicodeBlock) -> anyhow::Result<()> {
        for c in block.start()..=block.end() {
            let c = char::from_u32(c).unwrap();
            let s = c.to_string();
            self.measure(&s)?;
        }
        Ok(())
    }
}

fn main() {
    let mut widthdb = WidthDB::new();

    // widthdb.measure_block(unicode_blocks::BASIC_LATIN);
    // widthdb.measure_block(unicode_blocks::LATIN_1_SUPPLEMENT);
    // widthdb.measure_block(unicode_blocks::LATIN_EXTENDED_A);
    // widthdb.measure_block(unicode_blocks::LATIN_EXTENDED_ADDITIONAL);
    // widthdb.measure_block(unicode_blocks::LATIN_EXTENDED_B);
    // widthdb.measure_block(unicode_blocks::LATIN_EXTENDED_C);
    // widthdb.measure_block(unicode_blocks::LATIN_EXTENDED_D);
    // widthdb.measure_block(unicode_blocks::LATIN_EXTENDED_E);
    // widthdb.measure_block(unicode_blocks::LATIN_EXTENDED_F);
    // widthdb.measure_block(unicode_blocks::LATIN_EXTENDED_G);
    // widthdb.measure_block(unicode_blocks::EMOTICONS);
    widthdb.measure_block(unicode_blocks::BOX_DRAWING);

    println!();
    for (grapheme, width) in widthdb.0 {
        let expected = grapheme.width();
        println!("{grapheme} = {width} (expected: {expected})");
    }
}
