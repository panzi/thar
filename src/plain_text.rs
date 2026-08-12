use crate::{char_width::{CharWidth, crop}, styles::{CONTROL_STYLE, DEFAULT_STYLE}, termio::TermIO, wrap::find_wrap_point};


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlainTextItem {
    Newline,
    Text { text: String, width: usize },
    Special(char),
}

impl PlainTextItem {
    #[inline]
    pub fn width(&self) -> usize {
        match self {
            PlainTextItem::Newline => 0,
            PlainTextItem::Special(..) => 1,
            PlainTextItem::Text { width, .. } => *width,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlainText {
    pub(super) lines: Vec<Vec<PlainTextItem>>,
    pub(super) width: usize,
}

impl PlainText {
    #[inline]
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            width: 0,
        }
    }

    #[inline]
    pub fn parse(text: &str) -> Self {
        let mut plain_text = PlainText::new();
        plain_text.append(text);
        plain_text
    }

    #[inline]
    pub fn lines(&self) -> &[Vec<PlainTextItem>] {
        &self.lines
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
    pub fn clear(&mut self) {
        self.lines.clear();
        self.width = 0;
    }

    #[inline]
    pub fn into_inner(self) -> Vec<Vec<PlainTextItem>> {
        self.lines
    }

    pub fn right_pad(&mut self, width: usize) {
        if width > self.width {
            for line in &mut self.lines {
                let line_width = line_width(line);

                if line_width < width {
                    let diff = width - line_width;
                    if let Some(PlainTextItem::Text { text, width: text_width }) = line.last_mut() {
                        text.reserve(diff);
                        for _ in 0..diff {
                            text.push(' ');
                        }
                        *text_width += diff;
                    } else {
                        let text = " ".repeat(diff);
                        line.push(PlainTextItem::Text { text, width: diff });
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
                    if let Some(PlainTextItem::Text { text, width: text_width }) = line.first_mut() {
                        let mut new_text = String::with_capacity(text.len() + diff);
                        for _ in 0..diff {
                            new_text.push(' ');
                        }
                        new_text.push_str(&text);
                        *text = new_text;
                        *text_width += diff;
                    } else {
                        let text = " ".repeat(diff);
                        line.insert(0, PlainTextItem::Text { text, width: diff });
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

    pub fn vertical_append(&mut self, other: &PlainText) {
        self.right_pad(self.width);

        for (self_line, other_line) in self.lines.iter_mut().zip(other.lines.iter()) {
            self_line.extend_from_slice(&other_line);
        }

        if self.lines.len() < other.lines.len() {
            self.lines.reserve(other.lines.len() - self.lines.len());
            for other_line in &other.lines[self.lines.len()..] {
                let mut self_line = Vec::with_capacity(other_line.len() + (self.width > 0) as usize);
                if self.width > 0 {
                    let text = " ".repeat(self.width);
                    self_line.push(PlainTextItem::Text { text, width: self.width });
                }
                self_line.extend_from_slice(&other_line);
                self.lines.push(self_line);
            }
        }

        self.width += other.width;
    }

    pub fn append(&mut self, text: &str) {
        let mut line = if let Some(line) = self.lines.last_mut() {
            if line.last() == Some(&PlainTextItem::Newline) {
                self.lines.push_mut(Vec::new())
            } else {
                line
            }
        } else {
            self.lines.push_mut(Vec::new())
        };

        let mut line_width = line_width(line);
        let mut prev_index = 0;
        let mut prev_width = 0;

        for (index, ch) in text.char_indices() {
            if ch == '\n' {
                if prev_index < index {
                    if let Some(PlainTextItem::Text { text: item_text, width: item_width }) = line.last_mut() {
                        item_text.push_str(&text[prev_index..index]);
                        *item_width += line_width - prev_width;
                    } else {
                        line.push(PlainTextItem::Text {
                            text: text[prev_index..index].to_string(),
                            width: line_width - prev_width,
                        });
                    }
                }

                line.push(PlainTextItem::Newline);

                if line_width > self.width {
                    self.width = line_width;
                }

                line_width = 0;
                prev_width = 0;
                prev_index = index + 1;
                line = self.lines.push_mut(Vec::new());
            } else if ch.is_ascii_control() {
                if prev_index < index {
                    if let Some(PlainTextItem::Text { text: item_text, width: item_width }) = line.last_mut() {
                        item_text.push_str(&text[prev_index..index]);
                        *item_width += line_width - prev_width;
                    } else {
                        line.push(PlainTextItem::Text {
                            text: text[prev_index..index].to_string(),
                            width: line_width - prev_width,
                        });
                    }
                }

                line.push(PlainTextItem::Special(ch));

                line_width += 1;
                prev_width = line_width;
                prev_index = index + 1;
            } else {
                line_width += ch.char_width_ignore_unprintable();
            }
        }

        if prev_index < text.len() {
            if let Some(PlainTextItem::Text { text: item_text, width: item_width }) = line.last_mut() {
                item_text.push_str(&text[prev_index..]);
                *item_width += line_width - prev_width;
            } else {
                line.push(PlainTextItem::Text {
                    text: text[prev_index..].to_string(),
                    width: line_width - prev_width,
                });
            }
        }

        if line_width > self.width {
            self.width = line_width;
        }
    }

    pub fn wrap(&self, wrap_width: usize) -> Self {
        let wrap_width = wrap_width.max(1);

        if self.width <= wrap_width {
            return self.clone();
        }

        let mut line_width = 0;
        let mut lines = Vec::with_capacity(self.lines.len());
        let mut width = 0;

        for line in &self.lines {
            let mut new_line = lines.push_mut(Vec::with_capacity(line.len()));

            for item in line {
                match item {
                    &PlainTextItem::Newline => {
                        new_line.push(PlainTextItem::Newline);
                    }
                    &PlainTextItem::Special(ch) => {
                        if line_width + 1 > wrap_width {
                            if line_width > width {
                                width = line_width;
                            }
                            line_width = 0;
                            new_line = lines.push_mut(Vec::new());
                        }
                        new_line.push(PlainTextItem::Special(ch));
                        line_width += 1;
                    }
                    PlainTextItem::Text { text, width: text_width } => {
                        let text_width = *text_width;

                        if line_width + text_width <= wrap_width {
                            new_line.push(PlainTextItem::Text { text: text.clone(), width: text_width });
                            line_width += text_width;
                        } else {
                            let mut index = 0;

                            while index < text.len() {
                                let prev = index;
                                index = find_wrap_point(text, index);
                                if index == prev {
                                    // at word boundary
                                    index += text[index..].ceil_char_boundary(1);
                                }

                                let slice = &text[prev..index];
                                let word_width = slice.char_width_ignore_unprintable();

                                if line_width + word_width <= wrap_width {
                                    line_width += word_width;
                                } else {
                                    index = prev;

                                    if line_width == 0 {
                                        // wrap overly long words anywhere
                                        for (char_index, ch) in text[index..].char_indices() {
                                            let char_width = ch.char_width_ignore_unprintable();
                                            if line_width + char_width > wrap_width {
                                                index += char_index;
                                                break;
                                            }

                                            line_width += char_width;
                                        }
                                    }

                                    let line = &text[..index];
                                    if line.len() > 0 {
                                        new_line.push(PlainTextItem::Text {
                                            text: line.to_string(),
                                            width: line_width,
                                        });
                                    }

                                    if index == text.len() {
                                        break;
                                    }

                                    line_width = 0;
                                    new_line = lines.push_mut(Vec::new());
                                }
                            }
                        }
                    }
                }
            }
        }

        Self { lines, width }
    }

    #[inline]
    pub fn draw(&self, termio: &mut TermIO, row: i32, column: i32) -> std::io::Result<()> {
        self.draw_cropped(
            termio,
            row, column,
            0, 0,
            self.width.min(u32::MAX as usize) as u32,
            self.height().min(u32::MAX as usize) as u32,
        )
    }

    pub fn draw_cropped(&self, termio: &mut TermIO, row: i32, column: i32, crop_row: u32, crop_column: u32, crop_width: u32, crop_height: u32) -> std::io::Result<()> {
        let lines = &self.lines;

        let mut crop_row = crop_row as usize;
        let mut crop_column = crop_column as usize;

        let mut crop_row_end = crop_row + crop_height as usize;
        let mut crop_column_end = crop_column + crop_width as usize;

        let mut full_column_end = crop_column_end;

        if crop_row_end > lines.len() {
            crop_row_end = lines.len();
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

        let lines = &lines[crop_row..crop_row_end];

        let mut first = true;
        let mut prev_line_index = 0;

        for (line_index, line) in lines.iter().enumerate() {
            let mut moved = false;
            let mut line_width = 0;

            for code in line {
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

                match code {
                    &PlainTextItem::Special(ch) => {
                        let display_char = if ch == '\x7F' {
                            '\u{2421}'
                        } else {
                            unsafe { char::from_u32_unchecked(0x2400 + ch as u32) }
                        };

                        if line_width >= crop_column && line_width + 1 < crop_column_end {
                            DEFAULT_STYLE.write_diff(&CONTROL_STYLE, termio)?;
                            termio.write_str(display_char.encode_utf8(&mut [0; char::MAX_LEN_UTF8]))?;
                            CONTROL_STYLE.write_diff(&DEFAULT_STYLE, termio)?;
                        }

                        line_width += 1;
                    }
                    PlainTextItem::Text { text, width: text_width } => {
                        let text_width = *text_width;
                        if line_width >= crop_column && line_width + text_width <= crop_column_end {
                            termio.write_str(text)?;
                        } else if line_width + text_width >= crop_column && line_width < crop_column_end {
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
                    PlainTextItem::Newline => {
                        /* Only exists to distinquish wrapped lines from actual lines. */
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
        for line in &self.lines {
            for item in line {
                match item {
                    &PlainTextItem::Special(ch) => {
                        let display_char = if ch == '\x7F' {
                            '\u{2421}'
                        } else {
                            unsafe { char::from_u32_unchecked(0x2400 + ch as u32) }
                        };

                        DEFAULT_STYLE.write_diff(&CONTROL_STYLE, write)?;
                        write.write_all(display_char.encode_utf8(&mut [0; char::MAX_LEN_UTF8]).as_bytes())?;
                        CONTROL_STYLE.write_diff(&DEFAULT_STYLE, write)?;
                    }
                    PlainTextItem::Text { text, .. } => {
                        write.write_all(text.as_bytes())?;
                    }
                    PlainTextItem::Newline => {
                        /* Only exists to distinquish wrapped lines from actual lines. */
                    }
                }
            }
            write.write_all(b"\n")?;
        }

        Ok(())
    }

    pub fn write(&self, buf: &mut String) {
        for line in &self.lines {
            for item in line {
                match item {
                    &PlainTextItem::Special(ch) => {
                        let display_char = if ch == '\x7F' {
                            '\u{2421}'
                        } else {
                            unsafe { char::from_u32_unchecked(0x2400 + ch as u32) }
                        };

                        buf.push(display_char);
                    }
                    PlainTextItem::Text { text, .. } => {
                        buf.push_str(text);
                    }
                    PlainTextItem::Newline => {
                        /* Only exists to distinquish wrapped lines from actual lines. */
                    }
                }
            }
            buf.push('\n');
        }
    }

    /// Number of bytes that will be written by [Self::write()].
    pub fn write_len(&self) -> usize {
        let mut len = 0;

        for line in &self.lines {
            for item in line {
                match item {
                    PlainTextItem::Special(_) => {
                        len += 1;
                    }
                    PlainTextItem::Text { text, .. } => {
                        len += text.len();
                    }
                    PlainTextItem::Newline => {}
                }
            }
            len += 1;
        }

        len
    }

    pub fn unwrap(&self) -> Self {
        let mut new_lines = Vec::new();
        let mut last_line: Option<&mut Vec<PlainTextItem>> = None;
        let mut width = 0;

        for line in &self.lines {
            if let Some(actual_last_line) = &mut last_line {
                actual_last_line.extend_from_slice(line);

                if line.last() == Some(&PlainTextItem::Newline) {
                    let line_width = line_width(actual_last_line);
                    if line_width > width {
                        width = line_width;
                    }
                    last_line = None;
                }
            } else {
                let new_line = new_lines.push_mut(line.clone());

                if new_line.last() == Some(&PlainTextItem::Newline) {
                    let line_width = line_width(new_line);
                    if line_width > width {
                        width = line_width;
                    }
                    last_line = None;
                } else {
                    last_line = Some(new_line);
                }
            }
        }

        if let Some(last_line) = last_line {
            let line_width = line_width(last_line);
            if line_width > width {
                width = line_width;
            }
        }

        Self { lines: new_lines, width }
    }

    #[inline]
    pub fn to_string(&self) -> String {
        let mut buf = String::with_capacity(self.write_len());

        self.write(&mut buf);

        buf
    }
}

pub fn line_width(line: &[PlainTextItem]) -> usize {
    let mut line_width = 0;

    for item in line {
        line_width += item.width();
    }

    line_width
}

impl From<&str> for PlainText {
    #[inline]
    fn from(value: &str) -> Self {
        Self::parse(value)
    }
}

impl From<String> for PlainText {
    #[inline]
    fn from(value: String) -> Self {
        Self::parse(&value)
    }
}

impl From<&PlainText> for String {
    #[inline]
    fn from(value: &PlainText) -> Self {
        value.to_string()
    }
}
