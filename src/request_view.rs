use crate::{colorize::{colorize_json, colorize_sgml}, mime_types::{is_json, is_sgml}, property_list::PropertyList, rect::Rect, rich_text::RichText, rich_text_view::RichTextView, schema::Entry, tabs::{Tab, Tabs}, termio::TermIO, widget::{Widget, WidgetData, WidgetId}};

#[derive(Debug)]
pub struct RequestView {
    widget_data: WidgetData,
    tabs: Tabs,
}

impl RequestView {
    pub fn new() -> Self {
        Self {
            widget_data: WidgetData::new(),
            tabs: Tabs::new(),
        }
    }

    pub fn set_entry(&mut self, entry: &Entry) {
        let mut info = PropertyList::new("Key", "Value");
        let mut request_headers = PropertyList::new("Header", "Value");

        request_headers.set_rows(
            entry.request.headers.iter()
            .map(|header| (header.name.to_string(), header.value.to_string()))
            .collect::<Vec<_>>()
        );


        let mut info_rows: Vec<(String, String)> = Vec::new();

        // TODO
        info_rows.push(("Method".to_string(), entry.request.method.to_string()));
        info_rows.push(("HTTP Version".to_string(), entry.request.http_version.to_string()));
        info_rows.push(("URL".to_string(), entry.request.url.to_string()));

        if let Some(server_ip_address) = &entry.server_ip_address {
            info_rows.push(("Server IP".to_string(), server_ip_address.to_string()));
        }

        if let Some(connection) = &entry.connection {
            info_rows.push(("Connection".to_string(), connection.to_string()));
        }

        if let Some(pageref) = &entry.pageref {
            info_rows.push(("Page Ref".to_string(), pageref.to_string()));
        }

        if let Some(priority) = &entry._priority {
            info_rows.push(("Priority".to_string(), priority.to_string()));
        }

        if let Some(security_state) = &entry._security_state {
            info_rows.push(("Security State".to_string(), security_state.to_string()));
        }

        info.set_rows(info_rows);

        let mut tabs = vec![
            Tab {
                title: "Info".to_string(),
                mnemonic: 'I',
                content: Box::new(info)
            },
            Tab {
                title: "Request Headers".to_string(),
                mnemonic: 'H',
                content: Box::new(request_headers),
            },
        ];

        // TODO: all kinds of info, draw timings, etc.

        if !entry.request.query_string.is_empty() {
            let mut request_query = PropertyList::new("Key", "Value");

            request_query.set_rows(
                entry.request.query_string.iter()
                .map(|param| (param.name.to_string(), param.value.to_string()))
                .collect::<Vec<_>>()
            );

            tabs.push(Tab {
                title: "Query".to_string(),
                mnemonic: 'Q',
                content: Box::new(request_query)
            });
        }

        if !entry.request.cookies.is_empty() {
            let mut request_cookies = PropertyList::new("Key", "Value");

            request_cookies.set_rows(
                entry.request.cookies.iter()
                .map(|param| (param.name.to_string(), param.value.to_string()))
                .collect::<Vec<_>>()
            );

            tabs.push(Tab {
                title: "Cookies".to_string(),
                mnemonic: 'K',
                content: Box::new(request_cookies)
            });
        }

        if let Some(post_data) = &entry.request.post_data {
            let params = &post_data.params;
            if !params.is_empty() {
                let mut post_data = PropertyList::new("Key", "Value");

                post_data.set_rows(
                    params.iter()
                    .map(|param| (
                        param.name.to_string(),
                        if let Some(value) = &param.value { value.to_string() } else { String::new() }
                    ))
                    .collect::<Vec<_>>()
                );

                tabs.push(Tab {
                    title: "Post Data".to_string(),
                    mnemonic: 'P',
                    content: Box::new(post_data),
                });
            }

            if let Some(text) = &post_data.text {
                let mime_type: &str = if let Some(mime_type) = &post_data.mime_type {
                    &mime_type
                } else {
                    "application/octet-stream"
                };

                let mime_type = mime_type.split(';').next().unwrap_or(&mime_type);
                let mut rich_text = RichText::new();

                if is_sgml(mime_type) {
                    colorize_sgml(text, &mut rich_text);
                } else if is_json(mime_type) {
                    colorize_json(text, &mut rich_text);
                } else {
                    rich_text.append_plain_text(text);
                }

                tabs.push(Tab {
                    title: "Request Body".to_string(),
                    mnemonic: 'B',
                    content: Box::new(RichTextView::new(rich_text))
                });
            }
        }

        // TODO: entry.request.comment

        if let Some(comment) = &entry.comment {
            // TODO: wrappable text view!
            // TODO: editable plain text view!
            tabs.push(Tab {
                title: "Commet".to_string(),
                mnemonic: 'C',
                content: Box::new(RichTextView::from_plain_text(comment))
            });
        }

        self.tabs.set_tabs(tabs);

        self.widget_data.dirty = true;
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
