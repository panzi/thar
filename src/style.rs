use crate::{color::Color, termio::TermIO};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Normal,
    Bold,
    Faint,
}

impl Default for FontWeight {
    #[inline]
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDecoration {
    None,
    Underline,
    DoublyUnderline,
}

impl Default for TextDecoration {
    #[inline]
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Normal,
    Italic,
}

impl Default for FontStyle {
    #[inline]
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermIOState {
    None,
    DefaultForeground(Color),
    DefaultBackground(Color),
    Inverted(bool),
}

impl TermIOState {
    pub fn apply(&self, termio: &mut TermIO) {
        match self {
            Self::None => {},
            Self::DefaultForeground(color) => termio.set_default_fg(*color),
            Self::DefaultBackground(color) => termio.set_default_bg(*color),
            Self::Inverted(inverted) => termio.set_inverted(*inverted),
        }
    }
}

#[derive(Debug)]
pub struct ScopedTermIOState<'a> {
    termio: &'a mut TermIO,
    old_state: TermIOState,
}

impl<'a> ScopedTermIOState<'a> {
    #[inline]
    pub fn none(termio: &'a mut TermIO) -> Self {
        Self { termio, old_state: TermIOState::None }
    }

    #[inline]
    pub fn default_fg(termio: &'a mut TermIO, color: Color) -> Self {
        let old_state = TermIOState::DefaultForeground(termio.default_fg());
        termio.set_default_fg(color);
        Self { termio, old_state }
    }

    #[inline]
    pub fn default_bg(termio: &'a mut TermIO, color: Color) -> Self {
        let old_state = TermIOState::DefaultBackground(termio.default_bg());
        termio.set_default_bg(color);
        Self { termio, old_state }
    }

    #[inline]
    pub fn invert(termio: &'a mut TermIO) -> Self {
        let inverted = termio.inverted();
        let old_state = TermIOState::Inverted(inverted);
        termio.set_inverted(!inverted);
        Self { termio, old_state }
    }

    #[inline]
    pub fn inverted(termio: &'a mut TermIO, inverted: bool) -> Self {
        let old_state = TermIOState::Inverted(termio.inverted());
        termio.set_inverted(inverted);
        Self { termio, old_state }
    }

    #[inline]
    pub fn new(termio: &'a mut TermIO, style: TermIOState) -> Self {
        match style {
            TermIOState::None => {
                Self::none(termio)
            },
            TermIOState::DefaultForeground(color) => {
                Self::default_fg(termio, color)
            },
            TermIOState::DefaultBackground(color) => {
                Self::default_bg(termio, color)
            },
            TermIOState::Inverted(inverted) => {
                Self::inverted(termio, inverted)
            },
        }
    }

    #[inline]
    pub fn termio(&self) -> &TermIO {
        self.termio
    }

    #[inline]
    pub fn termio_mut(&mut self) -> &mut TermIO {
        self.termio
    }

    #[inline]
    pub fn old_state(&self) -> TermIOState {
        self.old_state
    }
}

impl Drop for ScopedTermIOState<'_> {
    fn drop(&mut self) {
        self.old_state.apply(self.termio);
    }
}
