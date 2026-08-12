use crate::{color::{Color, Color16}, event::{Event, Key}, message::MessageBroker, rect::Rect, rich_text::{RichText, RichTextStyle}, style::TextDecoration, termio::TermIO, widget::{ActionFlags, Widget, WidgetData, WidgetId}};

#[derive(Debug)]
pub struct Tab {
    pub title: String,
    pub mnemonic: char,
    pub content: Box<dyn Widget>,
}

#[derive(Debug)]
pub struct Tabs {
    widget_data: WidgetData,
    tabs: Vec<Tab>,
    formatted_tabs: Vec<RichText>,
    selected_tab_index: usize,
}

impl Tabs {
    #[inline]
    pub fn with_tabs(tabs: impl Into<Vec<Tab>>) -> Self {
        let mut tabs = Self {
            widget_data: WidgetData::new(),
            tabs: tabs.into(),
            formatted_tabs: Vec::new(),
            selected_tab_index: 0,
        };
        tabs.update();
        tabs
    }

    #[inline]
    pub fn new() -> Self {
        Self {
            widget_data: WidgetData::new(),
            tabs: Vec::new(),
            formatted_tabs: Vec::new(),
            selected_tab_index: 0,
        }
    }

    pub fn clear(&mut self) {
        self.tabs.clear();
        self.selected_tab_index = 0;
        self.update();
        self.widget_data.dirty = true;
    }

    pub fn add(&mut self, mut tab: Tab) {
        tab.content.set_draw_rect(&Rect {
            row: self.widget_data.rect.row + 1,
            height: self.widget_data.rect.height.saturating_sub(1),
            ..self.widget_data.rect
        });
        self.tabs.push(tab);
        self.update();
        self.widget_data.dirty = true;
    }

    pub fn extend(&mut self, tabs: impl IntoIterator<Item = Tab>) {
        let index = self.tabs.len();
        self.tabs.extend(tabs);
        for tab in &mut self.tabs[index..] {
            tab.content.set_draw_rect(&Rect {
                row: self.widget_data.rect.row + 1,
                height: self.widget_data.rect.height.saturating_sub(1),
                ..self.widget_data.rect
            });
        }
        self.update();
        self.widget_data.dirty = true;
    }

    pub fn set_tabs(&mut self, tabs: impl Into<Vec<Tab>>) {
        self.tabs = tabs.into();
        for tab in &mut self.tabs {
            tab.content.set_draw_rect(&Rect {
                row: self.widget_data.rect.row + 1,
                height: self.widget_data.rect.height.saturating_sub(1),
                ..self.widget_data.rect
            });
        }
        self.selected_tab_index = 0;
        self.update();
        self.widget_data.dirty = true;
    }

    pub fn set_content(&mut self, index: usize, content: Box<dyn Widget>) {
        let tab = &mut self.tabs[index];
        tab.content = content;
        tab.content.set_draw_rect(&Rect {
            row: self.widget_data.rect.row + 1,
            height: self.widget_data.rect.height.saturating_sub(1),
            ..self.widget_data.rect
        });
        tab.content.set_dirty(true);
        self.update();
        self.widget_data.dirty = true;
    }

    fn update(&mut self) {
        self.formatted_tabs.clear();

        let fg = Color::Color16(Color16::White);
        let bg = Color::Color16(Color16::Black);

        let style = RichTextStyle::build()
            .foreground(fg)
            .background(bg)
            .into();

        let mnemonic_style = RichTextStyle::build()
            .foreground(fg)
            .background(bg)
            .text_decoration(TextDecoration::Underline)
            .into();

        for tab in &self.tabs {
            let mut label = RichText::new();
            if let Some(index) = tab.title.find(|ch: char| ch.eq_ignore_ascii_case(&tab.mnemonic)) {
                label.append_text(&style, &tab.title[..index]);
                let next_index = tab.title.ceil_char_boundary(index + 1);
                label.append_text(&mnemonic_style, &tab.title[index..next_index]);
                label.append_text(&style, &tab.title[next_index..]);
            }
            self.formatted_tabs.push(label);
        }
    }

