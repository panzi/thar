use std::io::Write;

use crate::{event::{Event, Key}, message::MessageBroker, plain_text::{Cursor, PlainText, line_len}, rect::Rect, termio::TermIO, widget::{ActionFlags, Widget, WidgetData, WidgetId}};

#[derive(Debug)]
pub struct TextEditor {
    widget_data: WidgetData,
    text: PlainText,
    scroll_row: u32,
    scroll_column: u32,
    cursor: Cursor,
}

impl TextEditor {
    #[inline]
    pub fn new() -> Self {
        Self {
            widget_data: WidgetData::new(),
            text: PlainText::new(),
            scroll_row: 0,
            scroll_column: 0,
            cursor: Cursor::default(),
        }
    }

    #[inline]
    pub fn with_text(text: &str) -> Self {
        let text = PlainText::parse(text);
        let cursor_line_index = text.height().saturating_sub(1);
        let cursor_byte_index = if text.height() > 0 {
            line_len(&text.lines()[cursor_line_index])
        } else {
            0
        };

        Self {
            widget_data: WidgetData::new(),
            text,
            scroll_row: 0,
            scroll_column: 0,
            cursor: Cursor {
                line_index: cursor_line_index,
                byte_index: cursor_byte_index,
            }
        }
    }

    #[inline]
    pub fn write_to(&self, write: &mut impl Write) -> std::io::Result<()> {
        self.text.write(write)
    }

    fn clamp_scroll_row(&mut self) {
        let height = self.text.height();
        if self.widget_data.rect.height as usize > height {
            self.scroll_row = 0;
        } else {
            let max_overflow = (height - self.widget_data.rect.height as usize) as u32;

            if self.scroll_row > max_overflow {
                self.scroll_row = max_overflow;
            }
        }
    }

    fn clamp_scroll_column(&mut self) {
        if self.widget_data.rect.width as usize > self.text.width() {
            self.scroll_column = 0;
        } else {
            let max_overflow = (self.text.width() - self.widget_data.rect.width as usize) as u32;

            if self.scroll_column > max_overflow {
                self.scroll_column = max_overflow;
            }
        }
    }

    fn clamp_cursor(&mut self) {
        let height = self.text.height();

        if height == 0 {
            self.cursor.line_index = 0;
            self.cursor.byte_index = 0;
        } else {
            if self.cursor.line_index >= height {
                self.cursor.line_index = height - 1;
            }

            let len = line_len(&self.text.lines()[self.cursor.line_index]);
            if self.cursor.byte_index > len {
                self.cursor.byte_index = len;
            }
        }
    }

    pub fn update(&mut self) {
        // XXX: Very inefficient to reallocate everything on every keypress.
        self.text = self.text.wrap(self.widget_data.rect.width as usize);
    }

    fn insert_at_cursor(&mut self, text: &str) {
        
    }
}

impl Widget for TextEditor {
    #[inline]
    fn widget_id(&self) -> WidgetId {
        self.widget_data.widget_id
    }

    #[inline]
    fn draw_rect(&self) -> &Rect {
        &self.widget_data.rect
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
    fn set_draw_rect(&mut self, rect: &Rect) {
        if self.widget_data.rect != *rect {
            self.widget_data.rect = *rect;
            self.widget_data.dirty = true;
            self.update();
            self.clamp_scroll_row();
            self.clamp_scroll_column();
            self.clamp_cursor();
        }
    }

    fn draw(&mut self, termio: &mut TermIO, parent_row: i32, parent_column: i32) -> std::io::Result<()> {
        let row = self.widget_data.rect.row + parent_row;
        let column = self.widget_data.rect.column + parent_column;
        self.text.draw_cropped(
            termio,
            row, column,
            self.scroll_row,
            self.scroll_column,
            self.widget_data.rect.width,
            self.widget_data.rect.height,
            &Some(Cursor {
                line_index: self.cursor.line_index,
                byte_index: self.cursor.byte_index,
            })
        )
    }

    fn handle_event(&mut self, event: &Event, _broker: &mut MessageBroker) -> ActionFlags {
        match event {
            Event::KeyPress { key: Key::Left, ctrl: false, alt: false, shift: false } => {
                // TODO
            }
            Event::KeyPress { key: Key::Right, ctrl: false, alt: false, shift: false } => {
                // TODO
            }
            Event::KeyPress { key: Key::Up, ctrl: false, alt: false, shift: false } => {
                // TODO
            }
            Event::KeyPress { key: Key::Down, ctrl: false, alt: false, shift: false } => {
                // TODO
            }
            Event::KeyPress { key: Key::Home, ctrl, alt: false, shift: false } => {
                // TODO
            }
            Event::KeyPress { key: Key::End, ctrl, alt: false, shift: false } => {
                // TODO
            }
            Event::KeyPress { key: Key::Backspace, ctrl: false, alt: false, shift: false } => {
                self.cursor = self.text.backspace(&self.cursor);
                self.update();
                self.widget_data.dirty = true;
                return ActionFlags::Dirty;
            }
            Event::KeyPress { key: Key::Delete, ctrl: false, alt: false, shift: false } => {
                self.cursor = self.text.delete(&self.cursor);
                self.update();
                self.widget_data.dirty = true;
                return ActionFlags::Dirty;
            }
            Event::KeyPress { key: Key::Enter, ctrl: false, alt: false, shift: false } => {
                self.insert_at_cursor("\n");
                self.widget_data.dirty = true;
                return ActionFlags::Dirty;
            }
            Event::KeyPress { key: Key::Char(ch), ctrl: false, alt: false, shift: false } => {
                self.insert_at_cursor(ch.encode_utf8(&mut [0; char::MAX_LEN_UTF8]));
                self.update();
                self.widget_data.dirty = true;
                return ActionFlags::Dirty;
            }
            _ => {}
        }

        ActionFlags::None
    }
}
