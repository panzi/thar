use crate::{rect::Rect, tabs::Tabs, widget::WidgetId};

#[derive(Debug)]
pub struct RequestView {
    tabs: Tabs,
    draw_rect: Rect,
    widget_id: WidgetId,
}
