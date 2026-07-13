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
    pub fn parse(value: &str) -> Option<Self> {
        if value.eq_ignore_ascii_case("black") {
            Some(Color16::Black)
        } else if value.eq_ignore_ascii_case("red") {
            Some(Color16::Red)
        } else if value.eq_ignore_ascii_case("green") {
            Some(Color16::Green)
        } else if value.eq_ignore_ascii_case("yellow") {
            Some(Color16::Yellow)
        } else if value.eq_ignore_ascii_case("blue") {
            Some(Color16::Blue)
        } else if value.eq_ignore_ascii_case("magenta") {
            Some(Color16::Magenta)
        } else if value.eq_ignore_ascii_case("cyan") {
            Some(Color16::Cyan)
        } else if value.eq_ignore_ascii_case("white") {
            Some(Color16::White)
        } else if value.eq_ignore_ascii_case("brightblack") || value.eq_ignore_ascii_case("grey") || value.eq_ignore_ascii_case("gray") {
            Some(Color16::BrightBlack)
        } else if value.eq_ignore_ascii_case("brightred") {
            Some(Color16::BrightRed)
        } else if value.eq_ignore_ascii_case("brightgreen") {
            Some(Color16::BrightGreen)
        } else if value.eq_ignore_ascii_case("brightyellow") {
            Some(Color16::BrightYellow)
        } else if value.eq_ignore_ascii_case("brightblue") {
            Some(Color16::BrightBlue)
        } else if value.eq_ignore_ascii_case("brightmagenta") {
            Some(Color16::BrightMagenta)
        } else if value.eq_ignore_ascii_case("brightcyan") {
            Some(Color16::BrightCyan)
        } else if value.eq_ignore_ascii_case("brightwhite") {
            Some(Color16::BrightWhite)
        } else {
            None
        }
    }

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

    #[inline]
    pub fn to_color(&self) -> Color {
        Color::Color16(*self)
    }

    #[inline]
    pub fn into_color(self) -> Color {
        Color::Color16(self)
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

    #[inline]
    pub fn to_color(&self) -> Color {
        Color::Rgb { r: self.r, g: self.g, b: self. b }
    }

    #[inline]
    pub fn into_color(self) -> Color {
        Color::Rgb { r: self.r, g: self.g, b: self. b }
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
