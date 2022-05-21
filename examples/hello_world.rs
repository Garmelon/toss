use std::io;

use crossterm::style::{ContentStyle, Stylize};
use toss::frame::Pos;
use toss::terminal::Terminal;

fn main() -> io::Result<()> {
    // Automatically enters alternate screen and enables raw mode
    let mut term = Terminal::new()?;

    // Must be called before rendering, otherwise the terminal has out-of-date
    // size information and will present garbage.
    term.autoresize()?;

    // Render stuff onto the next frame
    let f = term.frame();
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

    // Show the next frame on the screen
    term.present()?;

    // Wait for input before exiting
    let _ = crossterm::event::read();

    Ok(())
}
