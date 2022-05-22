use std::io;

use crossterm::style::{ContentStyle, Stylize};
use toss::frame::{Frame, Pos};
use toss::terminal::Terminal;

fn draw(f: &mut Frame) {
    f.write(
        Pos::new(0, 0),
        "Writing over wide graphemes removes the entire overwritten grapheme.",
        ContentStyle::default(),
    );
    let under = ContentStyle::default().white().on_dark_blue();
    let over = ContentStyle::default().black().on_dark_yellow();
    for i in 0..6 {
        let delta = i - 2;
        f.write(Pos::new(2 + i * 7, 2), "a😀", under);
        f.write(Pos::new(2 + i * 7, 3), "a😀", under);
        f.write(Pos::new(2 + i * 7, 4), "a😀", under);
        f.write(Pos::new(2 + i * 7 + delta, 3), "b", over);
        f.write(Pos::new(2 + i * 7 + delta, 4), "😈", over);
    }

    f.write(
        Pos::new(0, 6),
        "Wide graphemes at the edges of the screen apply their style, but are not",
        ContentStyle::default(),
    );
    f.write(
        Pos::new(0, 7),
        "actually rendered.",
        ContentStyle::default(),
    );
    let x1 = -1;
    let x2 = f.size().width as i32 / 2 - 3;
    let x3 = f.size().width as i32 - 5;
    f.write(Pos::new(x1, 9), "123456", under);
    f.write(Pos::new(x1, 10), "😀😀😀", under);
    f.write(Pos::new(x2, 9), "123456", under);
    f.write(Pos::new(x2, 10), "😀😀😀", under);
    f.write(Pos::new(x3, 9), "123456", under);
    f.write(Pos::new(x3, 10), "😀😀😀", under);

    let scientist = "👩‍🔬";
    f.write(
        Pos::new(0, 12),
        "Most terminals ignore the zero width joiner and display this female",
        ContentStyle::default(),
    );
    f.write(
        Pos::new(0, 13),
        "scientist emoji as a woman and a microscope: 👩‍🔬",
        ContentStyle::default(),
    );
    for i in 0..(f.width(scientist) + 4) {
        f.write(Pos::new(2, 15 + i as i32), scientist, under);
        f.write(Pos::new(i as i32, 15 + i as i32), "x", over);
    }
}

fn main() -> io::Result<()> {
    // Automatically enters alternate screen and enables raw mode
    let mut term = Terminal::new()?;

    loop {
        // Must be called before rendering, otherwise the terminal has out-of-date
        // size information and will present garbage.
        term.autoresize()?;

        draw(term.frame());

        if !term.present()? {
            break;
        }
    }

    // Wait for input before exiting
    let _ = crossterm::event::read();

    Ok(())
}
