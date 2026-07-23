use crate::{rect::Rect, tabs::Tabs, widget::WidgetId};

#[derive(Debug)]
pub struct RequestView {
    tabs: Tabs,
    draw_rect: Rect,
    widget_id: WidgetId,

    // tabs
    overview_id: WidgetId,
    request_headers_id: WidgetId,
    response_headers_id: WidgetId,
    cookies_id: WidgetId,
    request_body_id: WidgetId,
    response_body_id: WidgetId,
    timings_id: WidgetId,
    commet_id: WidgetId,
}
