use std::io;

use crossterm::style::{ContentStyle, Stylize};
use toss::buffer::Pos;
use toss::terminal::Terminal;

fn main() -> io::Result<()> {
    // Automatically enters alternate screen and enables raw mode
    let mut term = Terminal::new(Box::new(io::stdout()))?;

    // Must be called before rendering, otherwise the terminal has out-of-date
    // size information and will present garbage.
    term.autoresize()?;

    // Render things to the buffer
    let b = term.buffer();
    b.write(
        Pos::new(0, 0),
        "Hello world!",
        ContentStyle::default().green(),
    );
    b.write(
        Pos::new(0, 1),
        "Press any key to exit",
        ContentStyle::default().on_dark_blue(),
    );
    b.set_cursor(Some(Pos::new(16, 0)));

    // Show the buffer's contents on screen
    term.present()?;

    // Wait for input before exiting
    let _ = crossterm::event::read();

    Ok(())
}
