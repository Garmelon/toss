use crossterm::event::Event;
use crossterm::style::{ContentStyle, Stylize};
use toss::frame::{Frame, Pos};
use toss::terminal::Terminal;

fn draw(f: &mut Frame) {
    f.write(
        Pos::new(0, 0),
        "Writing over wide graphemes removes the entire overwritten grapheme.",
    );
    let under = ContentStyle::default().white().on_dark_blue();
    let over = ContentStyle::default().black().on_dark_yellow();
    for i in 0..6 {
        let delta = i - 2;
        f.write(Pos::new(2 + i * 7, 2), ("ağ", under));
        f.write(Pos::new(2 + i * 7, 3), ("ağ", under));
        f.write(Pos::new(2 + i * 7, 4), ("ağ", under));
        f.write(Pos::new(2 + i * 7 + delta, 3), ("b", over));
        f.write(Pos::new(2 + i * 7 + delta, 4), ("ğ", over));
    }

    f.write(
        Pos::new(0, 6),
        "Wide graphemes at the edges of the screen apply their style, but are not",
    );
    f.write(Pos::new(0, 7), "actually rendered.");
    let x1 = -1;
    let x2 = f.size().width as i32 / 2 - 3;
    let x3 = f.size().width as i32 - 5;
    f.write(Pos::new(x1, 9), ("123456", under));
    f.write(Pos::new(x1, 10), ("ğğğ", under));
    f.write(Pos::new(x2, 9), ("123456", under));
    f.write(Pos::new(x2, 10), ("ğğğ", under));
    f.write(Pos::new(x3, 9), ("123456", under));
    f.write(Pos::new(x3, 10), ("ğğğ", under));

    let scientist = "ğ©âğ¬";
    f.write(
        Pos::new(0, 12),
        "Most terminals ignore the zero width joiner and display this female",
    );
    f.write(
        Pos::new(0, 13),
        "scientist emoji as a woman and a microscope: ğ©âğ¬",
    );
    for i in 0..(f.width(scientist) + 4) {
        f.write(Pos::new(2, 15 + i as i32), (scientist, under));
        f.write(Pos::new(i as i32, 15 + i as i32), ("x", over));
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
