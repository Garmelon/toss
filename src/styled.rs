use crossterm::style::{ContentStyle, StyledContent};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct Chunk {
    string: String,
    style: ContentStyle,
}

impl Chunk {
    pub fn new<S: ToString>(string: S, style: ContentStyle) -> Self {
        Self {
            string: string.to_string(),
            style,
        }
    }

    pub fn plain<S: ToString>(string: S) -> Self {
        Self::new(string, ContentStyle::default())
    }

    pub fn split_at(&self, mid: usize) -> (Self, Self) {
        let (lstr, rstr) = self.string.split_at(mid);
        let left = Self {
            string: lstr.to_string(),
            style: self.style,
        };
        let right = Self {
            string: rstr.to_string(),
            style: self.style,
        };
        (left, right)
    }
}

impl From<&str> for Chunk {
    fn from(str: &str) -> Self {
        Self::plain(str)
    }
}

impl From<String> for Chunk {
    fn from(string: String) -> Self {
        Self::plain(string)
    }
}

impl From<&String> for Chunk {
    fn from(string: &String) -> Self {
        Self::plain(string)
    }
}

impl<S: ToString> From<(S,)> for Chunk {
    fn from(tuple: (S,)) -> Self {
        Self::plain(tuple.0)
    }
}

impl<S: ToString> From<(S, ContentStyle)> for Chunk {
    fn from(tuple: (S, ContentStyle)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Styled(Vec<Chunk>);

impl Styled {
    pub fn new<C: Into<Chunk>>(chunk: C) -> Self {
        Self::default().then(chunk)
    }

    pub fn then<C: Into<Chunk>>(mut self, chunk: C) -> Self {
        self.0.push(chunk.into());
        self
    }

    pub fn and_then(mut self, other: Styled) -> Self {
        self.0.extend(other.0);
        self
    }

    pub fn text(&self) -> String {
        self.0.iter().flat_map(|c| c.string.chars()).collect()
    }

    pub fn graphemes(&self) -> impl Iterator<Item = &str> {
        self.0.iter().flat_map(|c| c.string.graphemes(true))
    }

    pub fn grapheme_indices(&self) -> impl Iterator<Item = (usize, &str)> {
        self.0
            .iter()
            .scan(0, |s, c| {
                let offset = *s;
                *s += c.string.len();
                Some((offset, c))
            })
            .flat_map(|(o, c)| {
                c.string
                    .grapheme_indices(true)
                    .map(move |(gi, g)| (o + gi, g))
            })
    }

    pub fn styled_graphemes(&self) -> impl Iterator<Item = StyledContent<&str>> {
        self.0.iter().flat_map(|c| {
            c.string
                .graphemes(true)
                .map(|g| StyledContent::new(c.style, g))
        })
    }

    pub fn split_at(self, mid: usize) -> (Self, Self) {
        let mut left = vec![];
        let mut right = vec![];
        let mut offset = 0;
        for chunk in self.0 {
            let len = chunk.string.len();
            if offset >= mid {
                right.push(chunk);
            } else if offset + len > mid {
                let (lchunk, rchunk) = chunk.split_at(mid - offset);
                left.push(lchunk);
                right.push(rchunk);
            } else {
                left.push(chunk);
            }
            offset += len;
        }
        (Self(left), Self(right))
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
        while let Some(last) = self.0.last_mut() {
            let trimmed = last.string.trim_end();
            if trimmed.is_empty() {
                self.0.pop();
            } else {
                last.string = trimmed.to_string();
                break;
            }
        }
    }
}

impl<C: Into<Chunk>> From<C> for Styled {
    fn from(chunk: C) -> Self {
        Self::new(chunk)
    }
}
