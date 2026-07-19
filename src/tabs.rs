use crate::{event::{Event, Key}, rich_text::{RichText, RichTextStyle}, style::TextDecoration, termio::TermIO, widget::{Rect, Widget}};

#[derive(Debug)]
pub struct Tab {
    pub title: String,
    pub menonic: char,
    pub content: Box<dyn TabContent>,
}

pub trait TabContent: Widget + std::fmt::Debug {}

#[derive(Debug, Default)]
pub struct Tabs {
    tabs: Vec<Tab>,
    formatted_tabs: Vec<RichText>,
    draw_rect: Rect,
    selected_tab_index: usize,
}

impl Tabs {
    #[inline]
    pub fn new(tabs: impl Into<Vec<Tab>>) -> Self {
        let mut tabs = Self {
            tabs: tabs.into(),
            formatted_tabs: Vec::new(),
            draw_rect: Rect::default(),
            selected_tab_index: 0,
        };
        tabs.update();
        tabs
    }

    fn update(&mut self) {
        let style = RichTextStyle::build().text_decoration(TextDecoration::Underline).into();
        for tab in &self.tabs {
            let mut label = RichText::new();
            if let Some(index) = tab.title.find(|ch: char| ch.eq_ignore_ascii_case(&tab.menonic)) {
                label.append_plain_text(&tab.title[..index]);
                let next_index = tab.title.ceil_char_boundary(index + 1);
                label.append_text(&style, &tab.title[index..next_index]);
                label.append_plain_text(&tab.title[next_index..]);
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
}

impl Widget for Tabs {
    #[inline]
    fn draw_rect(&self) -> Rect {
        self.draw_rect
    }

    #[inline]
    fn set_draw_rect(&mut self, rect: &Rect) {
        self.draw_rect = *rect;

        if self.selected_tab_index < self.tabs.len() {
            let child = &mut self.tabs[self.selected_tab_index].content;
            child.set_draw_rect(&Rect {
                row: rect.row + 1,
                height: if rect.height > 0 { rect.height - 1 } else { 0 },
                ..*rect
            });
        }
    }

    fn draw(&self, termio: &mut TermIO, global_row: i32, global_column: i32) -> std::io::Result<()> {
        let row = self.draw_rect.row + global_row;
        let column = self.draw_rect.column + global_column;

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

        if self.selected_tab_index < self.tabs.len() {
            let child = &self.tabs[self.selected_tab_index].content;
            child.draw(termio, global_row + 1, global_column)?;
        }

        Ok(())
    }

    fn handle_event(&mut self, event: &Event) {
        match event {
            Event::KeyPress { key: Key::Char(ch), ctrl: false, alt: true, shift: false } => {
                for (index, tab) in self.tabs.iter_mut().enumerate() {
                    if tab.menonic.eq_ignore_ascii_case(ch) {
                        self.selected_tab_index = index;

                        tab.content.set_draw_rect(&Rect {
                            row: self.draw_rect.row + 1,
                            height: if self.draw_rect.height > 0 { self.draw_rect.height - 1 } else { 0 },
                            ..self.draw_rect
                        });
                        break;
                    }
                }
            }
            _ => {
                if self.selected_tab_index < self.tabs.len() {
                    self.tabs[self.selected_tab_index].content.handle_event(event);
                }
            }
        }
    }
}
