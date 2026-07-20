use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{event::Event, message::{Message, MessageBroker, MessageReceiver}, rect::Rect, termio::TermIO};

static WIDGET_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WidgetId(usize);

pub fn next_widget_id() -> WidgetId {
    WidgetId(WIDGET_COUNTER.fetch_add(1, Ordering::Relaxed))
}

#[allow(unused)]
pub trait Widget: std::fmt::Debug {
    fn widget_id(&self) -> WidgetId;
    fn set_draw_rect(&mut self, rect: &Rect);
    fn draw_rect(&self) -> Rect;

    fn draw(&self, termio: &mut TermIO, parent_row: i32, parent_column: i32) -> std::io::Result<()>;

    fn handle_event(&mut self, event: &Event, broker: &mut MessageBroker) {}
    fn handle_message(&mut self, message: &mut Message, broker: &mut MessageBroker) {}
}

impl<W: Widget> MessageReceiver for W {
    #[inline]
    fn handle_message(&mut self, message: &mut Message, broker: &mut MessageBroker) {
        Widget::handle_message(self, message, broker);
    }
}
