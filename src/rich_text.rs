use crate::{ansi_codes::{write_bg, write_fg}, char_width::{CharWidth, crop}, color::{Color, Color16}, style::{FontStyle, FontWeight, Style, TextDecoration}, termio::TermIO, wrap::LineWrapper};

use bitflags::bitflags;

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
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct RichText {
    pub(super) lines: Vec<Vec<RichTextCode>>,
    pub(super) width: usize,
}

pub fn line_width(line: &[RichTextCode]) -> usize {
    let mut line_width = 0;

    for code in line {
        if let RichTextCode::Text { width, .. } = code {
            line_width += width;
        }
    }

    line_width
}

impl RichText {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_plain_text(plain_text: &str) -> Self {
        let mut rich_text = Self::new();
        rich_text.append_plain_text(plain_text);
        rich_text
    }

    pub fn parse(rich_text: &str) -> Result<Self, ParseError> {
        let mut new_rich_text = RichText::new();

        new_rich_text.append_rich_text(rich_text)?;

        Ok(new_rich_text)
    }

    pub fn append_line(&mut self) {
        self.lines.push(Vec::new());
    }

    pub fn append_lines(&mut self, count: usize) {
        self.lines.reserve(count);
        for _ in 0..count {
            self.lines.push(Vec::new());
        }
    }

    pub fn append(&mut self, other: &RichText) {
        self.lines.extend_from_slice(&other.lines);
        if other.width > self.width {
            self.width = other.width;
        }
    }

    pub fn append_plain_text(&mut self, plain_text: &str) {
        self.append_text(&DEFAULT_STYLE, plain_text);
    }

    fn append_style(&mut self, style: &RichTextStyle) {
        if self.lines.is_empty() {
            let line = self.lines.push_mut(Vec::new());
            DEFAULT_STYLE.diff(style, line);
            return;
        }

        if style.is_default() {
            return;
        }

        bitflags! {
            #[derive(Debug, PartialEq, Eq)]
            struct FinishedFeatures: u32 {
                const None           =  0;
                const FontStyle      =  1;
                const FontWeight     =  2;
                const TextDecoration =  4;
                const Foreground     =  8;
                const Background     = 16;
                const All = (
                    Self::FontStyle.bits() |
                    Self::FontWeight.bits() |
                    Self::TextDecoration.bits() |
                    Self::Foreground.bits() |
                    Self::Background.bits()
                );
            }
        }

        let mut feat = FinishedFeatures::None;

        if style.font_style == FontStyle::Normal {
            feat |= FinishedFeatures::FontStyle;
        }

        if style.font_weight == FontWeight::Normal {
            feat |= FinishedFeatures::FontWeight;
        }

        if style.text_decoration == TextDecoration::None {
            feat |= FinishedFeatures::TextDecoration;
        }

        if style.foreground == Color::Default {
            feat |= FinishedFeatures::Foreground;
        }

        if style.background == Color::Default {
            feat |= FinishedFeatures::Background;
        }

        'outer: for line in self.lines.iter_mut().rev() {
            for code in line.iter_mut().rev() {
                match code {
                    RichTextCode::FontStyle(font_style) => {
                        if !feat.contains(FinishedFeatures::FontStyle) {
                            *font_style = style.font_style;
                            feat |= FinishedFeatures::FontStyle;
                        }
                    }
                    RichTextCode::FontWeight(font_weight) => {
                        if !feat.contains(FinishedFeatures::FontWeight) {
                            *font_weight = style.font_weight;
                            feat |= FinishedFeatures::FontWeight;
                        }
                    }
                    RichTextCode::TextDecoration(text_decoration) => {
                        if !feat.contains(FinishedFeatures::TextDecoration) {
                            *text_decoration = style.text_decoration;
                            feat |= FinishedFeatures::TextDecoration;
                        }
                    }
                    RichTextCode::Foreground(color) => {
                        if !feat.contains(FinishedFeatures::Foreground) {
                            *color = style.foreground;
                            feat |= FinishedFeatures::Foreground;
                        }
                    }
                    RichTextCode::Background(color) => {
                        if !feat.contains(FinishedFeatures::Background) {
                            *color = style.background;
                            feat |= FinishedFeatures::Background;
                        }
                    }
                    RichTextCode::Text { .. } => {
                        break 'outer;
                    }
                }

                if feat == FinishedFeatures::All {
                    break 'outer;
                }
            }
        }

