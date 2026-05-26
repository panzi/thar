#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color16 {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl Color16 {
    #[inline]
    pub fn fg(&self) -> &'static [u8] {
        match *self {
            Self::Black         => b"\x1B[30m",
            Self::Red           => b"\x1B[31m",
            Self::Green         => b"\x1B[32m",
            Self::Yellow        => b"\x1B[33m",
            Self::Blue          => b"\x1B[34m",
            Self::Magenta       => b"\x1B[35m",
            Self::Cyan          => b"\x1B[36m",
            Self::White         => b"\x1B[37m",
            Self::BrightBlack   => b"\x1B[90m",
            Self::BrightRed     => b"\x1B[91m",
            Self::BrightGreen   => b"\x1B[92m",
            Self::BrightYellow  => b"\x1B[93m",
            Self::BrightBlue    => b"\x1B[94m",
            Self::BrightMagenta => b"\x1B[95m",
            Self::BrightCyan    => b"\x1B[96m",
            Self::BrightWhite   => b"\x1B[97m",
        }
    }

    #[inline]
    pub fn bg(&self) -> &'static [u8] {
        match *self {
            Self::Black         => b"\x1B[40m",
            Self::Red           => b"\x1B[41m",
            Self::Green         => b"\x1B[42m",
            Self::Yellow        => b"\x1B[43m",
            Self::Blue          => b"\x1B[44m",
            Self::Magenta       => b"\x1B[45m",
            Self::Cyan          => b"\x1B[46m",
            Self::White         => b"\x1B[47m",
            Self::BrightBlack   => b"\x1B[100m",
            Self::BrightRed     => b"\x1B[101m",
            Self::BrightGreen   => b"\x1B[102m",
            Self::BrightYellow  => b"\x1B[103m",
            Self::BrightBlue    => b"\x1B[104m",
            Self::BrightMagenta => b"\x1B[105m",
            Self::BrightCyan    => b"\x1B[106m",
            Self::BrightWhite   => b"\x1B[107m",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    #[inline]
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    #[inline]
    pub fn from_u32(color: u32) -> Self {
        Self {
            r: ((color >> 16) & 0xFF) as u8,
            g: ((color >>  0) & 0xFF) as u8,
            b: (color         & 0xFF) as u8,
        }
    }

    #[inline]
    pub fn to_u32(&self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

impl From<u32> for Rgb {
    #[inline]
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<Rgb> for u32 {
    #[inline]
    fn from(value: Rgb) -> Self {
        value.to_u32()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Default,
    Rgb { r: u8, g: u8, b: u8 },
    Color16(Color16),
}

impl Color {
    #[inline]
    pub fn from_u32(color: u32) -> Self {
        Self::Rgb {
            r: ((color >> 16) & 0xFF) as u8,
            g: ((color >>  0) & 0xFF) as u8,
            b: (color         & 0xFF) as u8,
        }
    }
}

impl Default for Color {
    #[inline]
    fn default() -> Self {
        Self::Default
    }
}

impl From<Rgb> for Color {
    #[inline]
    fn from(Rgb { r, g, b }: Rgb) -> Self {
        Self::Rgb { r, g, b }
    }
}

impl From<Color16> for Color {
    #[inline]
    fn from(color16: Color16) -> Self {
        Self::Color16(color16)
    }
}
