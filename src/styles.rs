// TODO: put in struct that is passed around to make configurable

use crate::{color::{Color, Color16}, rich_text::RichTextStyle, style::{FontStyle, FontWeight, TextDecoration}};

// table and property list

pub const EVEN_ROW_BACKGROUND:          Color = Color::from_u32(0x111111);
pub const ODD_ROW_BACKGROUND:           Color = Color::from_u32(0x000000);
pub const SELECTED_EVEN_ROW_BACKGROUND: Color = Color::from_u32(0x333333);
pub const SELECTED_ODD_ROW_BACKGROUND:  Color = Color::from_u32(0x222222);
pub const TABLE_FOREGROUND:             Color = Color::from_u32(0xFFFFFF);

// rich text

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

// colorize

pub const ERROR_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::from_u32(0xFFFFFF),
    background: Color::from_u32(0xFF2200),
    ..DEFAULT_STYLE
};

pub const NUMBER_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::from_u32(0x00FF00),
    ..DEFAULT_STYLE
};

pub const SYMBOL_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::from_u32(0x0088FF),
    ..DEFAULT_STYLE
};

pub const KEYWORD_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::from_u32(0x00FFFF),
    ..DEFAULT_STYLE
};

pub const STRING_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::from_u32(0xFF5500),
    ..DEFAULT_STYLE
};

pub const ESCAPE_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::from_u32(0xFFAA00),
    ..DEFAULT_STYLE
};

pub const COMMENT_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::from_u32(0xAAAAAA),
    ..DEFAULT_STYLE
};

pub const DOCTYPE_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::from_u32(0x00FF00),
    ..DEFAULT_STYLE
};

pub const TAG_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::from_u32(0xFFAA00),
    ..DEFAULT_STYLE
};

pub const ATTR_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::from_u32(0x00AAFF),
    ..DEFAULT_STYLE
};

// fields

pub const FIELD_ERROR_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::Color16(Color16::Red),
    ..DEFAULT_STYLE
};
