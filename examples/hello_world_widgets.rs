use std::io;

use crossterm::event::Event;
use crossterm::style::Stylize;
use toss::widgets::{BorderLook, Text};
use toss::{Style, Styled, Terminal, Widget, WidgetExt};

fn widget() -> impl Widget<io::Error> {
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
    let mut dirty = true;
    while dirty {
        dirty = term.measure_widths().unwrap();
        term.present_widget(widget()).unwrap();
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
