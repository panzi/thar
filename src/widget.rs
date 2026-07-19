use crate::{event::Event, rect::Rect, termio::TermIO};

pub trait Widget {
    fn set_draw_rect(&mut self, rect: &Rect);
    fn draw_rect(&self) -> Rect;

    fn draw(&self, termio: &mut TermIO, parent_row: i32, parent_column: i32) -> std::io::Result<()>;
    fn handle_event(&mut self, event: &Event);
}
