use crate::{color::{Color, Color16}, style::{FontStyle, FontWeight, TextDecoration, TextStyle}};

/// Simple rich text format, a bit like BB code.
/// 
/// Tags:
/// 
/// * `[b]bold[/b]`
/// * `[f]faint[/f]` (not sure about the tag name)
/// * `[i]italic[/i]`
/// * `[u]underline[/u]`
/// * `[du]doubly underline[/du]`
/// * `[color=#FF0000]foreground color hex code[/color]`
/// * `[color=red]foreground color name, can be default[/color]`
/// * `[bg=#0000FF]background color hex code[/bg]`
/// * `[bg=blue]background color name, can be default[/bg]`
/// * `[[` a single open bracket (`[`)
/// * `]]` a single close bracket (`]`)
#[derive(Debug)]
pub struct RichText(Vec<RichTextCode>);

impl RichText {
    pub fn parse(style: &RichTextStyle, rich_text: &str) -> Result<Self, ParseError> {
        let mut current_style = *style;
        let mut stack: Vec<(Tag, RichTextStyle)> = Vec::new();
        let mut code = Vec::new();

        let mut index = 0;
        let mut buf = String::new();

        while index < rich_text.len() {
            let old_index = index;

            let Some(bracket_index) = rich_text[index..].find(|c| c == '[' || c == ']') else {
                buf.push_str(&rich_text[old_index..]);
                break;
            };
            let bracket_index = bracket_index + index;

            index = bracket_index + 1;

            buf.push_str(&rich_text[old_index..bracket_index]);

            if rich_text[bracket_index..].starts_with(']') {
                if !rich_text[index..].starts_with(']') {
                    return Err(ParseError::new(ParseErrorKind::SyntaxError, index, rich_text));
                }

                buf.push(']');

                index += 1;
            } else if rich_text[index..].starts_with('[') {
                buf.push('[');

                index += 1;
            } else {
                if !buf.is_empty() {
                    code.push(RichTextCode::Text(buf.clone()));
                    buf.clear();
                }

                let is_end_tag = if rich_text[index..].starts_with('/') {
                    index += 1;
                    true
                } else {
                    false
                };

                let end_index = if let Some(end_index) = rich_text[index..].find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
                    index + end_index
                } else {
                    rich_text.len()
                };

                if end_index == index {
                    return Err(ParseError::new(ParseErrorKind::SyntaxError, index, rich_text));
                }

                let tag_name = &rich_text[index..end_index];

                let Some(tag) = Tag::from_tag_name(tag_name) else {
                    return Err(ParseError::new(ParseErrorKind::UnknownTag, index, rich_text));
                };

                index = end_index;

                if is_end_tag {
                    let Some((old_tag, old_style)) = stack.pop() else {
                        return Err(ParseError::new(ParseErrorKind::UnexpectedCloseTag { actual: tag, expected: None }, index, rich_text));
                    };

                    if old_tag != tag {
                        return Err(ParseError::new(ParseErrorKind::UnexpectedCloseTag { actual: tag, expected: Some(old_tag) }, index, rich_text));
                    }

                    match tag {
                        Tag::Bold | Tag::Faint => {
                            if current_style.font_weight != old_style.font_weight {
                                code.push(RichTextCode::FontWeight(old_style.font_weight));
                                current_style.font_weight = old_style.font_weight;
                            }
                        }
                        Tag::Italic => {
                            if current_style.font_style != old_style.font_style {
                                code.push(RichTextCode::FontStyle(old_style.font_style));
                                current_style.font_style = old_style.font_style;
                            }
                        }
                        Tag::Underline | Tag::DoublyUnderline => {
                            if current_style.text_decoration != old_style.text_decoration {
                                code.push(RichTextCode::TextDecoration(old_style.text_decoration));
                                current_style.text_decoration = old_style.text_decoration;
                            }
                        }
                        Tag::Foreground => {
                            if current_style.foreground != old_style.foreground {
                                code.push(RichTextCode::Foreground(old_style.foreground));
                                current_style.foreground = old_style.foreground;
                            }
                        }
                        Tag::Background => {
                            if current_style.background != old_style.background {
                                code.push(RichTextCode::Background(old_style.background));
                                current_style.background = old_style.background;
                            }
                        }
                    }
                } else {
                    stack.push((tag, current_style));

                    match tag {
                        Tag::Bold => {
                            if current_style.font_weight != FontWeight::Bold {
                                code.push(RichTextCode::FontWeight(FontWeight::Bold));
                                current_style.font_weight = FontWeight::Bold;
                            }
                        }
                        Tag::Faint => {
                            if current_style.font_weight != FontWeight::Faint {
                                code.push(RichTextCode::FontWeight(FontWeight::Faint));
                                current_style.font_weight = FontWeight::Faint;
                            }
                        }
                        Tag::Italic => {
                            if current_style.font_style != FontStyle::Italic {
                                code.push(RichTextCode::FontStyle(FontStyle::Italic));
                                current_style.font_style = FontStyle::Italic;
                            }
                        }
                        Tag::Underline => {
                            if current_style.text_decoration != TextDecoration::Underline {
                                code.push(RichTextCode::TextDecoration(TextDecoration::Underline));
                                current_style.text_decoration = TextDecoration::Underline;
                            }
                        }
                        Tag::DoublyUnderline => {
                            if current_style.text_decoration != TextDecoration::DoublyUnderline {
                                code.push(RichTextCode::TextDecoration(TextDecoration::DoublyUnderline));
                                current_style.text_decoration = TextDecoration::DoublyUnderline;
                            }
                        }
                        Tag::Foreground => {
                            let (new_index, color) = parse_color_attr(rich_text, index)?;
                            index = new_index;
                            if current_style.foreground != color {
                                code.push(RichTextCode::Foreground(color));
                                current_style.foreground = color;
                            }
                        }
                        Tag::Background => {
                            let (new_index, color) = parse_color_attr(rich_text, index)?;
                            index = new_index;
                            if current_style.background != color {
                                code.push(RichTextCode::Background(color));
                                current_style.background = color;
                            }
                        }
                    }
                }

                if !rich_text[index..].starts_with(']') {
                    return Err(ParseError::new(ParseErrorKind::SyntaxError, index, rich_text));
                }

                index += 1;
            }
        }

