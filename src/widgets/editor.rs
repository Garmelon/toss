use std::iter;

use crossterm::style::Stylize;
use unicode_segmentation::UnicodeSegmentation;

use crate::{Frame, Pos, Size, Style, Styled, Widget, WidthDb};

/// Like [`WidthDb::wrap`] but includes a final break index if the text ends
/// with a newline.
fn wrap(widthdb: &mut WidthDb, text: &str, width: usize) -> Vec<usize> {
    let mut breaks = widthdb.wrap(text, width);
    if text.ends_with('\n') {
        breaks.push(text.len())
    }
    breaks
}

///////////
// State //
///////////

#[derive(Debug, Clone)]
pub struct EditorState {
    text: String,

    /// Index of the cursor in the text.
    ///
    /// Must point to a valid grapheme boundary.
    cursor_idx: usize,

    /// Column of the cursor on the screen just after it was last moved
    /// horizontally.
    cursor_col: usize,

    /// Position of the cursor when the editor was last rendered.
    last_cursor_pos: Pos,
}

impl EditorState {
    pub fn new() -> Self {
        Self::with_initial_text(String::new())
    }

    pub fn with_initial_text(text: String) -> Self {
        Self {
            cursor_idx: text.len(),
            cursor_col: 0,
            last_cursor_pos: Pos::ZERO,
            text,
        }
    }

    ///////////////////////////////
    // Grapheme helper functions //
    ///////////////////////////////

    fn grapheme_boundaries(&self) -> Vec<usize> {
        self.text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain(iter::once(self.text.len()))
            .collect()
    }

    /// Ensure the cursor index lies on a grapheme boundary. If it doesn't, it
    /// is moved to the next grapheme boundary.
    ///
    /// Can handle arbitrary cursor index.
    fn move_cursor_to_grapheme_boundary(&mut self) {
        for i in self.grapheme_boundaries() {
            #[allow(clippy::comparison_chain)]
            if i == self.cursor_idx {
                // We're at a valid grapheme boundary already
                return;
            } else if i > self.cursor_idx {
                // There was no valid grapheme boundary at our cursor index, so
                // we'll take the next one we can get.
                self.cursor_idx = i;
                return;
            }
        }

        // The cursor was out of bounds, so move it to the last valid index.
        self.cursor_idx = self.text.len();
    }

    ///////////////////////////////
    // Line/col helper functions //
    ///////////////////////////////

    /// Like [`Self::grapheme_boundaries`] but for lines.
    ///
    /// Note that the last line can have a length of 0 if the text ends with a
    /// newline.
    fn line_boundaries(&self) -> Vec<usize> {
        let newlines = self
            .text
            .char_indices()
            .filter(|(_, c)| *c == '\n')
            .map(|(i, _)| i + 1); // utf-8 encodes '\n' as a single byte
        iter::once(0)
            .chain(newlines)
            .chain(iter::once(self.text.len()))
            .collect()
    }

    /// Find the cursor's current line.
    ///
    /// Returns `(line_nr, start_idx, end_idx)`.
    fn cursor_line(&self, boundaries: &[usize]) -> (usize, usize, usize) {
        let mut result = (0, 0, 0);
        for (i, (start, end)) in boundaries.iter().zip(boundaries.iter().skip(1)).enumerate() {
            if self.cursor_idx >= *start {
                result = (i, *start, *end);
            } else {
                break;
            }
        }
        result
    }

    fn cursor_col(&self, widthdb: &mut WidthDb, line_start: usize) -> usize {
        widthdb.width(&self.text[line_start..self.cursor_idx])
    }

    fn line(&self, line: usize) -> (usize, usize) {
        let boundaries = self.line_boundaries();
        boundaries
            .iter()
            .copied()
            .zip(boundaries.iter().copied().skip(1))
            .nth(line)
            .expect("line exists")
    }

    fn move_cursor_to_line_col(&mut self, widthdb: &mut WidthDb, line: usize, col: usize) {
        let (start, end) = self.line(line);
        let line = &self.text[start..end];

        let mut width = 0;
        for (gi, g) in line.grapheme_indices(true) {
            self.cursor_idx = start + gi;
            if col > width {
                width += widthdb.grapheme_width(g, width) as usize;
            } else {
                return;
            }
        }

        if !line.ends_with('\n') {
            self.cursor_idx = end;
        }
    }

    fn record_cursor_col(&mut self, widthdb: &mut WidthDb) {
        let boundaries = self.line_boundaries();
        let (_, start, _) = self.cursor_line(&boundaries);
        self.cursor_col = self.cursor_col(widthdb, start);
    }