        if feat != FinishedFeatures::All {
            let line = self.lines.last_mut().unwrap();

            if !feat.contains(FinishedFeatures::FontStyle) {
                line.push(RichTextCode::FontStyle(style.font_style));
            }

            if !feat.contains(FinishedFeatures::FontWeight) {
                line.push(RichTextCode::FontWeight(style.font_weight));
            }

            if !feat.contains(FinishedFeatures::TextDecoration) {
                line.push(RichTextCode::TextDecoration(style.text_decoration));
            }

            if !feat.contains(FinishedFeatures::Foreground) {
                line.push(RichTextCode::Foreground(style.foreground));
            }

            if !feat.contains(FinishedFeatures::Background) {
                line.push(RichTextCode::Background(style.background));
            }
        }
    }

    pub fn append_text(&mut self, style: &RichTextStyle, plain_text: &str) {
        self.append_style(style);

        let mut line = if let Some(line) = self.lines.last_mut() {
            line
        } else {
            self.lines.push_mut(Vec::new())
        };
        let mut line_width = line_width(line);
        let mut prev_index = 0;

        for (index, ch) in plain_text.char_indices() {
            if ch.is_ascii_control() {
                if prev_index < index {
                    let text = &plain_text[prev_index..index];
                    let text_width = text.char_width_ignore_unprintable();
                    line_width += text_width;
                    line.push(RichTextCode::Text { text: text.to_string(), width: text_width });
                }

                if ch.is_ascii_control() {
                    if ch == '\n' {
                        if line_width > self.width {
                            self.width = line_width;
                        }
                        line_width = 0;
                        line = self.lines.push_mut(Vec::new());
                    } else {
                        line_width += 1;
                        style.diff(&CONTROL_STYLE, &mut line);
                        line.push(RichTextCode::Text {
                            text: if ch == '\x7F' {
                                "\u{2421}".to_string()
                            } else {
                                unsafe { char::from_u32_unchecked(0x2400 + ch as u32) }.to_string()
                            },
                            width: 1,
                        });
                        CONTROL_STYLE.diff(style, &mut line);
                    }
                }

                prev_index = index + 1;
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

        style.diff(&DEFAULT_STYLE, &mut line);
    }

    pub fn append_rich_text(&mut self, rich_text: &str) -> Result<(), ParseError> {
        let old_width = self.width;
        let old_len = self.lines.len();

        let mut current_style = RichTextStyle::default();
        let mut stack: Vec<(Tag, RichTextStyle)> = Vec::new();

        let mut index = 0;
        let mut buf = String::new();

        let mut line = if let Some(line) = self.lines.last_mut() {
            line
        } else {
            self.lines.push_mut(Vec::new())
        };
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
                            self.lines.truncate(old_len);

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
                            self.lines.truncate(old_len);

                            return Err(ParseError::new(ParseErrorKind::SyntaxError, index, rich_text));
                        }

                        let tag_name = &rich_text[index..end_index];

                        let Some(tag) = Tag::from_tag_name(tag_name) else {
                            self.width = old_width;
                            self.lines.truncate(old_len);

                            return Err(ParseError::new(ParseErrorKind::UnknownTag, index, rich_text));
                        };

                        index = end_index;

                        if is_end_tag {
                            let Some((old_tag, old_style)) = stack.pop() else {
                                self.width = old_width;
                                self.lines.truncate(old_len);

                                return Err(ParseError::new(ParseErrorKind::UnexpectedCloseTag { actual: tag, expected: None }, index, rich_text));
                            };

                            if old_tag != tag {
                                self.width = old_width;
                                self.lines.truncate(old_len);

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
                                        append_font_weight(line, FontWeight::Bold);
                                        current_style.font_weight = FontWeight::Bold;
                                    }
                                }
                                Tag::Faint => {
                                    if current_style.font_weight != FontWeight::Faint {
                                        append_font_weight(line, FontWeight::Faint);
                                        current_style.font_weight = FontWeight::Faint;
                                    }
                                }
                                Tag::Italic => {
                                    if current_style.font_style != FontStyle::Italic {
                                        append_font_style(line, FontStyle::Italic);
                                        current_style.font_style = FontStyle::Italic;
                                    }
                                }
                                Tag::Underline => {
                                    if current_style.text_decoration != TextDecoration::Underline {
                                        append_text_decoration(line, TextDecoration::Underline);
                                        current_style.text_decoration = TextDecoration::Underline;
                                    }
                                }
                                Tag::DoublyUnderline => {
                                    if current_style.text_decoration != TextDecoration::DoublyUnderline {
                                        append_text_decoration(line, TextDecoration::DoublyUnderline);
                                        current_style.text_decoration = TextDecoration::DoublyUnderline;
                                    }
                                }
                                Tag::Foreground => {
                                    let (new_index, color) = parse_color_attr(rich_text, index)?;
                                    index = new_index;
                                    if current_style.foreground != color {
                                        append_foreground(line, color);
                                        current_style.foreground = color;
                                    }
                                }
                                Tag::Background => {
                                    let (new_index, color) = parse_color_attr(rich_text, index)?;
                                    index = new_index;
                                    if current_style.background != color {
                                        append_background(line, color);
                                        current_style.background = color;
                                    }
                                }
                            }
                        }

                        if !rich_text[index..].starts_with(']') {
                            self.width = old_width;
                            self.lines.truncate(old_len);

                            return Err(ParseError::new(ParseErrorKind::SyntaxError, index, rich_text));
                        }

                        index += 1;
                    }

                    continue 'outer;
                } else if ch.is_ascii_control() {
                    index += char_index;
                    buf.push_str(&rich_text[old_index..index]);

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
                        line = self.lines.push_mut(Vec::new());
                    } else {
                        line_width += 1;
                        current_style.diff(&CONTROL_STYLE, &mut line);
                        line.push(RichTextCode::Text {
                            text: if ch == '\x7F' {
                                "\u{2421}".to_string()
                            } else {
                                unsafe { char::from_u32_unchecked(0x2400 + ch as u32) }.to_string()
                            },
                            width: 1,
                        });
                        CONTROL_STYLE.diff(&current_style, &mut line);
                    }

                    index += 1;
                    continue 'outer;
                }
            }

            if index < rich_text.len() {
                buf.push_str(&rich_text[index..]);
            }

            break;
        }

        if let Some((tag, _)) = stack.pop() {
            self.width = old_width;
            self.lines.truncate(old_len);

            return Err(ParseError::new(ParseErrorKind::ExpectedCloseTag(tag), rich_text.len(), rich_text));
        }

        if !buf.is_empty() {
            let text_width = buf.char_width_ignore_unprintable();
            line_width += text_width;
            line.push(RichTextCode::Text {
                text: buf,
                width: text_width,
            });
        }

        if line_width > self.width {
            self.width = line_width;
        }

        Ok(())
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.lines.len()
    }

    #[inline]
    pub fn lines(&self) -> &[Vec<RichTextCode>] {
        &self.lines
    }

    #[inline]
    pub fn into_inner(self) -> Vec<Vec<RichTextCode>> {
        self.lines
    }

    pub fn right_pad(&mut self, width: usize) {
        if width > self.width {
            for line in &mut self.lines {
                let line_width = line_width(line);

                if line_width < width {
                    let diff = width - line_width;
                    if let Some(RichTextCode::Text { text, width: text_width }) = line.last_mut() {
                        text.reserve(diff);
                        for _ in 0..diff {
                            text.push(' ');
                        }
                        *text_width += diff;
                    } else {
                        let text = " ".repeat(diff);
                        line.push(RichTextCode::Text { text, width: diff });
                    }
                }
            }
            self.width = width;
        }
    }

    pub fn left_pad(&mut self, width: usize) {
        if width > self.width {
            for line in &mut self.lines {
                let line_width = line_width(line);

                if line_width < width {
                    let diff = width - line_width;
                    if let Some(RichTextCode::Text { text, width: text_width }) = line.first_mut() {
                        let mut new_text = String::with_capacity(text.len() + diff);
                        for _ in 0..diff {
                            new_text.push(' ');
                        }
                        new_text.push_str(&text);
                        *text = new_text;
                        *text_width += diff;
                    } else {
                        let text = " ".repeat(diff);
                        line.insert(0, RichTextCode::Text { text, width: diff });
                    }
                }
            }
            self.width = width;
        }
    }

    pub fn top_pad(&mut self, height: usize) {
        if height > self.height() {
            let mut new_lines = Vec::with_capacity(height);
            new_lines.resize_with(height - self.height(), Vec::new);
            new_lines.extend(self.lines.drain(..));
            self.lines = new_lines;
        }
    }

    pub fn bottom_pad(&mut self, height: usize) {
        if height > self.height() {
            self.lines.resize_with(height, Vec::new);
        }
    }

    pub fn vertical_append(&mut self, other: &RichText) {
        self.right_pad(self.width);

        let mut self_style = RichTextStyle::default();
        let mut other_style = RichTextStyle::default();

        for (self_line, other_line) in self.lines.iter_mut().zip(other.lines.iter()) {
            self_style.apply_changes(self_line);

            self_style.diff(&other_style, self_line);
            self_line.extend_from_slice(&other_line);

            other_style.apply_changes(other_line);
            other_style.diff(&self_style, self_line);
        }

        if self.lines.len() < other.lines.len() {
            self.lines.reserve(other.lines.len() - self.lines.len());
            for other_line in &other.lines[self.lines.len()..] {
                let mut self_line = Vec::with_capacity(other_line.len() + (self.width > 0) as usize);
                if self.width > 0 {
                    let text = " ".repeat(self.width);
                    self_line.push(RichTextCode::Text { text, width: self.width });
                }
                self_line.extend_from_slice(&other_line);
                self.lines.push(self_line);
            }
        }

        self.width += other.width;
    }

    #[inline]
    pub fn clear(&mut self) {
        self.lines.clear();
        self.width = 0;
    }

    pub fn draw(&self, termio: &mut TermIO, row: i32, column: i32) -> std::io::Result<()> {
        self.draw_cropped(
            termio, row, column, 0, 0,
            self.width.min(u32::MAX as usize) as u32,
            self.height().min(u32::MAX as usize) as u32,
        )
    }

    pub fn draw_cropped(&self, termio: &mut TermIO, row: i32, column: i32, crop_row: u32, crop_column: u32, crop_width: u32, crop_height: u32) -> std::io::Result<()> {
        let mut crop_row = crop_row as usize;
        let mut crop_column = crop_column as usize;

        let mut crop_row_end = crop_row + crop_height as usize;
        let mut crop_column_end = crop_column + crop_width as usize;

        let mut full_column_end = crop_column_end;

        if crop_row_end > self.height() {
            crop_row_end = self.height();
        }

        if crop_column_end > self.width {
            crop_column_end = self.width;
        }

        if crop_row >= crop_row_end {
            return Ok(());
        }

        if crop_column >= crop_column_end {
            return Ok(());
        }

        let window_size = *termio.window_size();

        if full_column_end > window_size.columns as usize {
            full_column_end = window_size.columns as usize;
        }

        let term_row;
        let term_column;

        if row < 0 {
            if crop_row_end < -row as usize {
                return Ok(());
            }

            let max_rows = -row as usize + window_size.rows as usize;
            if crop_row_end > max_rows {
                crop_row_end = max_rows;
            }

            crop_row += -row as usize;
            term_row = 0;
        } else {
            if row as u32 > window_size.rows {
                return Ok(());
            }

            if row as usize + (crop_row_end - crop_row) > window_size.rows as usize {
                crop_row_end = window_size.rows as usize + crop_row - row as usize;
            }
            term_row = row as u32;
        }

        if column < 0 {
            if crop_column_end < -column as usize {
                return Ok(());
            }

            let max_columns = -column as usize + window_size.columns as usize;
            if crop_column_end > max_columns {
                crop_column_end = max_columns;
            }

            crop_column += -column as usize;
            term_column = 0;
        } else {
            if column as u32 > window_size.columns {
                return Ok(());
            }

            if column as usize + (crop_column_end - crop_column) > window_size.columns as usize {
                crop_column_end = window_size.columns as usize + crop_column - column as usize;
            }
            term_column = column as u32;
        }

        let lines = &self.lines[crop_row..crop_row_end];

        termio.clear_style()?;

        termio.fg_default()?;
        termio.bg_default()?;

        let mut first = true;
        let mut prev_line_index = 0;

        for (line_index, line) in lines.iter().enumerate() {
            let mut moved = false;
            let mut line_width = 0;

            for code in line {
                match code {
                    RichTextCode::FontWeight(font_weight) => termio.font_weight(*font_weight)?,
                    RichTextCode::FontStyle(font_style) => termio.font_style(*font_style)?,
                    RichTextCode::TextDecoration(text_decoration) => termio.text_decoration(*text_decoration)?,
                    RichTextCode::Foreground(color) => termio.fg(*color)?,
                    RichTextCode::Background(color) => termio.bg(*color)?,
                    RichTextCode::Text { text, width: text_width } => {
                        let text_width = *text_width;
                        if line_width >= crop_column && line_width + text_width <= crop_column_end {
                            if !moved {
                                if !first && term_column == 0 && prev_line_index + 1 == line_index {
                                    termio.write_str("\n")?;
                                } else {
                                    first = false;
                                    termio.move_cursor(term_row + line_index as u32, term_column)?;
                                }
                                moved = true;
                                prev_line_index = line_index;
                            }

                            termio.write_str(text)?;
                        } else if line_width + text_width >= crop_column && line_width < crop_column_end {
                            if !moved {
                                if !first && term_column == 0 && prev_line_index + 1 == line_index {
                                    termio.write_str("\n")?;
                                } else {
                                    first = false;
                                    termio.move_cursor(term_row + line_index as u32, term_column)?;
                                }
                                moved = true;
                                prev_line_index = line_index;
                            }

                            let text_column = if line_width >= crop_column { 0 } else { crop_column - line_width };
                            let text_column_end = crop_column_end - line_width;
                            if text_column < text_column_end {
                                let text = crop(
                                    text,
                                    text_column,
                                    text_column_end,
                                );
                                termio.write_str(text)?;
                            }
                        }

                        line_width += text_width;
                    }
                }
            }

            if line_width < full_column_end {
                if !moved {
                    if !first && term_column == 0 && prev_line_index + 1 == line_index {
                        termio.write_str("\n")?;
                    } else {
                        first = false;
                        termio.move_cursor(term_row + line_index as u32, term_column)?;
                    }
                    prev_line_index = line_index;
                }

                termio.write(b" ")?;
                termio.repeat((full_column_end - line_width) as u32 - 1)?;
            }
        }

        Ok(())
    }

    pub fn print(&self, write: &mut impl std::io::Write) -> std::io::Result<()> {
        let mut style = RichTextStyle::default();
        for line in &self.lines {
            DEFAULT_STYLE.write_diff(&style, write)?;

            for item in line {
                match item {
                    RichTextCode::FontWeight(font_weight) => {
                        font_weight.write(write)?;
                        style.font_weight = *font_weight;
                    }
                    RichTextCode::FontStyle(font_style) => {
                        font_style.write(write)?;
                        style.font_style = *font_style;
                    }
                    RichTextCode::TextDecoration(text_decoration) => {
                        text_decoration.write(write)?;
                        style.text_decoration = *text_decoration;
                    }
                    &RichTextCode::Foreground(color) => {
                        write_fg(write, color)?;
                        style.foreground = color;
                    }
                    &RichTextCode::Background(color) => {
                        write_bg(write, color)?;
                        style.background = color;
                    }
                    RichTextCode::Text { text, .. } => {
                        write.write_all(text.as_bytes())?;
                    }
                }
            }

            style.write_diff(&DEFAULT_STYLE, write)?;
            write.write_all(b"\n")?;
        }

        Ok(())
    }

    pub fn wrap(&self, width: usize) -> RichText {
        let mut new_lines = Vec::with_capacity(self.lines.len());
        let mut max_width = 0;

        for line in &self.lines {
            let mut line_width = 0;
            let mut new_line = new_lines.push_mut(Vec::with_capacity(line.len()));

            for code in line {
                if let RichTextCode::Text { text, width: text_width } = code {
                    let new_width = line_width + text_width;
                    if new_width < width {
                        new_line.push(code.clone());
                        line_width = new_width;
                    } else if new_width == width {
                        new_line.push(code.clone());
                        if line_width > max_width {
                            max_width = line_width;
                        }
                        line_width = 0;
                        new_line = new_lines.push_mut(Vec::new());
                    } else {
                        for text_line in LineWrapper::with_prefix_width(text, width, line_width) {
                            let text_width = text_line.char_width_ignore_unprintable();
                            line_width += text_width;
                            new_line.push(RichTextCode::Text {
                                text: text_line.to_owned(),
                                width: text_width,
                            });
                            new_line = new_lines.push_mut(Vec::new());
                            if line_width > max_width {
                                max_width = line_width;
                            }
                            line_width = 0;
                        }
                    }
                } else {
                    new_line.push(code.clone());
                }
            }
        }

        RichText { lines: new_lines, width: max_width }
    }
}

