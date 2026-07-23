use crate::{color::Color, event::{Event, Key}, fields::{EntryField, Field, PageField}, rect::Rect, rich_text::RichText, schema::HAR, table::{SelectTableRow, Table}, tabs::{Tab, Tabs}, widget::{ActionFlags, Widget, WidgetId, next_widget_id}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Tabs,
    Entry(usize),
    Page(usize),
}

#[derive(Debug)]
pub struct AppConfig<'a> {
    pub request_columns: &'a [EntryField],
    pub page_columns: &'a [PageField],
}

#[derive(Debug)]
pub struct App {
    tabs: Tabs,
    draw_rect: Rect,
    widget_id: WidgetId,
    requests_table_id: WidgetId,
    pages_table_id: WidgetId,
    har: HAR,
    active_view: ActiveView,
}

impl App {
    pub fn new<'a>(har: HAR, config: &AppConfig<'a>) -> Self {
        let mut requests_table = Table::new();
        let mut pages_table = Table::new();

        let requests_table_id = requests_table.widget_id();
        let pages_table_id = pages_table.widget_id();

        requests_table.set_columns(config.request_columns.iter().cloned().map(Into::into));
        pages_table.set_columns(config.page_columns.iter().cloned().map(Into::into));

        let mut buf = String::new();

        for entry in &har.log.entries {
            let mut row = Vec::new();
            for column in config.request_columns {
                buf.clear();

                let mut cell = RichText::new();
                column.write_rich_text(entry, &mut cell, &mut buf).unwrap();

                row.push(cell);
            }
            requests_table.rows_mut().push(row);
        }

        for page in &har.log.pages {
            let mut row = Vec::new();
            for column in config.page_columns {
                buf.clear();

                let mut cell = RichText::new();
                column.write_rich_text(page, &mut cell, &mut buf).unwrap();

                row.push(cell);
            }
            pages_table.rows_mut().push(row);
        }

        requests_table.update();
        pages_table.update();

        let tabs = Tabs::new([
                Tab {
                    title: "Requests".to_string(),
                    mnemonic: 'R',
                    content: Box::new(requests_table),
                },
                Tab {
                    title: "Pages".to_string(),
                    mnemonic: 'P',
                    content: Box::new(pages_table),
                },

        ]);

        Self {
            draw_rect: Rect::default(),
            widget_id: next_widget_id(),
            requests_table_id,
            pages_table_id,
            tabs,
            har,
            active_view: ActiveView::Tabs,
        }
    }
}

impl Widget for App {
    #[inline]
    fn draw_rect(&self) -> Rect {
        self.draw_rect
    }

    #[inline]
    fn widget_id(&self) -> WidgetId {
        self.widget_id
    }

    fn set_draw_rect(&mut self, rect: &Rect) {
        self.draw_rect = *rect;

        let child_rect = Rect {
            row: 0,
            column: 0,
            width: rect.width,
            height: rect.height,
        };

        match self.active_view {
            ActiveView::Tabs => {
                self.tabs.set_draw_rect(&child_rect);
            }
            ActiveView::Entry(_) => {
                // TODO
            }
            ActiveView::Page(_) => {
                // TODO
            }
        }
    }

    fn draw(&self, termio: &mut crate::termio::TermIO, parent_row: i32, parent_column: i32) -> std::io::Result<()> {
        let row = self.draw_rect.row + parent_row;
        let column = self.draw_rect.column + parent_column;

        termio.clear_style()?;
        termio.clear_screen()?;

        let res = match self.active_view {
            ActiveView::Tabs => {
                self.tabs.draw(termio, row, column)
            }
            ActiveView::Entry(index) => {
                let text = RichText::from_plain_text(&format!("TODO: Request {index}"));
                text.draw(termio, row, column)
            }
            ActiveView::Page(index) => {
                let text = RichText::from_plain_text(&format!("TODO: Page {index}"));
                text.draw(termio, row, column)
            }
        };

        termio.set_default_fg(Color::Default);
        termio.set_default_bg(Color::Default);

        termio.flush()?;

        res
    }

    fn handle_event(&mut self, event: &crate::event::Event, broker: &mut crate::message::MessageBroker) -> ActionFlags {
        match event {
            Event::KeyPress { key: Key::Char('q'), alt: false, ctrl: false, shift: false } => {
                broker.dispatch(AppQuit);
                return ActionFlags::None;
            }
            Event::KeyPress { key: Key::Escape, ctrl: false, alt: false, shift: false } => {
                match self.active_view {
                    ActiveView::Entry(_) => {
                        self.active_view = ActiveView::Tabs;
                        self.tabs.set_selected_tab_id(self.requests_table_id);
                        return ActionFlags::Redraw;
                    }
                    ActiveView::Page(_) => {
                        self.active_view = ActiveView::Tabs;
                        self.tabs.set_selected_tab_id(self.pages_table_id);
                        return ActionFlags::Redraw;
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        match self.active_view {
            ActiveView::Tabs => {
                return event.send_to(&mut self.tabs, broker);
            }
            ActiveView::Entry(index) => {
                // TODO
            }
            ActiveView::Page(index) => {
                // TODO
            }
        }

        ActionFlags::None
    }

    fn handle_message(&mut self, message: &mut crate::message::Message, broker: &mut crate::message::MessageBroker) -> ActionFlags {
        if let Some(&SelectTableRow { widget_id, row_index }) = message.data() {
            if widget_id == self.requests_table_id {
                message.stop_propergation();
                let view = ActiveView::Entry(row_index);
                if view != self.active_view {
                    self.active_view = view;
                    return ActionFlags::Redraw;
                } else {
                    return ActionFlags::None;
                }
            } else if widget_id == self.pages_table_id {
                message.stop_propergation();
                let view = ActiveView::Page(row_index);
                if view != self.active_view {
                    self.active_view = view;
                    return ActionFlags::Redraw;
                } else {
                    return ActionFlags::None;
                }
            }
        }

        let flags = self.tabs.handle_message(message, broker);

        if self.active_view == ActiveView::Tabs {
            return flags;
        }

        ActionFlags::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppQuit;