    /////////////
    // Editing //
    /////////////

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, widthdb: &mut WidthDb, text: String) {
        self.text = text;
        self.move_cursor_to_grapheme_boundary();
        self.record_cursor_col(widthdb);
    }

    pub fn clear(&mut self) {
        self.text = String::new();
        self.cursor_idx = 0;
        self.cursor_col = 0;
    }

    /// Insert a character at the current cursor position and move the cursor
    /// accordingly.
    pub fn insert_char(&mut self, widthdb: &mut WidthDb, ch: char) {
        self.text.insert(self.cursor_idx, ch);
        self.cursor_idx += ch.len_utf8();
        self.record_cursor_col(widthdb);
    }

    /// Insert a string at the current cursor position and move the cursor
    /// accordingly.
    pub fn insert_str(&mut self, widthdb: &mut WidthDb, str: &str) {
        self.text.insert_str(self.cursor_idx, str);
        self.cursor_idx += str.len();
        self.record_cursor_col(widthdb);
    }

    /// Delete the grapheme before the cursor position.
    pub fn backspace(&mut self, widthdb: &mut WidthDb) {
        let boundaries = self.grapheme_boundaries();
        for (start, end) in boundaries.iter().zip(boundaries.iter().skip(1)) {
            if *end == self.cursor_idx {
                self.text.replace_range(start..end, "");
                self.cursor_idx = *start;
                self.record_cursor_col(widthdb);
                break;
            }
        }
    }

    /// Delete the grapheme after the cursor position.
    pub fn delete(&mut self) {
        let boundaries = self.grapheme_boundaries();
        for (start, end) in boundaries.iter().zip(boundaries.iter().skip(1)) {
            if *start == self.cursor_idx {
                self.text.replace_range(start..end, "");
                break;
            }
        }
    }

    /////////////////////
    // Cursor movement //
    /////////////////////

    pub fn move_cursor_left(&mut self, widthdb: &mut WidthDb) {
        let boundaries = self.grapheme_boundaries();
        for (start, end) in boundaries.iter().zip(boundaries.iter().skip(1)) {
            if *end == self.cursor_idx {
                self.cursor_idx = *start;
                self.record_cursor_col(widthdb);
                break;
            }
        }
    }

    pub fn move_cursor_right(&mut self, widthdb: &mut WidthDb) {
        let boundaries = self.grapheme_boundaries();
        for (start, end) in boundaries.iter().zip(boundaries.iter().skip(1)) {
            if *start == self.cursor_idx {
                self.cursor_idx = *end;
                self.record_cursor_col(widthdb);
                break;
            }
        }
    }

    pub fn move_cursor_left_a_word(&mut self, widthdb: &mut WidthDb) {
        let boundaries = self.grapheme_boundaries();
        let mut encountered_word = false;
        for (start, end) in boundaries.iter().zip(boundaries.iter().skip(1)).rev() {
            if *end == self.cursor_idx {
                let g = &self.text[*start..*end];
                let whitespace = g.chars().all(|c| c.is_whitespace());
                if encountered_word && whitespace {
                    break;
                } else if !whitespace {
                    encountered_word = true;
                }
                self.cursor_idx = *start;
            }
        }
        self.record_cursor_col(widthdb);
    }

    pub fn move_cursor_right_a_word(&mut self, widthdb: &mut WidthDb) {
        let boundaries = self.grapheme_boundaries();
        let mut encountered_word = false;
        for (start, end) in boundaries.iter().zip(boundaries.iter().skip(1)) {
            if *start == self.cursor_idx {
                let g = &self.text[*start..*end];
                let whitespace = g.chars().all(|c| c.is_whitespace());
                if encountered_word && whitespace {
                    break;
                } else if !whitespace {
                    encountered_word = true;
                }
                self.cursor_idx = *end;
            }
        }
        self.record_cursor_col(widthdb);
    }

    pub fn move_cursor_to_start_of_line(&mut self, widthdb: &mut WidthDb) {
        let boundaries = self.line_boundaries();
        let (line, _, _) = self.cursor_line(&boundaries);
        self.move_cursor_to_line_col(widthdb, line, 0);
        self.record_cursor_col(widthdb);
    }

    pub fn move_cursor_to_end_of_line(&mut self, widthdb: &mut WidthDb) {
        let boundaries = self.line_boundaries();
        let (line, _, _) = self.cursor_line(&boundaries);
        self.move_cursor_to_line_col(widthdb, line, usize::MAX);
        self.record_cursor_col(widthdb);
    }

    pub fn move_cursor_up(&mut self, widthdb: &mut WidthDb) {
        let boundaries = self.line_boundaries();
        let (line, _, _) = self.cursor_line(&boundaries);
        if line > 0 {
            self.move_cursor_to_line_col(widthdb, line - 1, self.cursor_col);
        }
    }

    pub fn move_cursor_down(&mut self, widthdb: &mut WidthDb) {
        let boundaries = self.line_boundaries();

        // There's always at least one line, and always at least two line
        // boundaries at 0 and self.text.len().
        let amount_of_lines = boundaries.len() - 1;

        let (line, _, _) = self.cursor_line(&boundaries);
        if line + 1 < amount_of_lines {
            self.move_cursor_to_line_col(widthdb, line + 1, self.cursor_col);
        }
    }

    pub fn last_cursor_pos(&self) -> Pos {
        self.last_cursor_pos
    }

    pub fn widget(&mut self) -> Editor<'_> {
        Editor {
            highlighted: Styled::new_plain(&self.text),
            hidden: None,
            focus: true,
            state: self,
        }
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}

