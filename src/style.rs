use crossterm::style::{ContentStyle, Stylize};

fn merge_cs(base: ContentStyle, cover: ContentStyle) -> ContentStyle {
    ContentStyle {
        foreground_color: cover.foreground_color.or(base.foreground_color),
        background_color: cover.background_color.or(base.background_color),
        underline_color: cover.underline_color.or(base.underline_color),
        attributes: cover.attributes,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Style {
    pub content_style: ContentStyle,
    pub opaque: bool,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn transparent(mut self) -> Self {
        self.opaque = false;
        self
    }

    pub fn opaque(mut self) -> Self {
        self.opaque = true;
        self
    }

    pub fn cover(self, base: Self) -> Self {
        if self.opaque {
            return self;
        }

        Self {
            content_style: merge_cs(base.content_style, self.content_style),
            opaque: base.opaque,
        }
    }
}

impl AsRef<ContentStyle> for Style {
    fn as_ref(&self) -> &ContentStyle {
        &self.content_style
    }
}

impl AsMut<ContentStyle> for Style {
    fn as_mut(&mut self) -> &mut ContentStyle {
        &mut self.content_style
    }
}

impl Stylize for Style {
    type Styled = Self;

    fn stylize(self) -> Self::Styled {
        self
    }
}