pub fn right_pad_line(line: &mut Vec<RichTextCode>, width: usize) {
    right_pad_line_with(line, line_width(line), width);
}

pub(super) fn right_pad_line_with(line: &mut Vec<RichTextCode>, line_width: usize, width: usize) {
    if line_width < width {
        let extra_width = width - line_width;
        if let Some(RichTextCode::Text { text, width: text_width }) = line.last_mut() {
            text.reserve(extra_width);
            for _ in 0..extra_width {
                text.push(' ');
            }
            *text_width += extra_width;
        } else {
            let text = " ".repeat(extra_width);
            line.push(RichTextCode::Text { text, width: extra_width });
        }
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

pub const DEFAULT_STYLE: RichTextStyle = RichTextStyle {
    font_weight: FontWeight::Normal,
    text_decoration: TextDecoration::None,
    font_style: FontStyle::Normal,
    foreground: Color::Default,
    background: Color::Default,
};

pub const CONTROL_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::Color16(Color16::Blue),
    ..DEFAULT_STYLE
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RichTextStyle {
    pub font_weight: FontWeight,
    pub text_decoration: TextDecoration,
    pub font_style: FontStyle,
    pub foreground: Color,
    pub background: Color,
}

impl Default for RichTextStyle {
    #[inline]
    fn default() -> Self {
        DEFAULT_STYLE
    }
}

impl RichTextStyle {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn is_default(&self) -> bool {
        self == &DEFAULT_STYLE
    }

    pub fn diff(&self, new_style: &RichTextStyle, code: &mut Vec<RichTextCode>) {
        if std::ptr::eq(self, new_style) {
            return;
        }

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

    pub fn write_diff(&self, new_style: &RichTextStyle, write: &mut impl std::io::Write) -> std::io::Result<()> {
        if std::ptr::eq(self, new_style) {
            return Ok(());
        }

        if self.font_weight != new_style.font_weight {
            new_style.font_weight.write(write)?;
        }

        if self.text_decoration != new_style.text_decoration {
            new_style.text_decoration.write(write)?;
        }

        if self.font_style != new_style.font_style {
            new_style.font_style.write(write)?;
        }

        if self.foreground != new_style.foreground {
            write_fg(write, new_style.foreground)?;
        }

        if self.background != new_style.background {
            write_bg(write, new_style.background)?;
        }

        Ok(())
    }

    #[inline]
    pub fn build() -> RichTextStyleBuilder {
        RichTextStyleBuilder::default()
    }

    pub fn apply_changes(&mut self, line: &[RichTextCode]) {
        for code in line {
            match code {
                RichTextCode::FontWeight(font_weight) => {
                    self.font_weight = *font_weight;
                }
                RichTextCode::FontStyle(font_style) => {
                    self.font_style = *font_style;
                }
                RichTextCode::TextDecoration(text_decoration) => {
                    self.text_decoration = *text_decoration;
                }
                RichTextCode::Foreground(color) => {
                    self.foreground = *color;
                }
                RichTextCode::Background(color) => {
                    self.background = *color;
                }
                RichTextCode::Text { .. } => {}
            }
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

impl RichTextCode {
    #[inline]
    pub fn width(&self) -> usize {
        match self {
            RichTextCode::Text { width, .. } => *width,
            _ => 0,
        }
    }
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

fn append_font_weight(line: &mut Vec<RichTextCode>, new_font_weight: FontWeight) {
    let mut found = false;
    for code in line.iter_mut().rev() {
        match code {
            RichTextCode::FontWeight(font_weigth) => {
                *font_weigth = new_font_weight;
                found = true;
            }
            RichTextCode::Text { .. } => {
                break;
            }
            _ => {}
        }
    }

    if !found {
        line.push(RichTextCode::FontWeight(new_font_weight));
    }
}

fn append_font_style(line: &mut Vec<RichTextCode>, new_font_style: FontStyle) {
    let mut found = false;
    for code in line.iter_mut().rev() {
        match code {
            RichTextCode::FontStyle(font_style) => {
                *font_style = new_font_style;
                found = true;
            }
            RichTextCode::Text { .. } => {
                break;
            }
            _ => {}
        }
    }

    if !found {
        line.push(RichTextCode::FontStyle(new_font_style));
    }
}

fn append_text_decoration(line: &mut Vec<RichTextCode>, new_text_decoration: TextDecoration) {
    let mut found = false;
    for code in line.iter_mut().rev() {
        match code {
            RichTextCode::TextDecoration(text_decoration) => {
                *text_decoration = new_text_decoration;
                found = true;
            }
            RichTextCode::Text { .. } => {
                break;
            }
            _ => {}
        }
    }

    if !found {
        line.push(RichTextCode::TextDecoration(new_text_decoration));
    }
}

fn append_foreground(line: &mut Vec<RichTextCode>, new_color: Color) {
    let mut found = false;
    for code in line.iter_mut().rev() {
        match code {
            RichTextCode::Foreground(color) => {
                *color = new_color;
                found = true;
            }
            RichTextCode::Text { .. } => {
                break;
            }
            _ => {}
        }
    }

    if !found {
        line.push(RichTextCode::Foreground(new_color));
    }
}

fn append_background(line: &mut Vec<RichTextCode>, new_color: Color) {
    let mut found = false;
    for code in line.iter_mut().rev() {
        match code {
            RichTextCode::Background(color) => {
                *color = new_color;
                found = true;
            }
            RichTextCode::Text { .. } => {
                break;
            }
            _ => {}
        }
    }

    if !found {
        line.push(RichTextCode::Background(new_color));
    }
}
