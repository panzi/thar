#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputSequence {
    Char(char),
    Alt(char),
    Modifier { modifier: u32, char: char },
    KeyCode { key_code: u32, modifier: u32 },
    CursorPos { row: u32, column: u32 },
    WindowSize { rows: u32, columns: u32 },
    SingleShift3(char),
}

pub const ESCAPE: u8 = 0x1B;

