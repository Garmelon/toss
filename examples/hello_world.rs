use std::io;

use crossterm::style::{ContentStyle, Stylize};
use toss::frame::{Frame, Pos};
use toss::terminal::Terminal;

fn draw(f: &mut Frame) {
    f.write(
        Pos::new(0, 0),
        "Hello world!",
        ContentStyle::default().green(),
    );
    f.write(
        Pos::new(0, 1),
        "Press any key to exit",
        ContentStyle::default().on_dark_blue(),
    );
    f.show_cursor(Pos::new(16, 0));
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
