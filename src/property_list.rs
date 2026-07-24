use crate::{rect::Rect, rich_text::RichText, widget::{WidgetId, next_widget_id}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyList {
    draw_rect: Rect,
    widget_id: WidgetId,
    max_widths: (usize, usize),
    header: (String, String),
    rows: Vec<(String, String)>,
    formatted_headers: (RichText, RichText),
    formatted_header: RichText,
    formatted_rows: Vec<(RichText, RichText)>,
    header_height: usize,
    rows_height: usize,
    selected_row_index: usize,

    // not sure about these. how will editing work?
    scroll_row: u32,
    scroll_column: u32,

    editable: bool,
}

impl PropertyList {
    #[inline]
    pub fn new(key_header: String, value_header: String) -> Self {
        let formatted_key = RichText::from_plain_text(&key_header);
        let formatted_value = RichText::from_plain_text(&value_header);
        let header_height = formatted_key.height().max(formatted_value.height());

        Self {
            draw_rect: Rect::default(),
            widget_id: next_widget_id(),
            max_widths: (formatted_key.width(), formatted_value.width()),
            header: (key_header, value_header),
            rows: Vec::new(),
            formatted_headers: (formatted_key, formatted_value),
            formatted_header: RichText::new(),
            formatted_rows: Vec::new(),
            header_height,
            rows_height: 0,
            selected_row_index: 0,
            scroll_row: 0,
            scroll_column: 0,
            editable: false,
        }
    }

    #[inline]
    pub fn header_height(&self) -> usize {
        self.header_height
    }

    #[inline]
    pub fn rows_height(&self) -> usize {
        self.rows_height
    }

    #[inline]
    pub fn update(&mut self) {
        self.formatted_header.clear();
        self.formatted_rows.clear();

        let mut max_key_width = self.formatted_headers.0.width();
        let mut max_value_width = self.formatted_headers.1.width();

        for row in &self.rows {
            // TODO
        }

    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateKey {
    pub widget_id: WidgetId,
    pub row_index: usize,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateValue {
    pub widget_id: WidgetId,
    pub row_index: usize,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertProperty {
    pub widget_id: WidgetId,
    pub row_index: usize,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteProperty {
    pub widget_id: WidgetId,
    pub row_index: usize,
}
