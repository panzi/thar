use crate::{rich_text::RichText, styles::{ATTR_STYLE, COMMENT_STYLE, DOCTYPE_STYLE, ERROR_STYLE, ESCAPE_STYLE, KEYWORD_STYLE, NUMBER_STYLE, STRING_STYLE, SYMBOL_STYLE, TAG_STYLE}};

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
                    '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' | '"' => {
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

fn find_nested_angle_end(sgml: &str, index: usize) -> Option<usize> {
    let mut nesting = 1;
    for (sub_index, ch) in sgml[index..].char_indices() {
        match ch {
            '>' => {
                nesting -= 1;
                if nesting == 0 {
                    return Some(index + sub_index + 1);
                }
            }
            '<' => {
                nesting += 1;
            }
            _ => {}
        }
    }

    None
}

pub fn colorize_sgml(sgml: &str, rich_text: &mut RichText) {
    let mut index = 0;

    while index < sgml.len() {
        let mut prev = index;
        index = find_sgml_start(sgml, index);

        if index > prev {
            rich_text.append_plain_text(&sgml[prev..index]);
        }
        prev = index;

        if index >= sgml.len() {
            break;
        }

        const CDATA_START: &str = "<![CDATA[";
        const CDATA_END: &str = "]]>";
        const COMMENT_START: &str = "<!--";
        const COMMENT_END: &str = "-->";
        const DOCTYPE_START: &str = "<!DOCTYPE";
        const PI_START: &str = "<?";
        const PI_END: &str = "?>";

        let suffix = &sgml[index..];
        if suffix.starts_with(CDATA_START) {
            // CDATA section
            index += CDATA_START.len();
            let Some(sub_index) = sgml[index..].find(CDATA_END) else {
                rich_text.append_text(&ERROR_STYLE, &sgml[prev..]);
                return;
            };
            let end_index = index + sub_index;

            rich_text.append_text(&COMMENT_STYLE, &sgml[prev..index]);
            prev = index;
            index = end_index;
            rich_text.append_plain_text(&sgml[prev..index]);
            prev = index;
            index += CDATA_END.len();
            rich_text.append_text(&COMMENT_STYLE, &sgml[prev..index]);
        } else if suffix.starts_with(COMMENT_START) {
            // comment
            index += COMMENT_START.len();
            let Some(sub_index) = sgml[index..].find(COMMENT_END) else {
                rich_text.append_text(&COMMENT_STYLE, &sgml[prev..]);
                return;
            };
            index += sub_index + COMMENT_END.len();
            rich_text.append_text(&COMMENT_STYLE, &sgml[prev..index]);
        } else if suffix.len() > DOCTYPE_START.len() &&
            suffix[..DOCTYPE_START.len()].eq_ignore_ascii_case(DOCTYPE_START) &&
            suffix[DOCTYPE_START.len()..].starts_with(|ch: char| !ch.is_alphanumeric() && ch != '_' && ch != '-') {
            // DOCTYPE
            index += DOCTYPE_START.len();
            let Some(next_index) = find_nested_angle_end(sgml, index) else {
                rich_text.append_text(&ERROR_STYLE, &sgml[prev..]);
                return;
            };

            index = next_index;
            rich_text.append_text(&DOCTYPE_STYLE, &sgml[prev..index]);
        } else if suffix.starts_with("<!") {
            // error
            index += 2;
            let Some(next_index) = find_nested_angle_end(sgml, index) else {
                rich_text.append_text(&ERROR_STYLE, &sgml[prev..]);
                return;
            };

            index = next_index;
            rich_text.append_text(&ERROR_STYLE, &sgml[prev..index]);
        } else if suffix.starts_with(PI_START) {
            // processing instructions
            index += PI_START.len();
            rich_text.append_text(&SYMBOL_STYLE, &sgml[prev..index]);

            prev = index;
            index = find_word_end(sgml, index);

            rich_text.append_text(&TAG_STYLE, &sgml[prev..index]);

            index = parse_attributes(sgml, index, rich_text);
            prev = index;

            if sgml[index..].starts_with(PI_END) {
                index += PI_END.len();
                rich_text.append_text(&SYMBOL_STYLE, &sgml[prev..index]);
            } else if let Some(ch) = sgml[index..].chars().next() {
                index += ch.len_utf8();
                rich_text.append_text(&ERROR_STYLE, &sgml[prev..index]);
            }
        } else if suffix.starts_with("</") {
            // end tag
            index += "</".len();
            rich_text.append_text(&SYMBOL_STYLE, &sgml[prev..index]);

            prev = index;
            index = find_word_end(sgml, index);

            rich_text.append_text(&TAG_STYLE, &sgml[prev..index]);

            prev = index;
            index = skipws(sgml, index);

            if prev < index {
                rich_text.append_plain_text(&sgml[prev..index]);
                prev = index;
            }

            if sgml[index..].starts_with('>') {
                index += 1;
                rich_text.append_text(&SYMBOL_STYLE, &sgml[prev..index]);
            } else if let Some(ch) = sgml[index..].chars().next() {
                index += ch.len_utf8();
                rich_text.append_text(&ERROR_STYLE, &sgml[prev..index]);
            }
        } else if suffix.starts_with('<') {
            // start tag
            index += 1;
            rich_text.append_text(&SYMBOL_STYLE, &sgml[prev..index]);

            prev = index;
            index = find_word_end(sgml, index);

            rich_text.append_text(&TAG_STYLE, &sgml[prev..index]);

            index = parse_attributes(sgml, index, rich_text);
            prev = index;

            if sgml[index..].starts_with('>') {
                index += 1;
                rich_text.append_text(&SYMBOL_STYLE, &sgml[prev..index]);
            } else if sgml[index..].starts_with("/>") {
                index += 2;
                rich_text.append_text(&SYMBOL_STYLE, &sgml[prev..index]);
            } else if let Some(ch) = sgml[index..].chars().next() {
                index += ch.len_utf8();
                rich_text.append_text(&ERROR_STYLE, &sgml[prev..index]);
            }
        } else {
            // &
            // entity reference
            if let Some(next_index) = parse_entity_ref(sgml, index) {
                index = next_index;
                rich_text.append_text(&KEYWORD_STYLE, &sgml[prev..index]);
            } else {
                index += 1;
                rich_text.append_text(&ERROR_STYLE, &sgml[prev..index]);
            }
        }
    }
}

fn find_word_end(sgml: &str, index: usize) -> usize {
    let Some(sub_index) = sgml[index..].find(|ch: char| !ch.is_alphanumeric() && ch != '_' && ch != '-' && ch != ':') else {
        return sgml.len();
    };

    index + sub_index
}

fn parse_attributes(sgml: &str, mut index: usize, rich_text: &mut RichText) -> usize {
    let mut prev = index;

    loop {
        index = skipws(sgml, index);

        if prev < index {
            rich_text.append_plain_text(&sgml[prev..index]);
        }

        prev = index;

        if !sgml[index..].starts_with(|ch: char| ch.is_alphanumeric() || ch == '_' || ch == '-') {
            return index;
        }

        index = find_word_end(sgml, index);

        rich_text.append_text(&ATTR_STYLE, &sgml[prev..index]);

        prev = index;
        index = skipws(sgml, index);

        if prev < index {
            rich_text.append_plain_text(&sgml[prev..index]);
            prev = index;
        }

        if sgml[index..].starts_with('=') {
            index += 1;

            rich_text.append_text(&SYMBOL_STYLE, &sgml[prev..index]);

            prev = index;
            index = skipws(sgml, index);

            if prev < index {
                rich_text.append_plain_text(&sgml[prev..index]);
                prev = index;
            }

            let suffix = &sgml[index..];
            if suffix.starts_with(|ch: char| ch.is_alphanumeric() || ch == '_' || ch == '-') {
                index = find_plain_attr_value_end(sgml, index);

                rich_text.append_text(&STRING_STYLE, &sgml[prev..index]);
            } else if suffix.starts_with('"') {
                index = parse_attr_value(sgml, index, '"', rich_text);
            } else if suffix.starts_with('\'') {
                index = parse_attr_value(sgml, index, '\'', rich_text);
            } else {
                return index;
            }
        }

        prev = index;
    }
}

fn find_plain_attr_value_end(sgml: &str, index: usize) -> usize {
    // TODO: entity refs in unquoted attributes?
    let Some(sub_index) = sgml[index..].find(|ch: char| ch.is_whitespace() || ch == '>' || ch == '<' || ch == '"' || ch == '\'') else {
        return sgml.len();
    };

    index + sub_index
}

fn parse_attr_value(sgml: &str, mut index: usize, quote: char, rich_text: &mut RichText) -> usize {
    let mut prev = index;

    if !sgml[index..].starts_with(quote) {
        return sgml.len();
    }

    index += 1;

    loop {
        let Some(sub_index) = sgml[index..].find(|ch: char| ch == '&' || ch == quote) else {
            rich_text.append_text(&ERROR_STYLE, &sgml[index..]);
            return sgml.len();
        };
        index += sub_index;

        if prev < index {
            rich_text.append_text(&STRING_STYLE, &sgml[prev..index]);
            prev = index;
        }

        let Some(ch) = sgml[index..].chars().next() else {
            rich_text.append_text(&ERROR_STYLE, &sgml[index..]);
            return sgml.len();
        };

        if ch == '&' {
            let Some(next_index) = parse_entity_ref(sgml, index) else {
                rich_text.append_text(&ERROR_STYLE, &sgml[index..]);
                return sgml.len();
            };
            index = next_index;
            rich_text.append_text(&KEYWORD_STYLE, &sgml[prev..index]);
        } else {
            index += 1;
            rich_text.append_text(&STRING_STYLE, &sgml[prev..index]);
            return index;
        }
        prev = index;
    }
}

fn parse_entity_ref(sgml: &str, mut index: usize) -> Option<usize> {
    let Some(ch) = sgml[index..].chars().next() else {
        return None;
    };

    if ch != '&' {
        return None;
    }

    index += 1;
    let Some(mut ch) = sgml[index..].chars().next() else {
        return None;
    };

    if ch == '#' {
        index += 1;
        let Some(next_ch) = sgml[index..].chars().next() else {
            return None;
        };
        ch = next_ch;

        if ch.eq_ignore_ascii_case(&'x') {
            index += 1;
            let Some(next_ch) = sgml[index..].chars().next() else {
                return None;
            };
            ch = next_ch;

            if !ch.is_ascii_hexdigit() {
                return None;
            }

            while ch.is_ascii_hexdigit() {
                index += 1;
                let Some(next_ch) = sgml[index..].chars().next() else {
                    return None;
                };
                ch = next_ch;
            }
        } else if ch.is_ascii_digit() {
            while ch.is_ascii_digit() {
                index += 1;
                let Some(next_ch) = sgml[index..].chars().next() else {
                    return None;
                };
                ch = next_ch;
            }
        } else {
            return None;
        }
    } else if ch.is_alphanumeric() || ch == '_' {
        while ch.is_alphanumeric() || ch == '_' {
            index += 1;
            let Some(next_ch) = sgml[index..].chars().next() else {
                return None;
            };
            ch = next_ch;
        }
    } else {
        return None;
    }

    if ch != ';' {
        return None;
    }

    index += 1;

    Some(index)
}