        if let Some((tag, _)) = stack.pop() {
            return Err(ParseError::new(ParseErrorKind::ExpectedCloseTag(tag), rich_text.len(), rich_text));
        }

        if !buf.is_empty() {
            code.push(RichTextCode::Text(buf.clone()));
        }

        Ok(Self(code))
    }

    #[inline]
    pub fn inner(&self) -> &[RichTextCode] {
        &self.0
    }

    #[inline]
    pub fn into_inner(self) -> Vec<RichTextCode> {
        self.0
    }
}

fn parse_hex_nibble(rich_text: &str, index: usize) -> Result<(usize, u8), ParseError> {
    let Some(value) = rich_text[index..].chars().next() else {
        return Err(ParseError::new(ParseErrorKind::SyntaxError, index, rich_text));
    };

    if value >= '0' && value <= '9' {
        return Ok((index + 1, (value as u32 - '0' as u32) as u8));
    }

    if value >= 'a' && value <= 'f' {
        return Ok((index + 1, (value as u32 - 'a' as u32) as u8 + 0xA));
    }

    if value >= 'A' && value <= 'F' {
        return Ok((index + 1, (value as u32 - 'A' as u32) as u8 + 0xA));
    }

    Err(ParseError::new(ParseErrorKind::SyntaxError, index, rich_text))
}

fn parse_hex_byte(rich_text: &str, mut index: usize) -> Result<(usize, u8), ParseError> {
    let (new_index, upper) = parse_hex_nibble(rich_text, index)?; index = new_index;
    let (new_index, lower) = parse_hex_nibble(rich_text, index)?; index = new_index;

    Ok((index, (upper << 4) | lower))
}

