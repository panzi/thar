use crate::color::{Color, Rgb};

pub const CLEAR_STYLE: &[u8] = b"\x1B[0m";
pub const BOLD: &[u8] = b"\x1B[1m";
pub const FAINT: &[u8] = b"\x1B[2m";
pub const ITALIC: &[u8] = b"\x1B[3m";
pub const UNDERLINE: &[u8] = b"\x1B[4m";
pub const DOUBLY_UNDERLINE: &[u8] = b"\x1B[21m";
pub const NORMAL_INTENSITY: &[u8] = b"\x1B[22m";
pub const NOT_ITALIC: &[u8] = b"\x1B[23m";
pub const NOT_UNDERLINE: &[u8] = b"\x1B[24m";

pub const FG_DEFAULT: &[u8] = b"\x1B[39m";
pub const BG_DEFAULT: &[u8] = b"\x1B[49m";

pub const CLEAR_SCREEN: &[u8] = b"\x1B[2J";
pub const CLEAR_LINE_TO_END: &[u8] = b"\x1B[0K";
pub const CLEAR_LINE_TO_START: &[u8] = b"\x1B[1K";
pub const CLEAR_LINE: &[u8] = b"\x1B[2K";

#[inline]
pub fn write_fg(write: &mut impl std::io::Write, color: Color) -> std::io::Result<()> {
    match color {
        Color::Default => write.write_all(FG_DEFAULT),
        Color::Rgb { r, g, b } => write_fg_rgb(write, Rgb { r, g, b }),
        Color::Color16(color) => write.write_all(color.fg()),
    }
}

#[inline]
pub fn write_fg_rgb(write: &mut impl std::io::Write, Rgb { r, g, b }: Rgb) -> std::io::Result<()> {
    write!(write, "\x1B[38;2;{r};{g};{b}m")
}

#[inline]
pub fn write_bg(write: &mut impl std::io::Write, color: Color) -> std::io::Result<()> {
    match color {
        Color::Default => write.write_all(BG_DEFAULT),
        Color::Rgb { r, g, b } => write_bg_rgb(write, Rgb { r, g, b }),
        Color::Color16(color) => write.write_all(color.bg()),
    }
}

#[inline]
pub fn write_bg_rgb(write: &mut impl std::io::Write, Rgb { r, g, b }: Rgb) -> std::io::Result<()> {
    write!(write, "\x1B[48;2;{r};{g};{b}m")
}
