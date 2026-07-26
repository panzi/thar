use crate::{char_width::{CharWidth, wcs_max_width}, rect::Rect, widget::{Widget, WidgetData, WidgetId}, wrap::wrap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyList {
    widget_data: WidgetData,
    widths: (usize, usize),
    formatted_widths: (usize, usize),
    header: (String, String),
    rows: Vec<(String, String)>,
    formatted_header: Vec<String>,
    formatted_rows: Vec<Vec<String>>,
    header_height: usize,
    rows_height: usize,
    selected_row_index: usize,

    // not sure about this. how will editing work?
    scroll_row: u32,

    editable: bool,
    edit_state: Option<EditState>,
}

impl PropertyList {
    #[inline]
    pub fn new(key_header: String, value_header: String) -> Self {
        Self {
            widget_data: WidgetData::new(),
            widths: (0, 0),
            formatted_widths: (0, 0),
            header: (key_header, value_header),
            rows: Vec::new(),
            formatted_header: Vec::new(),
            formatted_rows: Vec::new(),
            header_height: 0,
            rows_height: 0,
            selected_row_index: 0,
            scroll_row: 0,
            editable: false,
            edit_state: None,
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
        let mut key_width = wcs_max_width(&self.header.0);
        let mut value_width = wcs_max_width(&self.header.1);

        for (key, value) in &self.rows {
            let row_key_width = wcs_max_width(key);
            let row_value_width = wcs_max_width(value);

            if row_key_width > key_width {
                key_width = row_key_width;
            }

            if row_value_width > value_width {
                value_width = row_value_width;
            }
        }

        self.widths = (key_width, value_width);
    }

    pub fn postformat(&mut self) {
        self.formatted_header.clear();
        self.formatted_rows.clear();

        let mut key_width = self.widths.0;
        let mut value_width = self.widths.1;

        if key_width + 1 + value_width > self.widget_data.rect.width as usize {
            let half_width = self.widget_data.rect.width as usize / 2;
            let half_width = if half_width + half_width < self.widget_data.rect.width as usize {
                half_width
            } else {
                half_width - 1
            };

            if key_width <= half_width {
                value_width = self.widget_data.rect.width as usize - 1 - key_width;
            } else if value_width <= half_width {
                key_width = self.widget_data.rect.width as usize - 1 - value_width;
            } else {
                key_width = half_width;
                value_width = self.widget_data.rect.width as usize - 1 - key_width;
            }
        }

        self.formatted_widths = (key_width, value_width);

        self.header_height = format_row(&self.header.0, &self.header.1, key_width, value_width, &mut self.formatted_header);

        let mut rows_height = 0;
        for (key, value) in &self.rows {
            let formatted = self.formatted_rows.push_mut(Vec::new());
            rows_height += format_row(key, value, key_width, value_width, formatted);
        }

        self.rows_height = rows_height;
    }
}


impl Widget for PropertyList {
    #[inline]
    fn draw_rect(&self) -> &Rect {
        &self.widget_data.rect
    }

    #[inline]
    fn set_draw_rect(&mut self, rect: &Rect) {
        if self.widget_data.rect != *rect {
            self.widget_data.rect = *rect;
            self.widget_data.dirty = true;
            self.postformat();
        }
    }

    #[inline]
    fn is_dirty(&self) -> bool {
        self.widget_data.dirty
    }

    #[inline]
    fn set_dirty(&mut self, dirty: bool) {
        self.widget_data.dirty = dirty;
    }

    #[inline]
    fn widget_id(&self) -> WidgetId {
        self.widget_data.widget_id
    }

    fn draw(&mut self, termio: &mut crate::termio::TermIO, parent_row: i32, parent_column: i32) -> std::io::Result<()> {
        unimplemented!()
    }

    fn handle_event(&mut self, event: &crate::event::Event, broker: &mut crate::message::MessageBroker) -> crate::widget::ActionFlags {
        unimplemented!()
    }

    fn handle_message(&mut self, message: &mut crate::message::Message, broker: &mut crate::message::MessageBroker) -> crate::widget::ActionFlags {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Column {
    Key,
    Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EditState {
    column: Column,
    cursor: Location,
    row_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Location {
    lineno: usize,
    column: usize,
}

// TODO: draw cursor when editing
fn format_row(key: &str, value: &str, key_width: usize, value_width: usize, formatted: &mut Vec<String>) -> usize {
    let key = wrap(key, key_width).collect::<Vec<_>>();
    let value = wrap(value, value_width).collect::<Vec<_>>();

    let max_lines = key.len().max(value.len()).max(1);

    formatted.resize_with(max_lines, String::new);

    let value_start = key_width + 1;
    let row_width = value_start + value_width;

    for line_index in 0..max_lines {
        let self_line = if let Some(self_line) = formatted.get_mut(line_index) {
            self_line
        } else {
            formatted.push_mut(String::with_capacity(row_width))
        };

        let mut line_width = self_line.char_width_ignore_unprintable();
        if line_width < row_width {
            self_line.reserve(row_width - line_width);
        }

        if let Some(key_line) = key.get(line_index) {
            self_line.push_str(key_line);
            line_width += key_line.char_width_ignore_unprintable();
        }

        if value_start > line_width {
            let diff = value_start - line_width;
            self_line.reserve(diff);
            for _ in 0..diff {
                self_line.push(' ');
            }
            line_width = value_start;
        }

        if let Some(value_line) = value.get(line_index) {
            self_line.push_str(value_line);
            line_width += value_line.char_width_ignore_unprintable();
        }

        if row_width > line_width {
            let diff = row_width - line_width;
            self_line.reserve(diff);
            for _ in 0..diff {
                self_line.push(' ');
            }
            line_width = value_start;
        }
    }

    formatted.len()
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
