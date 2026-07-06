use crate::color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Normal,
    Bold,
    Faint,
}

impl Default for FontWeight {
    #[inline]
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDecoration {
    None,
    Underline,
    DoublyUnderline,
}

impl Default for TextDecoration {
    #[inline]
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Normal,
    Italic,
}

impl Default for FontStyle {
    #[inline]
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextStyle {
    pub font_weight: Option<FontWeight>,
    pub text_decoration: Option<TextDecoration>,
    pub font_style: Option<FontStyle>,
    pub foreground: Option<Color>,
    pub background: Option<Color>,
}

impl TextStyle {
    #[inline]
    pub fn build() -> TextStyleBuilder {
        TextStyleBuilder::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextStyleBuilder {
    inner: TextStyle
}

impl TextStyleBuilder {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn inner(&self) -> &TextStyle {
        &self.inner
    }

    #[inline]
    pub fn into_inner(self) -> TextStyle {
        self.inner
    }

    #[inline]
    pub fn font_weight(mut self, font_weight: FontWeight) -> Self {
        self.inner.font_weight = Some(font_weight);
        self
    }

    #[inline]
    pub fn text_decoration(mut self, text_decoration: TextDecoration) -> Self {
        self.inner.text_decoration = Some(text_decoration);
        self
    }

    #[inline]
    pub fn font_style(mut self, font_style: FontStyle) -> Self {
        self.inner.font_style = Some(font_style);
        self
    }

    #[inline]
    pub fn foreground(mut self, foreground: Color) -> Self {
        self.inner.foreground = Some(foreground);
        self
    }

    #[inline]
    pub fn background(mut self, background: Color) -> Self {
        self.inner.background = Some(background);
        self
    }
}

impl From<TextStyleBuilder> for TextStyle {
    #[inline]
    fn from(value: TextStyleBuilder) -> Self {
        value.into_inner()
    }
}
