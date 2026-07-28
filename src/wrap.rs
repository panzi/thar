use crate::char_width::CharWidth;


#[derive(Debug)]
pub struct LineWrapper<'a> {
    text: &'a str,
    prefix_width: usize,
    wrap_width: usize,
}

impl<'a> LineWrapper<'a> {
    #[inline]
    pub fn new(text: &'a str, wrap_width: usize) -> Self {
        Self { text, prefix_width: 0, wrap_width }
    }

    #[inline]
    pub fn with_prefix_width(text: &'a str, wrap_width: usize, prefix_width: usize) -> Self {
        Self { text, prefix_width, wrap_width }
    }

    #[inline]
    pub fn text(&self) -> &'a str {
        self.text
    }

    #[inline]
    pub fn prefix_width(&self) -> usize {
        self.prefix_width
    }

    #[inline]
    pub fn wrap_width(&self) -> usize {
        self.wrap_width
    }
}

#[inline]
pub fn wrap<'a>(text: &'a str, wrap_width: usize) -> LineWrapper<'a> {
    LineWrapper::new(text, wrap_width)
}

impl<'a> Iterator for LineWrapper<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.text.is_empty() {
            return None;
        }

        let mut index = 0;
        let mut line_width = self.prefix_width;
        loop {
            if index == self.text.len() {
                let line = &self.text[..index];
                self.text = &self.text[index..];
                self.prefix_width = 0;
                return Some(line);
            }

            if self.text[index..].starts_with('\n') {
                let line = &self.text[..index];
                self.text = &self.text[index + 1..];
                self.prefix_width = 0;
                return Some(line);
            }

            let prev = index;
            index = find_wrap_point(self.text, index);
            if index == prev {
                // at word boundary
                index += self.text[index..].ceil_char_boundary(1);
            }

            let slice = &self.text[prev..index];

            let word_width = slice.char_width_ignore_unprintable();

            if line_width + word_width <= self.wrap_width {
                line_width += word_width;
            } else {
                index = prev;

                if line_width == 0 {
                    // wrap overly long words anywhere
                    for (char_index, ch) in self.text[index..].char_indices() {
                        if ch == '\n' {
                            index += char_index;
                            let line = &self.text[..index];
                            self.text = &self.text[index + 1..];
                            self.prefix_width = 0;
                            return Some(line);
                        }

                        let char_width = ch.char_width_ignore_unprintable();
                        if line_width + char_width > self.wrap_width {
                            index += char_index;
                            break;
                        }

                        line_width += char_width;
                    }
                }

                let line = &self.text[..index];
                self.text = self.text[index..].trim_start();
                self.prefix_width = 0;
                return Some(line);
            }
        }
    }
}

#[inline]
pub fn find_wrap_point(text: &str, index: usize) -> usize {
    let Some(sub_index) = text[index..].find(|ch: char| !ch.is_alphabetic() && ch != '_') else {
        return text.len();
    };

    index + sub_index
}
