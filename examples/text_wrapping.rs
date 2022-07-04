use crossterm::event::Event;
use toss::frame::{Frame, Pos};
use toss::styled::Styled;
use toss::terminal::Terminal;

fn draw(f: &mut Frame) {
    let text = concat!(
        "This is a short paragraph in order to demonstrate unicode-aware word wrapping. ",
        "Resize your terminal to different widths to try it out. ",
        "After this sentence come two newlines, so it should always break here.\n",
        "\n",
        "Since the wrapping algorithm is aware of the Unicode Standard Annex #14, ",
        "it understands things like non-breaking spaces and word joiners: ",
        "This\u{00a0}sentence\u{00a0}is\u{00a0}separated\u{00a0}by\u{00a0}non-\u{2060}breaking\u{00a0}spaces.\n",
        "\n",
        "It can also properly handle wide graphemes (like emoji 🤔), ",
        "including ones usually displayed incorrectly by terminal emulators, like 👩‍🔬 (a female scientist emoji).",
    );

    let breaks = f.wrap(text, f.size().width.into());
    let lines = Styled::default().then(text).split_at_indices(&breaks);
    for (i, mut line) in lines.into_iter().enumerate() {
        line.trim_end();
        f.write(Pos::new(0, i as i32), line);
    }
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
