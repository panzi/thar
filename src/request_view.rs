use crate::{property_list::PropertyList, rect::Rect, tabs::{Tab, Tabs}, termio::TermIO, widget::{Widget, WidgetData, WidgetId}};

#[derive(Debug)]
pub struct RequestView {
    widget_data: WidgetData,
    tabs: Tabs,
    request_headers_id: WidgetId,
}

impl RequestView {
    pub fn new() -> Self {
        let request_headers = PropertyList::new("Header", "Value");
        let request_headers_id = request_headers.widget_id();

        let tabs = Tabs::new([
            Tab {
                title: "Request Headers".to_string(),
                mnemonic: 'H',
                content: Box::new(request_headers),
            }
        ]);

        Self {
            widget_data: WidgetData::new(),
            tabs,
            request_headers_id,
        }
    }

    #[inline]
    pub fn request_headers_id(&self) -> WidgetId {
        self.request_headers_id
    }
}

impl Widget for RequestView {
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
            self.tabs.set_draw_rect(rect);
        }
    }

    fn draw(&mut self, termio: &mut TermIO, parent_row: i32, parent_column: i32) -> std::io::Result<()> {
        self.tabs.draw(termio, parent_row, parent_column)
    }

    fn handle_event(&mut self, event: &crate::event::Event, broker: &mut crate::message::MessageBroker) -> crate::widget::ActionFlags {
        self.tabs.handle_event(event, broker)
    }

    fn handle_message(&mut self, message: &mut crate::message::Message, broker: &mut crate::message::MessageBroker) -> crate::widget::ActionFlags {
        self.tabs.handle_message(message, broker)
    }
}
