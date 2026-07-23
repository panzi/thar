use crate::{color::{Color, Color16}, event::{Event, Key}, message::MessageBroker, rect::Rect, rich_text::{RichText, RichTextStyle}, style::TextDecoration, termio::TermIO, widget::{ActionFlags, Widget, WidgetId, next_widget_id}};

#[derive(Debug)]
pub struct Tab {
    pub title: String,
    pub mnemonic: char,
    pub content: Box<dyn Widget>,
}

#[derive(Debug)]
pub struct Tabs {
    tabs: Vec<Tab>,
    formatted_tabs: Vec<RichText>,
    widget_id: WidgetId,
    draw_rect: Rect,
    selected_tab_index: usize,
}

impl Tabs {
    #[inline]
    pub fn new(tabs: impl Into<Vec<Tab>>) -> Self {
        let mut tabs = Self {
            tabs: tabs.into(),
            formatted_tabs: Vec::new(),
            widget_id: next_widget_id(),
            draw_rect: Rect::default(),
            selected_tab_index: 0,
        };
        tabs.update();
        tabs
    }

    fn update(&mut self) {
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
            self.selected_tab_index = index;
        }
    }

    #[inline]
    pub fn set_selected_tab_id(&mut self, widget_id: WidgetId) {
        if let Some(index) = self.tabs.iter().position(|tab| tab.content.widget_id() == widget_id) {
            self.selected_tab_index = index;
        }
    }
}

impl Widget for Tabs {
    #[inline]
    fn widget_id(&self) -> WidgetId {
        self.widget_id
    }

    #[inline]
    fn draw_rect(&self) -> Rect {
        self.draw_rect
    }

    #[inline]
    fn set_draw_rect(&mut self, rect: &Rect) {
        self.draw_rect = *rect;

        if let Some(tab) = self.tabs.get_mut(self.selected_tab_index) {
            tab.content.set_draw_rect(&Rect {
                row: rect.row + 1,
                height: if rect.height > 0 { rect.height - 1 } else { 0 },
                ..*rect
            });
        }
    }

    fn draw(&self, termio: &mut TermIO, parent_row: i32, parent_column: i32) -> std::io::Result<()> {
        let row = self.draw_rect.row + parent_row;
        let column = self.draw_rect.column + parent_column;

        // TODO: cropping
        let mut tab_column = column;
        for (index, tab) in self.formatted_tabs.iter().enumerate() {
            if index != 0 {
                tab_column += 1;
            }
            termio.set_inverted(index == self.selected_tab_index);
            let res = tab.draw(termio, row, tab_column);
            termio.set_inverted(index != self.selected_tab_index);
            res?;

            tab_column += tab.width() as i32;
        }

        termio.set_inverted(false);

        if self.selected_tab_index < self.tabs.len() {
            let child = &self.tabs[self.selected_tab_index].content;
            child.draw(termio, parent_row, parent_column)?;
        }

        Ok(())
    }

    fn handle_event(&mut self, event: &Event, broker: &mut MessageBroker) -> ActionFlags {
        if let Event::KeyPress { key: Key::Char(ch), ctrl: false, alt: true, shift: false } = event {
            for (index, tab) in self.tabs.iter_mut().enumerate() {
                if tab.mnemonic.eq_ignore_ascii_case(ch) {
                    self.selected_tab_index = index;

                    tab.content.set_draw_rect(&Rect {
                        row: self.draw_rect.row + 1,
                        height: if self.draw_rect.height > 0 { self.draw_rect.height - 1 } else { 0 },
                        ..self.draw_rect
                    });
                    return ActionFlags::Redraw;
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
