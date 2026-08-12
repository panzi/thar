use crate::{event::{Event, Key}, message::MessageBroker, rect::Rect, rich_text::{ParseError, RichText}, termio::TermIO, widget::{ActionFlags, Widget, WidgetData, WidgetId}};

#[derive(Debug)]
pub struct RichTextView {
    widget_data: WidgetData,
    rich_text: RichText,
    scroll_row: u32,
    scroll_column: u32,
}

impl RichTextView {
    #[inline]
    pub fn new(rich_text: RichText) -> Self {
        Self {
            widget_data: WidgetData::new(),
            rich_text,
            scroll_row: 0,
            scroll_column: 0,
        }
    }

    #[inline]
    pub fn from_plain_text(plain_text: &str) -> Self {
        Self {
            widget_data: WidgetData::new(),
            rich_text: RichText::from_plain_text(plain_text),
            scroll_row: 0,
            scroll_column: 0,
        }
    }

    #[inline]
    pub fn from_rich_text(rich_text: &str) -> Result<Self, ParseError> {
        Ok(Self {
            widget_data: WidgetData::new(),
            rich_text: RichText::parse(rich_text)?,
            scroll_row: 0,
            scroll_column: 0,
        })
    }

    #[inline]
    pub fn scroll_row(&self) -> u32 {
        self.scroll_row
    }

    #[inline]
    pub fn scroll_column(&self) -> u32 {
        self.scroll_column
    }

    #[inline]
    pub fn rich_text(&self) -> &RichText {
        &self.rich_text
    }

    pub fn clamp_scroll_row(&mut self) {
        let height = self.rich_text.height();
        if self.widget_data.rect.height as usize > height {
            self.scroll_row = 0;
        } else {
            let max_overflow = (height - self.widget_data.rect.height as usize) as u32;

            if self.scroll_row > max_overflow {
                self.scroll_row = max_overflow;
            }
        }
    }

    pub fn clamp_scroll_column(&mut self) {
        if self.widget_data.rect.width as usize > self.rich_text.width() {
            self.scroll_column = 0;
        } else {
            let max_overflow = (self.rich_text.width() - self.widget_data.rect.width as usize) as u32;

            if self.scroll_column > max_overflow {
                self.scroll_column = max_overflow;
            }
        }
    }
}

