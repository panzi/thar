use std::fmt::Write;

pub const ESCAPE: u8 = 0x1B;

// move = 35
// click left = 0
// click middle = 1
// click right = 2
// move+hold left = 32
// move+hold middle = 33
// move+hold right = 34

// ctrl+click left = 16
// ctrl+click middle = 17
// ctrl+click right = 18
// ctrl+move+hold left = 48
// ctrl+move+hold middle = 49
// ctrl+move+hold right = 50
// ctrl+move = 51

// alt+click left = ?
// alt+click middle = ?
// alt+click right = ?
// alt+move+hold left = ?
// alt+move+hold middle = ?
// alt+move+hold right = ?
// alt+move = 43

// ctrl+alt+click left = 24
// ctrl+alt+click middle = 25
// ctrl+alt+click right = 26
// ctrl+alt+move+hold left = 56
// ctrl+alt+move+hold middle = 57
// ctrl+alt+move+hold right = 58
// ctrl+alt+move = 59

// wheel up = 64
// wheel down = 65
// alt+wheel up = 72
// alt+wheel down = 73
// ctrl+wheel up = 80
// ctrl+wheel down = 81
// ctrl+alt+wheel up = 88
// ctrl+alt+wheel down = 89

pub const MOUSE_MASK_BUTTON: u32 = 3;
pub const MOUSE_MASK_SHIFT: u32 = 4;
pub const MOUSE_MASK_ALT: u32 = 8;
pub const MOUSE_MASK_CTRL: u32 = 16;
pub const MOUSE_MASK_MOVE: u32 = 32;
pub const MOUSE_MASK_WHEEL: u32 = 64;
pub const MOUSE_MASK_UNKNOWN: u32 = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    None,
    Left,
    Middle,
    Right,
}

impl MouseButton {
    #[inline]
    pub fn from_flags(flags: u32) -> Self {
        match flags & MOUSE_MASK_BUTTON {
            0 => Self::Left,
            1 => Self::Middle,
            2 => Self::Right,
            3 => Self::None,
            _ => panic!("impossible mouse button flags: {flags}"),
        }
    }

    #[inline]
    pub fn to_flags(&self) -> u32 {
        match self {
            Self::Left   => 0,
            Self::Middle => 1,
            Self::Right  => 2,
            Self::None   => 3,
        }
    }
}

impl std::fmt::Display for MouseButton {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Left   => "Left Mouse Button".fmt(f),
            Self::Middle => "Middle Mouse Button".fmt(f),
            Self::Right  => "Right Mouse Button".fmt(f),
            Self::None   => "None".fmt(f),
        }
    }
}

impl From<u32> for MouseButton {
    #[inline]
    fn from(value: u32) -> Self {
        Self::from_flags(value)
    }
}

impl From<MouseButton> for u32 {
    #[inline]
    fn from(value: MouseButton) -> Self {
        value.to_flags()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    CursorPosition { row: u32, column: u32 },
    WindowSize { rows: u32, columns: u32 },
    KeyPress { key: Key, ctrl: bool, alt: bool, shift: bool },
    MouseDown { row: u32, column: u32, shift: bool, ctrl: bool, alt: bool, button: MouseButton },
    MouseUp   { row: u32, column: u32, shift: bool, ctrl: bool, alt: bool, button: MouseButton },
    MouseMove { row: u32, column: u32, shift: bool, ctrl: bool, alt: bool, button: MouseButton },
    WheelUp   { row: u32, column: u32, shift: bool, ctrl: bool, alt: bool },
    WheelDown { row: u32, column: u32, shift: bool, ctrl: bool, alt: bool },
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

fn fmt_modifiers(shift: bool, alt: bool, ctrl: bool, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MouseState {
    Up,
    Down,
    Move,
}

impl std::fmt::Display for MouseState {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Up => "Up".fmt(f),
            Self::Down => "Down".fmt(f),
            Self::Move => "Move".fmt(f),
        }
    }
}

fn fmt_mouse(state: MouseState, row: u32, column: u32, shift: bool, alt: bool, ctrl: bool, button: MouseButton, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fmt_modifiers(shift, alt, ctrl, f)?;

    if button == MouseButton::None {
        write!(f, "Mouse {state}")?;
    } else {
        write!(f, "{button} {state}")?;
    }

    write!(f, " {column}x{row}")
}

fn fmt_wheel(state: MouseState, row: u32, column: u32, shift: bool, alt: bool, ctrl: bool, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fmt_modifiers(shift, alt, ctrl, f)?;

    write!(f, "Wheel {state} {column}x{row}")
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::KeyPress { ctrl, alt, shift, key } => {
                f.write_str("Key Press ")?;
                fmt_modifiers(shift, alt, ctrl, f)?;

                std::fmt::Display::fmt(&key, f)
            }
            Self::MouseMove { row, column, shift, ctrl, alt, button } => {
                fmt_mouse(MouseState::Move, row, column, shift, alt, ctrl, button, f)
            }
            Self::MouseDown { row, column, shift, ctrl, alt, button } => {
                fmt_mouse(MouseState::Down, row, column, shift, alt, ctrl, button, f)
            }
            Self::MouseUp { row, column, shift, ctrl, alt, button } => {
                fmt_mouse(MouseState::Up, row, column, shift, alt, ctrl, button, f)
            }
            Self::WheelDown { row, column, shift, ctrl, alt } => {
                fmt_wheel(MouseState::Down, row, column, shift, alt, ctrl, f)
            }
            Self::WheelUp { row, column, shift, ctrl, alt } => {
                fmt_wheel(MouseState::Up, row, column, shift, alt, ctrl, f)
            }
            Self::WindowSize { rows, columns } => {
                write!(f, "Window Size {columns}x{rows}")
            }
            Self::CursorPosition { row, column } => {
                write!(f, "Cursor Position {column}x{row}")
            }
            _ => {
                std::fmt::Debug::fmt(&self, f)
            }
        }
    }
}

pub const ESCAPE_EVENT: Event = Event::KeyPress { ctrl: false, alt: false, shift: false, key: Key::Escape };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    //PrintScreen, // not possible?
    //ScrollLock,  // not possible?
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
                if ch.is_ascii_control() {
                    if ch == '\x7F' {
                        f.write_char('\u{2421}')?;
                    } else {
                        f.write_char(unsafe { char::from_u32_unchecked(0x2400 + ch as u32) })?;
                    }
                } else {
                    let upper = ch.to_uppercase();
                    if upper.len() == 1 {
                        for ch in upper {
                            f.write_char(ch)?;
                        }
                    } else {
                        f.write_char(ch)?;
                    }
                }

                Ok(())
            },
            Self::Function(n) => write!(f, "F{n}"),
            _ => std::fmt::Debug::fmt(&self, f),
        }
    }
}
