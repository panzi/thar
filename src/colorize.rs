use crate::{color::{Color, Color16}, rich_text::{DEFAULT_STYLE, RichText, RichTextStyle}};

fn skipws(text: &str, index: usize) -> usize {
    let Some(sub_index) = text[index..].find(|ch: char| !ch.is_whitespace()) else {
        return text.len();
    };

    index + sub_index
}

fn find_ok_json_char(text: &str, index: usize) -> usize {
    let Some(sub_index) = text[index..].find(|ch: char|
        ch.is_whitespace() ||
        ch.is_ascii_digit() ||
        matches!(ch, '"' | '{' | '}' | '[' | ']' | ',' | ':' | '-')
    ) else {
        return text.len();
    };

    index + sub_index
}

const ERROR_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::from_u32(0xFFFFFF),
    background: Color::from_u32(0xFF2200),
    ..DEFAULT_STYLE
};

const NUMBER_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::from_u32(0x00FF00),
    ..DEFAULT_STYLE
};

const SYMBOL_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::from_u32(0x0088FF),
    ..DEFAULT_STYLE
};

const KEYWORD_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::from_u32(0x00FFFF),
    ..DEFAULT_STYLE
};

const STRING_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::from_u32(0xFF2200),
    ..DEFAULT_STYLE
};

const ESCAPE_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::from_u32(0xFFAA00),
    ..DEFAULT_STYLE
};

const COMMENT_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::from_u32(0xAAAAAA),
    ..DEFAULT_STYLE
};

