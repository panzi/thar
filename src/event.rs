use std::fmt::Write;

pub const ESCAPE: u8 = 0x1B;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    CursorPosition { row: u32, column: u32 },
    WindowSize { rows: u32, columns: u32 },
    KeyPress { ctrl: bool, alt: bool, shift: bool, key: Key },
    Mouse { ctrl: bool, alt: bool, shift: bool, release: bool, button: u8, row: u32, column: u32 },
    Unsupported,
}

impl Event {
    #[inline]
    pub fn key(key: Key) -> Self {
        Self::KeyPress { ctrl: false, alt: false, shift: false, key }
    }

    #[inline]
    pub fn alt_key(key: Key) -> Self {
        Self::KeyPress { ctrl: false, alt: true, shift: false, key }
    }

    #[inline]
    pub fn from_char(ch: char) -> Self {
        Self::KeyPress { ctrl: false, alt: false, shift: ch.is_uppercase(), key: Key::from_char(ch) }
    }

    #[inline]
    pub fn from_char_alt(ch: char) -> Self {
        Self::KeyPress { ctrl: false, alt: true, shift: ch.is_uppercase(), key: Key::from_char(ch) }
    }
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Self::KeyPress { ctrl, alt, shift, key } = *self {
            f.write_str("KeyPress ")?;

            if shift {
                f.write_str("Shift")?;
            }

            if ctrl {
                if shift {
                    f.write_str("+Ctrl")?;
                } else {
                    f.write_str("Ctrl")?;
                }
            }

            if alt {
                if shift || ctrl {
                    f.write_str("+Alt")?;
                } else {
                    f.write_str("Alt")?;
                }
            }

            if shift || ctrl || alt {
                f.write_str("+")?;
            }

            std::fmt::Display::fmt(&key, f)
        } else {
            std::fmt::Debug::fmt(&self, f)
        }
    }
}

pub const ESCAPE_EVENT: Event = Event::KeyPress { ctrl: false, alt: false, shift: false, key: Key::Escape };

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Key {
    Char(char),
    Function(u16),
    Insert,
    Delete,
    Home,
    End,
    Up,
    Down,
    Left,
    Right,
    Keypad5,
    Backspace,
    PrintScreen, // TODO
    ScrollLock,  // TODO
    Pause,
    Enter,
    PageUp,
    PageDown,
    Escape,
}

impl Key {
    #[inline]
    pub fn from_char(ch: char) -> Self {
        match ch {
            '\r' | '\n' => Self::Enter,
            '\x1B' => Self::Escape,
            '\x7F' => Self::Backspace,
            _ => Self::Char(ch),
        }
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Char(' ') => f.write_str("Space"),
            Self::Char('\t') => f.write_str("Tab"),
            Self::Char(ch) => {
                let upper = ch.to_uppercase();
                if upper.len() == 1 {
                    for ch in upper {
                        f.write_char(ch)?;
                    }
                } else {
                    f.write_char(ch)?;
                }

                Ok(())
            },
            Self::Function(n) => write!(f, "F{n}"),
            _ => std::fmt::Debug::fmt(&self, f),
        }
    }
}
