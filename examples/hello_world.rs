use crossterm::event::Event;
use crossterm::style::Stylize;
use toss::{Frame, Pos, Style, Terminal};

fn draw(f: &mut Frame) {
    f.write(Pos::new(0, 0), ("Hello world!", Style::new().green()));
    f.write(
        Pos::new(0, 1),
        ("Press any key to exit", Style::new().on_dark_blue()),
    );
    f.show_cursor(Pos::new(16, 0));
}

fn render_frame(term: &mut Terminal) {
    // Must be called before rendering, otherwise the terminal has out-of-date
    // size information and will present garbage.
    term.autoresize().unwrap();
    draw(term.frame());
    while term.present().unwrap() {
        term.autoresize().unwrap();
        draw(term.frame());
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