fn parse_color_attr(rich_text: &str, mut index: usize) -> Result<(usize, Color), ParseError> {
    if !rich_text[index..].starts_with('=') {
        return Err(ParseError::new(ParseErrorKind::SyntaxError, index, rich_text));
    }

    index += 1;

    if rich_text[index..].starts_with('#') {
        index += 1;

        let (new_index, r) = parse_hex_byte(rich_text, index)?; index = new_index;
        let (new_index, g) = parse_hex_byte(rich_text, index)?; index = new_index;
        let (new_index, b) = parse_hex_byte(rich_text, index)?; index = new_index;

        Ok((index, Color::Rgb { r, g, b }))
    } else {
        let Some(end_index) = rich_text[index..].find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) else {
            return Err(ParseError::new(ParseErrorKind::UnexpectedEndOfInput, rich_text.len(), rich_text));
        };
        let end_index = index + end_index;

        if index == end_index {
            return Err(ParseError::new(ParseErrorKind::SyntaxError, index, rich_text));
        }

        let color_name = &rich_text[index..end_index];

        let color = if color_name.eq_ignore_ascii_case("default") {
            Color::Default
        } else if let Some(color) = Color16::parse(color_name) {
            Color::Color16(color)
        } else {
            return Err(ParseError::new(ParseErrorKind::SyntaxError, index, rich_text));
        };

        index = end_index;

        Ok((index, color))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RichTextStyle {
    pub font_weight: FontWeight,
    pub text_decoration: TextDecoration,
    pub font_style: FontStyle,
    pub foreground: Color,
    pub background: Color,
}

impl RichTextStyle {
    pub fn diff(&self, new_style: &RichTextStyle, code: &mut Vec<RichTextCode>) {
        if self.font_weight != new_style.font_weight {
            code.push(RichTextCode::FontWeight(new_style.font_weight));
        }

        if self.text_decoration != new_style.text_decoration {
            code.push(RichTextCode::TextDecoration(new_style.text_decoration));
        }

        if self.font_style != new_style.font_style {
            code.push(RichTextCode::FontStyle(new_style.font_style));
        }

        if self.foreground != new_style.foreground {
            code.push(RichTextCode::Foreground(new_style.foreground));
        }

        if self.background != new_style.background {
            code.push(RichTextCode::Background(new_style.background));
        }
    }
}

impl From<&TextStyle> for RichTextStyle {
    #[inline]
    fn from(value: &TextStyle) -> Self {
        Self {
            font_weight: value.font_weight.unwrap_or_default(),
            text_decoration: value.text_decoration.unwrap_or_default(),
            font_style: value.font_style.unwrap_or_default(),
            foreground: value.foreground.unwrap_or_default(),
            background: value.background.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RichTextCode {
    FontWeight(FontWeight),
    TextDecoration(TextDecoration),
    FontStyle(FontStyle),
    Foreground(Color),
    Background(Color),
    Text(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseError {
    kind: ParseErrorKind,
    index: usize,
    location: Location,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, index: usize, text: &str) -> Self {
        Self {
            kind,
            index,
            location: Location::from_index(text, index)
        }
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn kind(&self) -> ParseErrorKind {
        self.kind
    }

    #[inline]
    pub fn location(&self) -> &Location {
        &self.location
    }

    pub fn print_line(&self, text: &str, f: &mut impl std::io::Write) -> std::io::Result<()> {
        let line = &text[self.location.line_start..self.location.line_end];

        let lineno_width = {
            let mut lineno_width = 0;
            let mut lineno = self.location.lineno;

            while lineno > 0 {
                lineno /= 10;
                lineno_width += 1;
            }

            lineno_width
        };

        write!(f, "{}: {}\n{:<lineno_width$}  {:->column$}\n", self.location.lineno, line, "", "^", column = self.location.column)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: ", self.location.lineno, self.location.column)?;

        match self.kind {
            ParseErrorKind::UnexpectedCloseTag { actual, expected } =>
                if let Some(expected) = expected {
                    write!(f, "Unexpected close tag.\n    expected: [/{}]\n      actual: [/{}]",
                        expected.tag_name(), actual.tag_name())
                } else {
                    write!(f, "Unexpected close tag: [/{}]", actual.tag_name())
                },
            ParseErrorKind::ExpectedCloseTag(tag) =>
                write!(f, "Unexpected end of input.\n    expected: [/{}]", tag.tag_name()),
            ParseErrorKind::UnexpectedEndOfInput =>
                write!(f, "Unexpected end of input"),
            ParseErrorKind::UnknownTag => write!(f, "Unknown Tag"),
            ParseErrorKind::SyntaxError => write!(f, "Syntax Error"),
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    UnexpectedCloseTag { actual: Tag, expected: Option<Tag> },
    ExpectedCloseTag(Tag),
    UnexpectedEndOfInput,
    UnknownTag,
    SyntaxError,
}

impl ParseErrorKind {
    #[inline]
    pub fn is_unexpected_close_tag(&self) -> bool {
        matches!(self, ParseErrorKind::UnexpectedCloseTag { actual: _, expected: _ })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    Bold,
    Faint,
    Italic,
    Underline,
    DoublyUnderline,
    Foreground,
    Background,
}

impl Tag {
    #[inline]
    pub fn from_tag_name(tag_name: &str) -> Option<Self> {
        match tag_name {
            "b" => Some(Tag::Bold),
            "i" => Some(Tag::Italic),
            "f" => Some(Tag::Faint),
            "u" => Some(Tag::Underline),
            "du" => Some(Tag::DoublyUnderline),
            "color" | "fg" => Some(Tag::Foreground),
            "bg" => Some(Tag::Background),
            _ => None
        }
    }

    #[inline]
    pub fn tag_name(&self) -> &'static str {
        match self {
            Tag::Bold => "b",
            Tag::Faint => "f",
            Tag::Italic => "i",
            Tag::Underline => "u",
            Tag::DoublyUnderline => "du",
            Tag::Foreground => "color",
            Tag::Background => "bg",
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IllegalTagName;

impl TryFrom<&str> for Tag {
    type Error = IllegalTagName;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Tag::from_tag_name(value).ok_or(IllegalTagName)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub index: usize,
    pub lineno: usize,
    pub column: usize,
    pub line_start: usize,
    pub line_end: usize,
    pub column_index: usize,
}

impl Location {
    #[inline]
    pub fn from_index(text: &str, index: usize) -> Self {
        let (lineno, line_start, line_end, suffix) = if let Some(line_start) = text[..index].rfind('\n') {
            let suffix = &text[line_start + 1..];
            (
                text[..line_start].chars().filter(|c| *c == '\n').count() + 1,
                line_start,
                line_start + 1 + suffix.find('\n').unwrap_or(suffix.len()),
                suffix,
            )
        } else {
            (1, 0, text.find('\n').unwrap_or(text.len()), text)
        };

        Self {
            index,
            lineno,
            column: suffix[..index - line_start].chars().count() + 1,
            line_start,
            line_end,
            column_index: index - line_start,
        }
    }
}