fn colorize_string(text: &str, mut index: usize, rich_text: &mut RichText) -> usize {
    let mut prev = index;

    if !text[index..].starts_with('"') {
        return index;
    }

    index += 1;

    while let Some(ch) = text[index..].chars().next() {
        match ch {
            '"' => {
                index += 1;
                rich_text.append_text(&STRING_STYLE, &text[prev..index]);
                return index;
            }
            '\\' => {
                if index > prev {
                    rich_text.append_text(&STRING_STYLE, &text[prev..index]);
                }
                prev = index;

                index += 1;
                let Some(ch) = text[index..].chars().next() else {
                    rich_text.append_text(&ERROR_STYLE, &text[prev..]);
                    return text.len();
                };

                match ch {
                    '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' => {
                        index += 1;
                        rich_text.append_text(&ESCAPE_STYLE, &text[prev..index]);
                    }
                    'u' => {
                        index += 1;
                        let mut count = 0;
                        while count < 4 && index < text.len() && text[index..].starts_with(|ch: char| ch.is_ascii_hexdigit()) {
                            index += 1;
                            count += 1;
                        }

                        if count != 4 {
                            rich_text.append_text(&ERROR_STYLE, &text[prev..index]);
                        } else {
                            rich_text.append_text(&ESCAPE_STYLE, &text[prev..index]);
                        }
                    }
                    _ => {
                        rich_text.append_text(&ERROR_STYLE, "\\");
                    }
                }

                prev = index;
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    text.len()
}

fn colorize_number(text: &str, mut index: usize, rich_text: &mut RichText) -> usize {
    let prev = index;

    let Some(mut ch) = text[index..].chars().next() else {
        return index;
    };

    if ch == '-' {
        index += 1;

        let Some(next_ch) = text[index..].chars().next() else {
            rich_text.append_text(&ERROR_STYLE, "-");
            return index;
        };

        ch = next_ch;
    }

    if !ch.is_ascii_digit() {
        if index > prev {
            rich_text.append_text(&ERROR_STYLE, &text[prev..index]);
        }
        return index;
    }

    while ch.is_ascii_digit() {
        index += 1;
        let Some(next_ch) = text[index..].chars().next() else {
            break;
        };

        ch = next_ch;
    }

    let Some(mut ch) = text[index..].chars().next() else {
        rich_text.append_text(&NUMBER_STYLE, &text[prev..index]);
        return index;
    };

    if ch == '.' {
        index += 1;

        let Some(next_ch) = text[index..].chars().next() else {
            rich_text.append_text(&ERROR_STYLE, &text[prev..index]);
            return index;
        };

        ch = next_ch;

        if !ch.is_ascii_digit() {
            rich_text.append_text(&ERROR_STYLE, &text[prev..index]);
            return index;
        }

        while ch.is_ascii_digit() {
            index += 1;
            let Some(next_ch) = text[index..].chars().next() else {
                break;
            };

            ch = next_ch;
        }
    }

    if ch == 'e' || ch == 'E' {
        index += 1;

        let Some(next_ch) = text[index..].chars().next() else {
            rich_text.append_text(&ERROR_STYLE, &text[prev..index]);
            return index;
        };

        ch = next_ch;

        if ch == '-' || ch == '+' {
            index += 1;

            let Some(next_ch) = text[index..].chars().next() else {
                rich_text.append_text(&ERROR_STYLE, &text[prev..index]);
                return index;
            };

            ch = next_ch;
        }

        if !ch.is_ascii_digit() {
            rich_text.append_text(&ERROR_STYLE, &text[prev..index]);
            return index;
        }

        while ch.is_ascii_digit() {
            index += 1;
            let Some(next_ch) = text[index..].chars().next() else {
                break;
            };

            ch = next_ch;
        }
    }

    if ch.is_alphanumeric() || ch == '_' {
        rich_text.append_text(&ERROR_STYLE, &text[prev..index]);
        return index;
    }


    rich_text.append_text(&NUMBER_STYLE, &text[prev..index]);

    index
}

#[inline]
pub fn startswith_keyword(text: &str, keyword: &str) -> bool {
    text.starts_with(keyword) && (text.len() == keyword.len() || text[keyword.len()..].starts_with(|ch: char| !ch.is_alphanumeric() && ch != '_'))
}

pub fn colorize_json(json: &str, rich_text: &mut RichText) {
    let mut prev = 0;
    let mut index = skipws(json, 0);

    loop {
        if index > prev {
            rich_text.append_plain_text(&json[prev..index]);
        }

        let suffix = &json[index..];

        if startswith_keyword(suffix, "null") {
            rich_text.append_text(&KEYWORD_STYLE, "null");
            index += "null".len();
        } else if startswith_keyword(suffix, "true") {
            rich_text.append_text(&KEYWORD_STYLE, "true");
            index += "true".len();
        } else if startswith_keyword(suffix, "false") {
            rich_text.append_text(&KEYWORD_STYLE, "false");
            index += "false".len();
        } else if let Some(ch) = suffix.chars().next() {
            match ch {
                '"' => {
                    index = colorize_string(json, index, rich_text);
                }
                '{' | '}' | '[' | ']' | ',' | ':' => {
                    rich_text.append_text(&SYMBOL_STYLE, &json[index..index + 1]);
                    index += 1;
                }
                _ => {
                    if ch.is_ascii_digit() || ch == '-' {
                        index = colorize_number(json, index, rich_text);
                    } else {
                        // error
                        let ok_index = find_ok_json_char(json, index);
                        if ok_index > index {
                            rich_text.append_text(&ERROR_STYLE, &json[index..ok_index]);
                        }
                        index = ok_index;
                    }
                }
            }
        } else {
            break;
        }

        prev = index;
        index = skipws(json, index);
    }
}

fn find_sgml_start(sgml: &str, index: usize) -> usize {
    let Some(sub_index) = sgml[index..].find(|ch: char| matches!(ch, '<' | '&')) else {
        return sgml.len();
    };

    index + sub_index
}

pub fn colorize_sgml(sgml: &str, rich_text: &mut RichText) {
    let mut prev = 0;
    let mut index = find_sgml_start(sgml, 0);

    loop {
        if index > prev {
            rich_text.append_plain_text(&sgml[prev..index]);
        }
        prev = index;

        let suffix = &sgml[index..];
        if suffix.starts_with("<![CDATA[") {
            // CDATA section
            index += "<![CDATA[".len();
            let Some(sub_index) = sgml[index..].find("]]>") else {
                rich_text.append_text(&ERROR_STYLE, &sgml[prev..]);
                return;
            };
            index += sub_index;
            rich_text.append_text(&COMMENT_STYLE, &sgml[prev..index]);
        } else if suffix.starts_with("<!--") {
            // comment
            index += "<!--".len();
            let Some(sub_index) = sgml[index..].find("-->") else {
                rich_text.append_text(&COMMENT_STYLE, &sgml[prev..]);
                return;
            };
            index += sub_index;
            rich_text.append_text(&COMMENT_STYLE, &sgml[prev..index]);
        } else if suffix.starts_with("<!DOCTYPE") {
            // TODO: DOCTYPE
        } else if suffix.starts_with("<?") {
            // TODO: processing instructions
        } else if suffix.starts_with('<') {
            // TODO: tags
        } else {
            // &
            // TODO: entity reference
        }

        prev = index;
        index = find_sgml_start(sgml, index);
    }
}
