use crate::{rect::Rect, rich_text::RichText, widget::WidgetId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyList {
    draw_rect: Rect,
    widget_id: WidgetId,
    widths: (usize, usize),
    header: (String, String),
    rows: Vec<(String, String)>,
    formatted_header: RichText,
    formatted_rows: Vec<RichText>,
    width: usize,
    header_height: usize,
    rows_height: usize,
    selected_row_index: usize,
    scroll_row: u32,
    scroll_column: u32,
    editable: bool,
}
