use crate::{char_width::CharWidth, color::{Color, Color16}, style::{FontStyle, FontWeight, TextDecoration, TextStyle}};

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
pub struct RichText {
    rich_text: Vec<Vec<RichTextCode>>,
    lines: usize,
    width: usize,
}

impl Default for RichText {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl RichText {
    #[inline]
    pub fn new() -> Self {
        Self {
            rich_text: Vec::new(),
            lines: 1,
            width: 0,
        }
    }

    pub fn from_plain_text(toplevel_style: &RichTextStyle, control_style: &RichTextStyle, plain_text: &str) -> Self {
        let mut rich_text = Self::new();
        rich_text.append_plain_text(toplevel_style, control_style, plain_text);
        rich_text
    }

    pub fn parse(toplevel_style: &RichTextStyle, control_style: &RichTextStyle, rich_text: &str) -> Result<Self, ParseError> {
        let mut new_rich_text = RichText::new();

        new_rich_text.append_rich_text(toplevel_style, control_style, rich_text)?;

        Ok(new_rich_text)
    }

    pub fn append(&mut self, other: &RichText) {
        self.rich_text.extend_from_slice(&other.rich_text);
        if other.width > self.width {
            self.width = other.width;
        }
        self.lines += other.lines;
    }

    pub fn append_plain_text(&mut self, toplevel_style: &RichTextStyle, control_style: &RichTextStyle, plain_text: &str) {
        let mut line = Vec::new();
        let mut line_width = 0;
        let mut prev_index = 0;

        for (index, ch) in plain_text.char_indices() {
            if ch.is_ascii_control() {
                if prev_index < index {
                    let text = &plain_text[prev_index..index];
                    let text_width = text.char_width_ignore_unprintable();
                    line_width += text_width;
                    line.push(RichTextCode::Text { text: text.to_string(), width: text_width });
                }

                if ch == '\n' {
                    if ch == '\n' {
                        if line_width > self.width {
                            self.width = line_width;
                        }
                        line_width = 0;
                        self.lines += 1;
                        std::mem::swap(self.rich_text.push_mut(Vec::new()), &mut line);
                    } else {
                        line_width += 1;
                        toplevel_style.diff(control_style, &mut line);
                        line.push(RichTextCode::Text {
                            text: if ch == '\x7F' {
                                "\u{2421}".to_string()
                            } else {
                                unsafe { char::from_u32_unchecked(0x2400 + ch as u32) }.to_string()
                            },
                            width: 1,
                        });
                        control_style.diff(toplevel_style, &mut line);
                    }
                }

                prev_index = index;
            }
        }

        if prev_index < plain_text.len() {
            let text = &plain_text[prev_index..];
            let text_width = text.char_width_ignore_unprintable();
            line_width += text_width;
            if line_width > self.width {
                self.width = line_width;
            }
            line.push(RichTextCode::Text {
                text: text.to_string(),
                width: text_width,
            });
        }

        if !line.is_empty() {
            self.rich_text.push(line);
        }
    }

