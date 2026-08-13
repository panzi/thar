#[inline]
pub fn display_char(ch: char) -> char {
    if ch.is_ascii_control() {
        if ch == '\x7F' {
            '\u{2421}'
        } else {
            unsafe { char::from_u32_unchecked(0x2400 + ch as u32) }
        }
    } else {
        ch
    }
}