impl Widget for RichTextView {
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
            self.clamp_scroll_row();
            self.clamp_scroll_column();
        }
    }

    fn draw(&mut self, termio: &mut TermIO, parent_row: i32, parent_column: i32) -> std::io::Result<()> {
        if self.widget_data.dirty {
            let Rect { row, column, width, height } = self.widget_data.rect;

            if width == 0 || height == 0 {
                self.widget_data.dirty = false;
                return Ok(());
            }

            let &mut Self { scroll_row, scroll_column, .. } = self;
            let row    = row    + parent_row;
            let column = column + parent_column;

            self.rich_text.draw_cropped(
                termio,
                row, column,
                scroll_row, scroll_column,
                width, height,
            )?;

            if height as usize > self.rich_text.height() {
                let offset_body_height = self.rich_text.height() as i32 - scroll_row as i32;
                let line_row = (row + offset_body_height.max(0)) as u32;
                let line_column;
                let line_width;

                if column < 0 {
                    line_column = 0;
                    line_width = width - (-column) as u32;
                } else {
                    line_column = column as u32;
                    line_width = width;
                }

                let window_width = termio.window_size().columns;

                termio.fg_default()?;
                termio.bg_default()?;

                if line_column < window_width {
                    let line_width = if line_column + line_width > window_width {
                        window_width - line_column
                    } else {
                        line_width
                    };
                    let repeat_count = line_width - 1;

                    for line_index in 0..(height - self.rich_text.height() as u32) {
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
        }

        Ok(())
    }

    fn handle_event(&mut self, event: &Event, _broker: &mut MessageBroker) -> ActionFlags {
        match event {
            &Event::KeyPress { key: Key::Up, alt: false, ctrl: false, shift: false } => {
                if self.scroll_row > 0 {
                    self.scroll_row -= 1;
                    self.widget_data.dirty = true;
                    return ActionFlags::Dirty;
                }
            }
            Event::KeyPress { key: Key::Home, alt: false, ctrl: false, shift: false } => {
                if self.scroll_row > 0 {
                    self.scroll_row = 0;
                    self.widget_data.dirty = true;
                    return ActionFlags::Dirty;
                }
            }
            &Event::KeyPress { key: Key::Down, alt: false, ctrl: false, shift: false } => {
                if self.scroll_row < u32::MAX {
                    let scroll_row = self.scroll_row;
                    self.scroll_row += 1;
                    self.clamp_scroll_row();
                    if self.scroll_row != scroll_row {
                        self.widget_data.dirty = true;
                        return ActionFlags::Dirty;
                    }
                }
            }
            Event::KeyPress { key: Key::End, alt: false, ctrl: false, shift: false } => {
                let scroll_row = (self.rich_text.height() - self.widget_data.rect.height as usize) as u32;
                if self.scroll_row != scroll_row {
                    self.scroll_row = scroll_row;
                    self.widget_data.dirty = true;
                    return ActionFlags::Dirty;
                }
            }
            Event::KeyPress { key: Key::Left, alt: false, ctrl: false, shift: false } => {
                if self.scroll_column > 0 {
                    self.scroll_column -= 1;
                    self.widget_data.dirty = true;
                    return ActionFlags::Dirty;
                }
            }
            Event::KeyPress { key: Key::Right, alt: false, ctrl: false, shift: false } => {
                if self.scroll_column < u32::MAX {
                    let scroll_column = self.scroll_column;
                    self.scroll_column += 1;
                    self.clamp_scroll_column();
                    if self.scroll_column != scroll_column {
                        self.widget_data.dirty = true;
                        return ActionFlags::Dirty;
                    }
                }
            }
            Event::KeyPress { key: Key::Home, alt: false, ctrl: true, shift: false } => {
                if self.scroll_column > 0 {
                    self.scroll_column = 0;
                    self.widget_data.dirty = true;
                    return ActionFlags::Dirty;
                }
            }
            Event::KeyPress { key: Key::End, alt: false, ctrl: true, shift: false } => {
                if self.scroll_column < u32::MAX {
                    if self.widget_data.rect.width as usize > self.rich_text.width() {
                        if self.scroll_column != 0 {
                            self.scroll_column = 0;
                            self.widget_data.dirty = true;
                            return ActionFlags::Dirty;
                        }
                    } else {
                        let max_overflow = (self.rich_text.width() - self.widget_data.rect.width as usize) as u32;
                        if self.scroll_column != max_overflow {
                            self.scroll_column = max_overflow;
                            self.widget_data.dirty = true;
                            return ActionFlags::Dirty;
                        }
                    }
                }
            }
            Event::KeyPress { key: Key::PageUp, alt: false, ctrl: false, shift: false } => {
                let scroll_row = self.scroll_row.saturating_sub(self.widget_data.rect.height);

                if scroll_row != self.scroll_row {
                    self.scroll_row = scroll_row;
                    self.widget_data.dirty = true;
                    return ActionFlags::Dirty;
                }
            }
            Event::KeyPress { key: Key::PageDown, alt: false, ctrl: false, shift: false } => {
                let scroll_row = self.scroll_row.saturating_add(self.widget_data.rect.height).min(
                    (self.rich_text.height() - self.widget_data.rect.height as usize) as u32
                );

                if scroll_row != self.scroll_row {
                    self.scroll_row = scroll_row;
                    self.widget_data.dirty = true;
                    return ActionFlags::Dirty;
                }
            }
            _ => {}
        }

        ActionFlags::None
    }
}