    pub fn append_rich_text(&mut self, toplevel_style: &RichTextStyle, control_style: &RichTextStyle, rich_text: &str) -> Result<(), ParseError> {
        let old_lines = self.lines;
        let old_width = self.width;
        let old_len = self.rich_text.len();

        let mut current_style = *toplevel_style;
        let mut stack: Vec<(Tag, RichTextStyle)> = Vec::new();

        let mut index = 0;
        let mut buf = String::new();

        let mut line = Vec::new();
        let mut line_width = 0;

        'outer: while index < rich_text.len() {
            let old_index = index;

            for (char_index, ch) in rich_text[index..].char_indices() {
                if ch == '[' || ch == ']' {
                    index += char_index;

                    buf.push_str(&rich_text[old_index..index]);

                    let new_index = index + 1;

                    if ch == ']' {
                        if !rich_text[new_index..].starts_with(']') {
                            self.width = old_width;
                            self.lines = old_lines;
                            self.rich_text.truncate(old_len);

                            return Err(ParseError::new(ParseErrorKind::SyntaxError, index, rich_text));
                        }

                        buf.push(']');

                        index = new_index + 1;
                    } else if rich_text[new_index..].starts_with('[') {
                        buf.push('[');

                        index = new_index + 1;
                    } else {
                        index = new_index;

                        if !buf.is_empty() {
                            let text_width = buf.char_width_ignore_unprintable();
                            line_width += text_width;
                            line.push(RichTextCode::Text {
                                text: buf.clone(),
                                width: text_width,
                            });
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
                            self.width = old_width;
                            self.lines = old_lines;
                            self.rich_text.truncate(old_len);

                            return Err(ParseError::new(ParseErrorKind::SyntaxError, index, rich_text));
                        }

                        let tag_name = &rich_text[index..end_index];

                        let Some(tag) = Tag::from_tag_name(tag_name) else {
                            self.width = old_width;
                            self.lines = old_lines;
                            self.rich_text.truncate(old_len);

                            return Err(ParseError::new(ParseErrorKind::UnknownTag, index, rich_text));
                        };

                        index = end_index;

                        if is_end_tag {
                            let Some((old_tag, old_style)) = stack.pop() else {
                                self.width = old_width;
                                self.lines = old_lines;
                                self.rich_text.truncate(old_len);

                                return Err(ParseError::new(ParseErrorKind::UnexpectedCloseTag { actual: tag, expected: None }, index, rich_text));
                            };

                            if old_tag != tag {
                                self.width = old_width;
                                self.lines = old_lines;
                                self.rich_text.truncate(old_len);

                                return Err(ParseError::new(ParseErrorKind::UnexpectedCloseTag { actual: tag, expected: Some(old_tag) }, index, rich_text));
                            }

                            match tag {
                                Tag::Bold | Tag::Faint => {
                                    if current_style.font_weight != old_style.font_weight {
                                        line.push(RichTextCode::FontWeight(old_style.font_weight));
                                        current_style.font_weight = old_style.font_weight;
                                    }
                                }
                                Tag::Italic => {
                                    if current_style.font_style != old_style.font_style {
                                        line.push(RichTextCode::FontStyle(old_style.font_style));
                                        current_style.font_style = old_style.font_style;
                                    }
                                }
                                Tag::Underline | Tag::DoublyUnderline => {
                                    if current_style.text_decoration != old_style.text_decoration {
                                        line.push(RichTextCode::TextDecoration(old_style.text_decoration));
                                        current_style.text_decoration = old_style.text_decoration;
                                    }
                                }
                                Tag::Foreground => {
                                    if current_style.foreground != old_style.foreground {
                                        line.push(RichTextCode::Foreground(old_style.foreground));
                                        current_style.foreground = old_style.foreground;
                                    }
                                }
                                Tag::Background => {
                                    if current_style.background != old_style.background {
                                        line.push(RichTextCode::Background(old_style.background));
                                        current_style.background = old_style.background;
                                    }
                                }
                            }
                        } else {
                            stack.push((tag, current_style));

                            match tag {
                                Tag::Bold => {
                                    if current_style.font_weight != FontWeight::Bold {
                                        line.push(RichTextCode::FontWeight(FontWeight::Bold));
                                        current_style.font_weight = FontWeight::Bold;
                                    }
                                }
                                Tag::Faint => {
                                    if current_style.font_weight != FontWeight::Faint {
                                        line.push(RichTextCode::FontWeight(FontWeight::Faint));
                                        current_style.font_weight = FontWeight::Faint;
                                    }
                                }
                                Tag::Italic => {
                                    if current_style.font_style != FontStyle::Italic {
                                        line.push(RichTextCode::FontStyle(FontStyle::Italic));
                                        current_style.font_style = FontStyle::Italic;
                                    }
                                }
                                Tag::Underline => {
                                    if current_style.text_decoration != TextDecoration::Underline {
                                        line.push(RichTextCode::TextDecoration(TextDecoration::Underline));
                                        current_style.text_decoration = TextDecoration::Underline;
                                    }
                                }
                                Tag::DoublyUnderline => {
                                    if current_style.text_decoration != TextDecoration::DoublyUnderline {
                                        line.push(RichTextCode::TextDecoration(TextDecoration::DoublyUnderline));
                                        current_style.text_decoration = TextDecoration::DoublyUnderline;
                                    }
                                }
                                Tag::Foreground => {
                                    let (new_index, color) = parse_color_attr(rich_text, index)?;
                                    index = new_index;
                                    if current_style.foreground != color {
                                        line.push(RichTextCode::Foreground(color));
                                        current_style.foreground = color;
                                    }
                                }
                                Tag::Background => {
                                    let (new_index, color) = parse_color_attr(rich_text, index)?;
                                    index = new_index;
                                    if current_style.background != color {
                                        line.push(RichTextCode::Background(color));
                                        current_style.background = color;
                                    }
                                }
                            }
                        }

                        if !rich_text[index..].starts_with(']') {
                            self.width = old_width;
                            self.lines = old_lines;
                            self.rich_text.truncate(old_len);

                            return Err(ParseError::new(ParseErrorKind::SyntaxError, index, rich_text));
                        }

                        index += 1;
                    }

                    continue 'outer;
                } else if ch.is_ascii_control() {
                    if !buf.is_empty() {
                        let text_width = buf.char_width_ignore_unprintable();
                        line_width += text_width;
                        line.push(RichTextCode::Text {
                            text: buf.clone(),
                            width: text_width,
                        });
                        buf.clear();
                    }

                    if ch == '\n' {
                        if line_width > self.width {
                            self.width = line_width;
                        }
                        line_width = 0;
                        self.lines += 1;
                        std::mem::swap(self.rich_text.push_mut(Vec::new()), &mut line);
                    } else {
                        line_width += 1;
                        current_style.diff(control_style, &mut line);
                        line.push(RichTextCode::Text {
                            text: if ch == '\x7F' {
                                "\u{2421}".to_string()
                            } else {
                                unsafe { char::from_u32_unchecked(0x2400 + ch as u32) }.to_string()
                            },
                            width: 1,
                        });
                        control_style.diff(&current_style, &mut line);
                    }

                    index += char_index + 1;
                    continue 'outer;
                }
            }

            break;
        }

