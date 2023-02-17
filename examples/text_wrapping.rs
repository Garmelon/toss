use crossterm::event::Event;
use toss::{Frame, Pos, Styled, Terminal};

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
        "including ones usually displayed incorrectly by terminal emulators, like 👩‍🔬 (a female scientist emoji).\n",
        "\n",
        "Finally, tabs are supported as well. ",
        "The following text is rendered with a tab width of 4:\n",
        "\tx\n",
        "1\tx\n",
        "12\tx\n",
        "123\tx\n",
        "1234\tx\n",
        "12345\tx\n",
        "123456\tx\n",
        "1234567\tx\n",
        "12345678\tx\n",
        "123456789\tx\n",
    );

    let width = f.size().width.into();
    let breaks = f.widthdb().wrap(text, width);
    let lines = Styled::new_plain(text).split_at_indices(&breaks);
    for (i, mut line) in lines.into_iter().enumerate() {
        line.trim_end();
        f.write(Pos::new(0, i as i32), line);
    }
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
    term.set_tab_width(4);

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
