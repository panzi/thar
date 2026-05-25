use crate::{event::Event, termio::{TermIO, WindowSize}};

pub mod table;

pub trait Widget {
    fn draw(&self, termio: &mut TermIO, draw_state: &DrawState) -> std::io::Result<()>;
    fn handle_event(&mut self, termio: &mut TermIO, draw_state: &DrawState, event: &Event) -> std::io::Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub row:    i32,
    pub column: i32,
    pub width:  u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawState {
    pub window_size: WindowSize,
    pub rect: Rect,
    pub focus: bool,
}
