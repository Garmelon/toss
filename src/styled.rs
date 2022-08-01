use std::iter::Peekable;
use std::{slice, vec};

use crossterm::style::{ContentStyle, StyledContent};
use unicode_segmentation::{GraphemeIndices, Graphemes, UnicodeSegmentation};

#[derive(Debug, Default, Clone)]
pub struct Styled {
    text: String,
    /// List of `(style, until)` tuples. The style should be applied to all
    /// chars in the range `prev_until..until`.
    styles: Vec<(ContentStyle, usize)>,
}

impl Styled {
    pub fn new<S: AsRef<str>>(text: S, style: ContentStyle) -> Self {
        Self::default().then(text, style)
    }

    pub fn new_plain<S: AsRef<str>>(text: S) -> Self {
        Self::default().then_plain(text)
    }

    pub fn then<S: AsRef<str>>(mut self, text: S, style: ContentStyle) -> Self {
        let text = text.as_ref();
        if !text.is_empty() {
            self.text.push_str(text);
            self.styles.push((style, self.text.len()));
        }
        self
    }

    pub fn then_plain<S: AsRef<str>>(self, text: S) -> Self {
        self.then(text, ContentStyle::default())
    }

    pub fn and_then(mut self, mut other: Styled) -> Self {
        let delta = self.text.len();
        for (_, until) in &mut other.styles {
            *until += delta;
        }

        self.text.push_str(&other.text);
        self.styles.extend(other.styles);
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn split_at(self, mid: usize) -> (Self, Self) {
        let (left_text, right_text) = self.text.split_at(mid);

        let mut left_styles = vec![];
        let mut right_styles = vec![];
        let mut from = 0;
        for (style, until) in self.styles {
            if from < mid {
                left_styles.push((style, until.max(mid)));
            }
            if mid < until {
                right_styles.push((style, until.saturating_sub(mid)));
            }
            from = until;
        }

        let left = Self {
            text: left_text.to_string(),
            styles: left_styles,
        };

        let right = Self {
            text: right_text.to_string(),
            styles: right_styles,
        };

        (left, right)
    }

    pub fn split_at_indices(self, indices: &[usize]) -> Vec<Self> {
        let mut lines = vec![];

        let mut rest = self;
        let mut offset = 0;

        for i in indices {
            let (left, right) = rest.split_at(i - offset);
            lines.push(left);
            rest = right;
            offset = *i;
        }

        lines.push(rest);

        lines
    }

    pub fn trim_end(&mut self) {
        self.text = self.text.trim_end().to_string();

        let text_len = self.text.len();
        let mut styles_len = 0;
        for (_, until) in &mut self.styles {
            styles_len += 1;
            if *until >= text_len {
                *until = text_len;
                break;
            }
        }

        while self.styles.len() > styles_len {
            self.styles.pop();
        }
    }
}

//////////////////////////////
// Iterating over graphemes //
//////////////////////////////

pub struct StyledGraphemeIndices<'a> {
    text: GraphemeIndices<'a>,
    styles: Peekable<slice::Iter<'a, (ContentStyle, usize)>>,
}

impl<'a> Iterator for StyledGraphemeIndices<'a> {
    type Item = (usize, StyledContent<&'a str>);

    fn next(&mut self) -> Option<Self::Item> {
        let (gi, grapheme) = self.text.next()?;
        let (mut style, mut until) = **self.styles.peek().expect("styles cover entire text");
        while gi >= until {
            self.styles.next();
            (style, until) = **self.styles.peek().expect("styles cover entire text");
        }
        Some((gi, StyledContent::new(style, grapheme)))
    }
}

impl Styled {
    pub fn graphemes(&self) -> Graphemes<'_> {
        self.text.graphemes(true)
    }

    pub fn grapheme_indices(&self) -> GraphemeIndices<'_> {
        self.text.grapheme_indices(true)
    }

    pub fn styled_grapheme_indices(&self) -> StyledGraphemeIndices<'_> {
        StyledGraphemeIndices {
            text: self.grapheme_indices(),
            styles: self.styles.iter().peekable(),
        }
    }
}

//////////////////////////
// Converting to Styled //
//////////////////////////

impl From<&str> for Styled {
    fn from(text: &str) -> Self {
        Self::new_plain(text)
    }
}

impl From<String> for Styled {
    fn from(text: String) -> Self {
        Self::new_plain(&text)
    }
}

impl<S: AsRef<str>> From<(S,)> for Styled {
    fn from((text,): (S,)) -> Self {
        Self::new_plain(text)
    }
}

impl<S: AsRef<str>> From<(S, ContentStyle)> for Styled {
    fn from((text, style): (S, ContentStyle)) -> Self {
        Self::new(text, style)
    }
}

impl<S: AsRef<str>> From<&[(S, ContentStyle)]> for Styled {
    fn from(segments: &[(S, ContentStyle)]) -> Self {
        let mut result = Self::default();
        for (text, style) in segments {
            result = result.then(text, *style);
        }
        result
    }
}
