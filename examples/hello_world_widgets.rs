use std::convert::Infallible;

use crossterm::event::Event;
use crossterm::style::Stylize;
use toss::widgets::{BorderLook, Text};
use toss::{Style, Styled, Terminal, Widget, WidgetExt};

fn widget() -> impl Widget<Infallible> {
    let styled = Styled::new("Hello world!", Style::new().dark_green())
        .then_plain("\n")
        .then("Press any key to exit", Style::new().on_dark_blue());
    Text::new(styled)
        .padding()
        .horizontal(1)
        .border()
        .look(BorderLook::LINE_DOUBLE)
        .style(Style::new().dark_red())
        .background()
        .style(Style::new().on_yellow().opaque())
        .float()
        .all(0.5)
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
