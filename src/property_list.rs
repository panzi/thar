use crate::{char_width::{CharWidth, wcs_max_width_control_is_one}, plain_text::{PlainText, PlainTextItem, line_width}, rect::Rect, style::{FontWeight, ScopedTermIOState}, styles::{EVEN_ROW_BACKGROUND, ODD_ROW_BACKGROUND, SELECTED_EVEN_ROW_BACKGROUND, SELECTED_ODD_ROW_BACKGROUND, TABLE_FOREGROUND}, termio::TermIO, widget::{Widget, WidgetData, WidgetId}, wrap::wrap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyList {
    widget_data: WidgetData,
    widths: (usize, usize),
    formatted_widths: (usize, usize),
    header: (String, String),
    rows: Vec<(String, String)>,
    formatted_header: PlainText,
    formatted_rows: Vec<PlainText>,
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
            formatted_header: PlainText::new(),
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
        let mut key_width = wcs_max_width_control_is_one(&self.header.0);
        let mut value_width = wcs_max_width_control_is_one(&self.header.1);

        for (key, value) in &self.rows {
            let row_key_width = wcs_max_width_control_is_one(key);
            let row_value_width = wcs_max_width_control_is_one(value);

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
            let formatted = self.formatted_rows.push_mut(PlainText::new());
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

    fn draw(&mut self, termio: &mut TermIO, parent_row: i32, parent_column: i32) -> std::io::Result<()> {
        if self.widget_data.dirty {
            let Rect { row, column, width, height } = self.widget_data.rect;

            if width == 0 || height == 0 {
                self.widget_data.dirty = false;
                return Ok(());
            }

            let &mut PropertyList { selected_row_index, scroll_row, .. } = self;
            let row    = row    + parent_row;
            let column = column + parent_column;

            let mut scoped_state = ScopedTermIOState::default_bg(termio, ODD_ROW_BACKGROUND);
            let mut scoped_state = ScopedTermIOState::default_fg(scoped_state.termio_mut(), TABLE_FOREGROUND);

            {
                scoped_state.termio_mut().font_weight(FontWeight::Bold)?;

                let res = self.formatted_header.draw_cropped(
                    scoped_state.termio_mut(),
                    row,
                    column,
                    0,
                    0,
                    width,
                    height,
                );

                scoped_state.termio_mut().font_weight(FontWeight::Normal)?;

                res?;
            }

            let header_height = self.formatted_header.height();

            if (height as usize) < header_height {
                return Ok(());
            }

            let mut body_height = 0;
            let mut current_row_index = 0;

            while current_row_index < self.formatted_rows.len() {
                let row_height = self.formatted_rows[current_row_index].height();
                if body_height as i32 + row_height as i32 - scroll_row as i32 >= header_height as i32 {
                    break;
                }

                body_height += row_height;
                current_row_index += 1;
            }

            let mut avail_height = height - header_height as u32 + scroll_row;

            while current_row_index < self.formatted_rows.len() {
                let table_row = &self.formatted_rows[current_row_index];

                let mut scoped_state = ScopedTermIOState::default_bg(
                    scoped_state.termio_mut(),
                    if ((current_row_index + scroll_row as usize) & 1) == 0 {
                        if current_row_index == selected_row_index { SELECTED_EVEN_ROW_BACKGROUND } else { EVEN_ROW_BACKGROUND }
                    } else {
                        if current_row_index == selected_row_index { SELECTED_ODD_ROW_BACKGROUND } else { ODD_ROW_BACKGROUND }
                    }
                );

                let offset_body_height = body_height as i32 - scroll_row as i32;

                table_row.draw_cropped(
                    scoped_state.termio_mut(),
                    row + header_height as i32 + offset_body_height.max(0),
                    column,
                    -offset_body_height.min(0) as u32,
                    0,
                    width,
                    avail_height,
                )?;

                current_row_index += 1;

                let row_height = table_row.height();
                body_height += row_height;
                avail_height = if (avail_height as usize) > row_height {
                    avail_height - row_height as u32
                } else {
                    break;
                };
            }

            if body_height < avail_height as usize {
                let offset_body_height = body_height as i32 - scroll_row as i32;
                let line_row = (row + header_height as i32 + offset_body_height.max(0)) as u32;
                let line_column;
                let line_width;

                if column < 0 {
                    line_column = 0;
                    line_width = width - (-column) as u32;
                } else {
                    line_column = column as u32;
                    line_width = width;
                }

                let termio = scoped_state.termio_mut();
                let window_width = termio.window_size().columns;

                if line_column < window_width {
                    let line_width = if line_column + line_width > window_width {
                        window_width - line_column
                    } else {
                        line_width
                    };
                    let repeat_count = line_width - 1;

                    for line_index in 0..((avail_height as usize - body_height) as u32) {
                        if line_index == 0 || line_column != 0 {
                            termio.move_cursor(line_row + line_index, column as u32)?;
                        } else {
                            termio.write(b"\n")?;
                        }

                        termio.write(b" ")?;
                        termio.repeat(repeat_count)?;
                    }
                }
            }

            self.set_dirty(false);
        }

        Ok(())
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
    line_index: usize,
    item_index: usize,
    char_index: usize,
}

fn append_items(items: &mut Vec<PlainTextItem>, text: &str) -> usize {
    let mut width = 0;
    let mut prev_index = 0;
    let mut prev_width = 0;

    for (index, ch) in text.char_indices() {
        if ch.is_ascii_control() {
            if prev_index < index {
                if let Some(PlainTextItem::Text { text: item_text, width: item_width }) = items.last_mut() {
                    item_text.push_str(&text[prev_index..index]);
                    *item_width += width - prev_width;
                } else {
                    items.push(PlainTextItem::Text {
                        text: text[prev_index..index].to_string(),
                        width: width - prev_width,
                    });
                }
            }

            items.push(PlainTextItem::Special(ch));

            width += 1;
            prev_width = width;
            prev_index = index + 1;
        } else {
            width += ch.char_width_ignore_unprintable();
        }
    }

    if prev_index < text.len() {
        if let Some(PlainTextItem::Text { text: item_text, width: item_width }) = items.last_mut() {
            item_text.push_str(&text[prev_index..]);
            *item_width += width - prev_width;
        } else {
            items.push(PlainTextItem::Text {
                text: text[prev_index..].to_string(),
                width: width - prev_width,
            });
        }
    }

    width
}

// TODO: draw cursor when editing
fn format_row(key: &str, value: &str, key_width: usize, value_width: usize, formatted: &mut PlainText) -> usize {
    let key = wrap(key, key_width).collect::<Vec<_>>();
    let value = wrap(value, value_width).collect::<Vec<_>>();

    let max_lines = key.len().max(value.len()).max(1);

    formatted.bottom_pad(max_lines);

    let value_start = key_width + 1;
    let row_width = value_start + value_width;

    for line_index in 0..max_lines {
        let self_line = if let Some(self_line) = formatted.lines.get_mut(line_index) {
            self_line
        } else {
            formatted.lines.push_mut(Vec::new())
        };

        let mut line_width = line_width(self_line);

        if let Some(&key_line) = key.get(line_index) {
            line_width += append_items(self_line, key_line);
        }

        if value_start > line_width {
            let diff = value_start - line_width;
            if let Some(PlainTextItem::Text { text, width }) = self_line.last_mut() {
                text.reserve(diff);
                for _ in 0..diff {
                    text.push(' ');
                }
                *width += diff;
            } else {
                let text = " ".repeat(diff);
                self_line.push(PlainTextItem::Text { text, width: diff })
            }
            line_width = value_start;
        }

        if let Some(value_line) = value.get(line_index) {
            line_width += append_items(self_line, value_line);
        }

        if row_width > line_width {
            let diff = row_width - line_width;
            if let Some(PlainTextItem::Text { text, width }) = self_line.last_mut() {
                text.reserve(diff);
                for _ in 0..diff {
                    text.push(' ');
                }
                *width += diff;
            } else {
                let text = " ".repeat(diff);
                self_line.push(PlainTextItem::Text { text, width: diff })
            }
            line_width = row_width;
        }
    }

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
