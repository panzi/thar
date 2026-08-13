use std::io::Write;

use crate::{event::{Event, Key}, message::MessageBroker, plain_text::{Cursor, PlainText, PlainTextItem, line_width}, rect::Rect, termio::TermIO, widget::{ActionFlags, Widget, WidgetData, WidgetId}};

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
        let cursor_row = text.height().saturating_sub(1) as u32;
        let cursor_column = if text.height() > 0 {
            line_width(&text.lines()[cursor_row as usize]) as u32
        } else {
            0
        };

        Self {
            widget_data: WidgetData::new(),
            text,
            scroll_row: 0,
            scroll_column: 0,
            cursor: Cursor {
                row: cursor_row,
                column: cursor_column,
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
            self.cursor.row = 0;
            self.cursor.column = 0;
        } else {
            if self.cursor.row as usize >= height {
                self.cursor.row = (height - 1) as u32;
            }

            let width = line_width(&self.text.lines()[self.cursor.row as usize]);
            if self.cursor.column as usize > width {
                self.cursor.column = width as u32;
            }
        }
    }

    pub fn update(&mut self) {
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
                row: self.cursor.row,
                column: self.cursor.column
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
