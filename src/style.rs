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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
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

pub type RichText<'a> = &'a [(TextStyle, &'a str)];

#[macro_export]
macro_rules! rich_text {
    () => { &[] };
    ($({$($style:tt)*})? $text:literal $($tok:tt)*) => {
        rich_text!(@rt [] {$($($style)*)?} $text $($tok)*)
    };

    (@rt [$($items:tt)*]) => { &[$($items)*] };
    (@rt [$($items:tt)*] {$($style:tt)*} $text:literal $($tok:tt)*) => {
        rich_text!(@rt
            [
                $($items)*
                (rich_text!(@style (crate::style::TextStyleBuilder::new()) $($style)*), $text),
            ]
            $($tok)*
        )
    };

    (@style ($($sb:tt)+)) => { $($sb)+.into_inner() };
    (@style ($($sb:tt)+) weight = $weight:ident $($rest:tt)*) => {
        rich_text!(@style ($($sb)+.font_weight(rich_text!(@weight $weight))) $($rest)*)
    };
    (@style ($($sb:tt)+) decoration = $deco:ident $($rest:ident)*) => {
        rich_text!(@style ($($sb)+.text_decoration(rich_text!(@deco $deco))) $($rest)*)
    };
    (@style ($($sb:tt)+) style = $style:ident $($rest:ident)*) => {
        rich_text!(@style ($($sb)+.font_style(rich_text!(@fstyle $style))) $($rest)*)
    };

    (@style ($($sb:tt)+) fg = # $color:literal $($rest:ident)*) => {
        rich_text!(@style ($($sb)+.foreground(crate::color::Color::from_u32($color))) $($rest)*)
    };
    (@style ($($sb:tt)+) fg = default $($rest:ident)*) => {
        rich_text!(@style ($($sb)+.foreground(crate::color::Color::Default)) $($rest)*)
    };
    (@style ($($sb:tt)+) fg = $color:ident $($rest:ident)*) => {
        rich_text!(@style ($($sb)+.foreground(crate::color::Color::Color16(crate::color::Color16::$color))) $($rest)*)
    };

    (@style ($($sb:tt)+) bg = # $color:literal $($rest:ident)*) => {
        rich_text!(@style ($($sb)+.background(crate::color::Rgb::from_u32($color))) $($rest)*)
    };
    (@style ($($sb:tt)+) bg = default $($rest:ident)*) => {
        rich_text!(@style ($($sb)+.background(crate::color::Color::Default)) $($rest)*)
    };
    (@style ($($sb:tt)+) bg = $color:ident $($rest:ident)*) => {
        rich_text!(@style ($($sb)+.background(crate::color::Color::Color16(crate::color::Color16::$color))) $($rest)*)
    };

    (@weight normal) => { crate::style::FontWeight::Normal };
    (@weight bold) => { crate::style::FontWeight::Bold };
    (@weight faint) => { crate::style::FontWeight::Faint };

    (@deco none) => { crate::style::TextDecoration::None };
    (@deco underline) => { crate::style::TextDecoration::Underline };
    (@deco dblunderline) => { crate::style::TextDecoration::DoublyUnderline };

    (@fstyle normal) => { crate::style::FontStyle::Normal };
    (@fstyle bold) => { crate::style::FontStyle::Italic };
}