        if let Some((tag, _)) = stack.pop() {
            self.width = old_width;
            self.lines = old_lines;
            self.rich_text.truncate(old_len);

            return Err(ParseError::new(ParseErrorKind::ExpectedCloseTag(tag), rich_text.len(), rich_text));
        }

        if !buf.is_empty() {
            let text_width = buf.char_width_ignore_unprintable();
            line_width += text_width;
            if line_width > self.width {
                self.width = line_width;
            }
            line.push(RichTextCode::Text {
                text: buf.clone(),
                width: text_width,
            });
        }

        if !line.is_empty() {
            self.rich_text.push(line);
        }

        Ok(())
    }

    #[inline]
    pub fn lines(&self) -> usize {
        self.lines
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn rich_text(&self) -> &[Vec<RichTextCode>] {
        &self.rich_text
    }

    #[inline]
    pub fn into_inner(self) -> Vec<Vec<RichTextCode>> {
        self.rich_text
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

    #[inline]
    pub fn build() -> RichTextStyleBuilder {
        RichTextStyleBuilder::default()
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

impl From<&RichTextStyle> for TextStyle {
    #[inline]
    fn from(value: &RichTextStyle) -> Self {
        Self {
            font_weight: Some(value.font_weight),
            text_decoration: Some(value.text_decoration),
            font_style: Some(value.font_style),
            foreground: Some(value.foreground),
            background: Some(value.background),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RichTextStyleBuilder {
    inner: RichTextStyle
}

impl RichTextStyleBuilder {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn inner(&self) -> &RichTextStyle {
        &self.inner
    }

    #[inline]
    pub fn into_inner(self) -> RichTextStyle {
        self.inner
    }

    #[inline]
    pub fn font_weight(mut self, font_weight: FontWeight) -> Self {
        self.inner.font_weight = font_weight;
        self
    }

    #[inline]
    pub fn text_decoration(mut self, text_decoration: TextDecoration) -> Self {
        self.inner.text_decoration = text_decoration;
        self
    }

    #[inline]
    pub fn font_style(mut self, font_style: FontStyle) -> Self {
        self.inner.font_style = font_style;
        self
    }

    #[inline]
    pub fn foreground(mut self, foreground: Color) -> Self {
        self.inner.foreground = foreground;
        self
    }

    #[inline]
    pub fn background(mut self, background: Color) -> Self {
        self.inner.background = background;
        self
    }
}

impl From<RichTextStyleBuilder> for RichTextStyle {
    #[inline]
    fn from(value: RichTextStyleBuilder) -> Self {
        value.into_inner()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RichTextCode {
    FontWeight(FontWeight),
    TextDecoration(TextDecoration),
    FontStyle(FontStyle),
    Foreground(Color),
    Background(Color),
    Text { text: String, width: usize },
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
