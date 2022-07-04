use crossterm::event::Event;
use crossterm::style::{ContentStyle, Stylize};
use toss::frame::{Frame, Pos};
use toss::terminal::Terminal;

fn draw(f: &mut Frame) {
    f.write(
        Pos::new(0, 0),
        ("Hello world!", ContentStyle::default().green()),
    );
    f.write(
        Pos::new(0, 1),
        (
            "Press any key to exit",
            ContentStyle::default().on_dark_blue(),
        ),
    );
    f.show_cursor(Pos::new(16, 0));
}

fn render_frame(term: &mut Terminal) {
    loop {
        // Must be called before rendering, otherwise the terminal has out-of-date
        // size information and will present garbage.
        term.autoresize().unwrap();

        draw(term.frame());
        term.present().unwrap();

        if term.measuring_required() {
            term.measure_widths().unwrap();
        } else {
            break;
        }
    }
}

fn main() {
    // Automatically enters alternate screen and enables raw mode
    let mut term = Terminal::new().unwrap();
    term.set_measuring(true);

    loop {
        // Render and display a frame. A full frame is displayed on the terminal
        // once this function exits.
        render_frame(&mut term);

        // Exit if the user presses any buttons
        if !matches!(crossterm::event::read().unwrap(), Event::Resize(_, _)) {
            break;
        }
    }
}
