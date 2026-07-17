use crate::{event::Event, termio::TermIO};

pub trait Widget {
    fn set_draw_rect(&mut self, rect: &Rect);
    fn draw_rect(&self) -> Rect;

    fn draw(&self, termio: &mut TermIO) -> std::io::Result<()>;
    fn handle_event(&mut self, event: &Event);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub row: i32,
    pub column: i32,
    pub width: u32,
    pub height: u32,
}