////////////
// Widget //
////////////

#[derive(Debug)]
pub struct Editor<'a> {
    state: &'a mut EditorState,
    highlighted: Styled,
    pub hidden: Option<Styled>,
    pub focus: bool,
}

impl Editor<'_> {
    pub fn state(&mut self) -> &mut EditorState {
        self.state
    }

    pub fn text(&self) -> &Styled {
        &self.highlighted
    }

    pub fn highlight<F>(&mut self, highlight: F)
    where
        F: FnOnce(&str) -> Styled,
    {
        self.highlighted = highlight(&self.state.text);
        assert_eq!(self.state.text, self.highlighted.text());
    }

    pub fn with_highlight<F>(mut self, highlight: F) -> Self
    where
        F: FnOnce(&str) -> Styled,
    {
        self.highlight(highlight);
        self
    }

    pub fn with_visible(mut self) -> Self {
        self.hidden = None;
        self
    }

    pub fn with_hidden<S: Into<Styled>>(mut self, placeholder: S) -> Self {
        self.hidden = Some(placeholder.into());
        self
    }

    pub fn with_hidden_default_placeholder(self) -> Self {
        self.with_hidden(("<hidden>", Style::new().grey().italic()))
    }

    pub fn with_focus(mut self, active: bool) -> Self {
        self.focus = active;
        self
    }

    fn wrapped_cursor(cursor_idx: usize, break_indices: &[usize]) -> (usize, usize) {
        let mut row = 0;
        let mut line_idx = cursor_idx;

        for break_idx in break_indices {
            if cursor_idx < *break_idx {
                break;
            } else {
                row += 1;
                line_idx = cursor_idx - break_idx;
            }
        }

        (row, line_idx)
    }

    fn indices(&self, widthdb: &mut WidthDb, max_width: Option<u16>) -> Vec<usize> {
        let max_width = max_width
            // One extra column for cursor
            .map(|w| w.saturating_sub(1) as usize)
            .unwrap_or(usize::MAX);
        wrap(widthdb, self.state.text(), max_width)
    }

    fn rows(&self, indices: &[usize]) -> Vec<Styled> {
        let text = match self.hidden.as_ref() {
            Some(hidden) if !self.highlighted.text().is_empty() => hidden,
            _ => &self.highlighted,
        };
        text.clone().split_at_indices(indices)
    }

    fn cursor(&self, widthdb: &mut WidthDb, width: u16, indices: &[usize], rows: &[Styled]) -> Pos {
        if self.hidden.is_some() {
            return Pos::new(0, 0);
        }

        let (cursor_row, cursor_line_idx) = Self::wrapped_cursor(self.state.cursor_idx, indices);
        let cursor_col = widthdb.width(rows[cursor_row].text().split_at(cursor_line_idx).0);

        // Ensure the cursor is always visible
        let cursor_col = cursor_col.min(width.saturating_sub(1).into());

        let cursor_row: i32 = cursor_row.try_into().unwrap_or(i32::MAX);
        let cursor_col: i32 = cursor_col.try_into().unwrap_or(i32::MAX);
        Pos::new(cursor_col, cursor_row)
    }
}

impl<E> Widget<E> for Editor<'_> {
    fn size(
        &self,
        widthdb: &mut WidthDb,
        max_width: Option<u16>,
        _max_height: Option<u16>,
    ) -> Result<Size, E> {
        let indices = self.indices(widthdb, max_width);
        let rows = self.rows(&indices);

        let width = rows
            .iter()
            .map(|row| widthdb.width(row.text()))
            .max()
            .unwrap_or(0)
            // One extra column for cursor
            .saturating_add(1);
        let height = rows.len();

        let width: u16 = width.try_into().unwrap_or(u16::MAX);
        let height: u16 = height.try_into().unwrap_or(u16::MAX);
        Ok(Size::new(width, height))
    }

    fn draw(mut self, frame: &mut Frame) -> Result<(), E> {
        let size = frame.size();
        let indices = self.indices(frame.widthdb(), Some(size.width));
        let rows = self.rows(&indices);
        let cursor = self.cursor(frame.widthdb(), size.width, &indices, &rows);

        for (i, row) in rows.into_iter().enumerate() {
            frame.write(Pos::new(0, i as i32), row);
        }

        if self.focus {
            frame.set_cursor(Some(cursor));
        }
        self.state.last_cursor_pos = cursor;

        Ok(())
    }
}
