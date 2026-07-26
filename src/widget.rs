use std::sync::atomic::{AtomicUsize, Ordering};

use bitflags::bitflags;

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
    fn draw_rect(&self) -> &Rect;
    fn is_dirty(&self) -> bool;
    fn set_dirty(&mut self, dirty: bool);

    fn draw(&mut self, termio: &mut TermIO, parent_row: i32, parent_column: i32) -> std::io::Result<()>;

    fn handle_event(&mut self, event: &Event, broker: &mut MessageBroker) -> ActionFlags { ActionFlags::None }
    fn handle_message(&mut self, message: &mut Message, broker: &mut MessageBroker) -> ActionFlags { ActionFlags::None }
}

impl<W: Widget> MessageReceiver for W {
    #[inline]
    fn handle_message(&mut self, message: &mut Message, broker: &mut MessageBroker) -> ActionFlags {
        Widget::handle_message(self, message, broker)
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct ActionFlags: u32 {
        const None  = 0;
        const Dirty = 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WidgetData {
    pub rect: Rect,
    pub widget_id: WidgetId,
    pub dirty: bool,
}

impl WidgetData {
    #[inline]
    pub fn new() -> Self {
        Self {
            rect: Rect::default(),
            widget_id: next_widget_id(),
            dirty: true,
        }
    }
}

impl Default for WidgetData {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
