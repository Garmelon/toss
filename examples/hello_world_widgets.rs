use std::convert::Infallible;

use crossterm::event::Event;
use crossterm::style::{ContentStyle, Stylize};
use toss::widgets::{Border, BorderLook, Text};
use toss::{Styled, Terminal, Widget};

fn widget() -> impl Widget<Infallible> {
    Border::new(Text::new(
        Styled::new("Hello world!", ContentStyle::default().green())
            .then_plain("\n")
            .then(
                "Press any key to exit",
                ContentStyle::default().on_dark_blue(),
            ),
    ))
    .look(BorderLook::LINE_DOUBLE)
    .style(ContentStyle::default().dark_red())
}

fn render_frame(term: &mut Terminal) {
    loop {
        // Must be called before rendering, otherwise the terminal has out-of-date
        // size information and will present garbage.
        term.autoresize().unwrap();

        widget().draw(term.frame()).unwrap();
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
