use crate::{rect::Rect, rich_text::{DEFAULT_STYLE, RichText, right_pad_line, right_pad_line_with}, widget::{Widget, WidgetId, next_widget_id}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyList {
    draw_rect: Rect,
    widget_id: WidgetId,
    widths: (usize, usize),
    formatted_widths: (usize, usize),
    header: (String, String),
    rows: Vec<(String, String)>,
    formatted_headers: (RichText, RichText),
    formatted_header: RichText,
    formatted_pairs: Vec<(RichText, RichText)>,
    formatted_rows: Vec<RichText>,
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
            widths: (formatted_key.width(), formatted_value.width()),
            formatted_widths: (formatted_key.width(), formatted_value.width()),
            header: (key_header, value_header),
            rows: Vec::new(),
            formatted_headers: (formatted_key, formatted_value),
            formatted_header: RichText::new(),
            formatted_pairs: Vec::new(),
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
        self.preformat();
        self.postformat();
    }

    pub fn preformat(&mut self) {
        self.formatted_pairs.clear();

        let mut key_width = self.formatted_headers.0.width();
        let mut value_width = self.formatted_headers.1.width();

        for (key, value) in &self.rows {
            let rich_text_key = RichText::from_plain_text(key);
            let rich_text_value = RichText::from_plain_text(value);

            if rich_text_key.width() > key_width {
                key_width = rich_text_key.width();
            }

            if rich_text_value.width() > value_width {
                value_width = rich_text_value.width();
            }

            self.formatted_pairs.push((rich_text_key, rich_text_value));
        }

        self.widths = (key_width, value_width);
    }

    pub fn postformat(&mut self) {
        self.formatted_header.clear();
        self.formatted_rows.clear();

        let mut key_width = self.widths.0;
        let mut value_width = self.widths.1;

        if key_width + 1 + value_width > self.draw_rect.width as usize {
            let half_width = self.draw_rect.width as usize / 2;
            let half_width = if half_width + half_width < self.draw_rect.width as usize {
                half_width
            } else {
                half_width - 1
            };

            if key_width <= half_width {
                value_width = self.draw_rect.width as usize - 1 - key_width;
            } else if value_width <= half_width {
                key_width = self.draw_rect.width as usize - 1 - value_width;
            } else {
                key_width = half_width;
                value_width = self.draw_rect.width as usize - 1 - key_width;
            }
        }

        self.formatted_widths = (key_width, value_width);

        self.header_height = format_row(&self.formatted_headers.0, &self.formatted_headers.1, key_width, value_width, &mut self.formatted_header);

        let mut rows_height = 0;
        for (key, value) in &self.formatted_pairs {
            let formatted = self.formatted_rows.push_mut(RichText::new());
            rows_height += format_row(key, value, key_width, value_width, formatted);
        }

        self.rows_height = rows_height;
    }
}


impl Widget for PropertyList {
    #[inline]
    fn draw_rect(&self) -> &Rect {
        &self.draw_rect
    }

    #[inline]
    fn set_draw_rect(&mut self, rect: &Rect) {
        self.draw_rect = *rect;
        self.postformat();
    }

    #[inline]
    fn widget_id(&self) -> WidgetId {
        self.widget_id
    }

    fn draw(&self, termio: &mut crate::termio::TermIO, parent_row: i32, parent_column: i32) -> std::io::Result<()> {
        unimplemented!()
    }

    fn handle_event(&mut self, event: &crate::event::Event, broker: &mut crate::message::MessageBroker) -> crate::widget::ActionFlags {
        unimplemented!()
    }

    fn handle_message(&mut self, message: &mut crate::message::Message, broker: &mut crate::message::MessageBroker) -> crate::widget::ActionFlags {
        unimplemented!()
    }
}

fn format_row(key: &RichText, value: &RichText, key_width: usize, value_width: usize, formatted: &mut RichText) -> usize {
    let key = key.wrap(key_width);
    let value = value.wrap(value_width);

    let max_lines = key.height().max(value.height()).max(1);

    formatted.bottom_pad(max_lines);

    let mut key_style = DEFAULT_STYLE;
    let mut value_style = DEFAULT_STYLE;

    let row_width = key_width + 1 + value_width;

    for line_index in 0..max_lines {
        let self_line = if let Some(self_line) = formatted.lines.get_mut(line_index) {
            //self_style.apply_changes(self_line);
            self_line
        } else {
            formatted.lines.push_mut(Vec::new())
        };

        self_line.reserve(row_width);

        value_style.diff(&key_style, self_line);
        if let Some(key_line) = key.lines.get(line_index) {
            self_line.extend_from_slice(key_line);
            key_style.apply_changes(key_line);
        }

        right_pad_line(self_line, key_width + 1);

        key_style.diff(&value_style, self_line);
        if let Some(value_line) = key.lines.get(line_index) {
            self_line.extend_from_slice(value_line);
            value_style.apply_changes(value_line);
        }

        right_pad_line_with(self_line, key_width + 1, row_width);
    }

    formatted.width = row_width;
    formatted.height()
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