    #[inline]
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    #[inline]
    pub fn selected_tab_index(&self) -> usize {
        self.selected_tab_index
    }

    #[inline]
    pub fn selected_tab_id(&self) -> Option<WidgetId> {
        self.tabs.get(self.selected_tab_index).map(|tab| tab.content.widget_id())
    }

    #[inline]
    pub fn set_selected_tab_index(&mut self, index: usize) {
        if index < self.tabs.len() {
            let tab = &mut self.tabs[index];

            tab.content.set_draw_rect(&Rect {
                row: self.widget_data.rect.row + 1,
                height: self.widget_data.rect.height.saturating_sub(1),
                ..self.widget_data.rect
            });
            tab.content.set_dirty(true);

            if index != self.selected_tab_index {
                self.selected_tab_index = index;
                self.widget_data.dirty = true;
            }
        }
    }

    #[inline]
    pub fn set_selected_tab_id(&mut self, widget_id: WidgetId) {
        if let Some(index) = self.tabs.iter().position(|tab| tab.content.widget_id() == widget_id) {
            self.set_selected_tab_index(index);
        }
    }

    fn draw_tabs(&self, termio: &mut TermIO, parent_row: i32, parent_column: i32) -> std::io::Result<()> {
        // TODO: cropping
        let row = self.widget_data.rect.row + parent_row;
        let column = self.widget_data.rect.column + parent_column;

        let mut tab_column = column;
        for (index, tab) in self.formatted_tabs.iter().enumerate() {
            if index != 0 {
                termio.write_str(" ")?;
                tab_column += 1;
            }
            termio.set_inverted(index == self.selected_tab_index);
            let res = tab.draw(termio, row, tab_column);
            termio.set_inverted(index != self.selected_tab_index);
            res?;

            tab_column += tab.width() as i32;
        }

        termio.set_inverted(false);

        Ok(())
    }
}

impl Widget for Tabs {
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

            if let Some(tab) = self.tabs.get_mut(self.selected_tab_index) {
                tab.content.set_draw_rect(&Rect {
                    row: rect.row + 1,
                    height: rect.height.saturating_sub(1),
                    ..*rect
                });
            }
        }
    }

    fn draw(&mut self, termio: &mut TermIO, parent_row: i32, parent_column: i32) -> std::io::Result<()> {
        if self.widget_data.dirty {
            self.draw_tabs(termio, parent_row, parent_column)?;
        }

        if self.selected_tab_index < self.tabs.len() {
            let child = &mut self.tabs[self.selected_tab_index].content;
            child.draw(termio, parent_row, parent_column)?;
        }

        self.widget_data.dirty = false;

        Ok(())
    }

    fn handle_event(&mut self, event: &Event, broker: &mut MessageBroker) -> ActionFlags {
        if let Event::KeyPress { key: Key::Char(ch), ctrl: false, alt: true, shift: false } = event {
            for (index, tab) in self.tabs.iter_mut().enumerate() {
                if tab.mnemonic.eq_ignore_ascii_case(ch) {
                    if self.selected_tab_index == index {
                        return ActionFlags::None;
                    }
                    self.set_selected_tab_index(index);
                    return ActionFlags::Dirty;
                }
            }
        }

        if let Some(tab) = self.tabs.get_mut(self.selected_tab_index) {
            return event.send_to(tab.content.as_mut(), broker);
        }

        ActionFlags::None
    }

    fn handle_message(&mut self, message: &mut crate::message::Message, broker: &mut MessageBroker) -> ActionFlags {
        let mut flags = ActionFlags::None;
        for (index, tab) in self.tabs.iter_mut().enumerate() {
            let tab_flags = tab.content.handle_message(message, broker);
            if index == self.selected_tab_index {
                flags = tab_flags;
            }
        }

        flags
    }
}
