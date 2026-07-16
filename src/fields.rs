use crate::{color::{Color, Color16}, rich_text::{RichText, RichTextStyle}, schema::{CacheState, Entry}, widgets::table::{Align, ColumnDef}};

use std::fmt::Write;

pub trait Field {
    fn header(&self) -> &str;
    fn align(&self) -> Align;
    fn write_rich_text(&self, entry: &Entry, rich_text: &mut RichText, buf: &mut String) -> std::fmt::Result;

    #[inline]
    fn to_column_def(&self) -> ColumnDef {
        ColumnDef {
            header: RichText::from_plain_text(self.header()),
            align: self.align(),
        }
    }
}

impl<F> From<F> for ColumnDef where F: Field {
    #[inline]
    fn from(value: F) -> Self {
        value.to_column_def()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryField {
    StartedDateTime,
    Time,
    Request(RequestField),
    Response(ResponseField),
    Cache(CacheField),
    Timings(TimingsField),
    ServerIpAddress,
    Connection,
    Comment,
}

impl Field for EntryField {
    #[inline]
    fn header(&self) -> &str {
        match self {
            Self::StartedDateTime  => "Started At",
            Self::Time             => "Time",
            Self::Request(req)     => req.header(),
            Self::Response(res)    => res.header(),
            Self::Cache(cache)     => cache.header(),
            Self::Timings(timings) => timings.header(),
            Self::ServerIpAddress  => "Server IP",
            Self::Connection       => "Connection",
            Self::Comment          => "Comment",
        }
    }

    #[inline]
    fn align(&self) -> Align {
        match self {
            Self::StartedDateTime  => Align::Left,
            Self::Time             => Align::Right,
            Self::Request(req)     => req.align(),
            Self::Response(res)    => res.align(),
            Self::Cache(cache)     => cache.align(),
            Self::Timings(timings) => timings.align(),
            Self::ServerIpAddress  => Align::Right,
            Self::Connection       => Align::Left,
            Self::Comment          => Align::Left,
        }
    }

    fn write_rich_text(&self, entry: &Entry, rich_text: &mut RichText, buf: &mut String) -> std::fmt::Result {
        match self {
            Self::StartedDateTime => {
                write!(buf, "{}", entry.started_date_time)?;
                rich_text.append_plain_text(buf);
            }
            Self::Time => {
                write!(buf, "{}", entry.time)?;
                rich_text.append_plain_text(buf);
            }
            Self::Request(req) => {
                req.write_rich_text(entry, rich_text, buf)?;
            }
            Self::Response(res) => {
                res.write_rich_text(entry, rich_text, buf)?;
            }
            Self::Cache(cache) => {
                cache.write_rich_text(entry, rich_text, buf)?;
            }
            Self::Timings(timings) => {
                timings.write_rich_text(entry, rich_text, buf)?;
            }
            Self::ServerIpAddress => {
                if let Some(server_ip_address) = &entry.server_ip_address {
                    rich_text.append_plain_text(server_ip_address);
                }
            }
            Self::Connection => {
                if let Some(connection) = &entry.connection {
                    rich_text.append_plain_text(connection);
                }
            }
            Self::Comment => {
                if let Some(connection) = &entry.comment {
                    rich_text.append_plain_text(connection);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestField {
    Method,
    Url,
    Scheme,
    Host,
    Port,
    Domain,
    Path,
    Query,
    Fragment,
    HttpVersion,
    HeadersSize,
    BodySize,
    Comment,
}

pub fn get_method_color(method: &str) -> Color {
    if method.eq_ignore_ascii_case("GET") {
        Color::Color16(Color16::Green)
    } else if method.eq_ignore_ascii_case("POST") {
        Color::Color16(Color16::Yellow)
    } else if method.eq_ignore_ascii_case("PUT") {
        Color::Color16(Color16::Cyan)
    } else if method.eq_ignore_ascii_case("PATCH") {
        Color::Color16(Color16::Blue)
    } else if method.eq_ignore_ascii_case("DELETE") {
        Color::Color16(Color16::Red)
    } else if method.eq_ignore_ascii_case("HEAD") {
        Color::Color16(Color16::Magenta)
    } else {
        Color::Color16(Color16::BrightBlack)
    }
}

impl Field for RequestField {
    #[inline]
    fn header(&self) -> &str {
        match self {
            Self::Method      => "Method",
            Self::Url         => "URL",
            Self::Scheme      => "Scheme",
            Self::Host        => "Host",
            Self::Port        => "Port",
            Self::Domain      => "Domain",
            Self::Path        => "Path",
            Self::Query       => "Query",
            Self::Fragment    => "Fragment",
            Self::HttpVersion => "HTTP Version",
            Self::HeadersSize => "Request Header Size",
            Self::BodySize    => "Request Body Size",
            Self::Comment     => "Request Comment",
        }
    }

    #[inline]
    fn align(&self) -> Align {
        match self {
            Self::Method      => Align::Left,
            Self::Url         => Align::Left,
            Self::Scheme      => Align::Left,
            Self::Host        => Align::Left,
            Self::Port        => Align::Right,
            Self::Domain      => Align::Left,
            Self::Path        => Align::Left,
            Self::Query       => Align::Left,
            Self::Fragment    => Align::Left,
            Self::HttpVersion => Align::Right,
            Self::HeadersSize => Align::Right,
            Self::BodySize    => Align::Right,
            Self::Comment     => Align::Left,
        }
    }

    fn write_rich_text(&self, entry: &Entry, rich_text: &mut RichText, buf: &mut String) -> std::fmt::Result {
        match self {
            Self::Method => {
                let style = RichTextStyle::build().foreground(get_method_color(&entry.request.method)).into_inner();
                rich_text.append_text(&style, &entry.request.method);
            },
            Self::Url => {
                write!(buf, "{}", entry.request.url)?;
                rich_text.append_plain_text(buf);
            },
            Self::Scheme => {
                rich_text.append_plain_text(entry.request.url.scheme());
            },
            Self::Host => {
                if let Some(host) = entry.request.url.host_str() {
                    rich_text.append_plain_text(host);
                }
            },
            Self::Port => {
                if let Some(port) = entry.request.url.port() {
                    write!(buf, "{}", port)?;
                    rich_text.append_plain_text(buf);
                }
            },
            Self::Domain => {
                if let Some(domain) = entry.request.url.domain() {
                    rich_text.append_plain_text(domain);
                }
            },
            Self::Path => {
                rich_text.append_plain_text(entry.request.url.path());
            },
            Self::Query => {
                if let Some(query) = entry.request.url.query() {
                    rich_text.append_plain_text(query);
                }
            },
            Self::Fragment => {
                if let Some(fragment) = entry.request.url.fragment() {
                    rich_text.append_plain_text(fragment);
                }
            },
            Self::HttpVersion => {
                rich_text.append_plain_text(&entry.request.http_version);
            },
            Self::HeadersSize => {
                write!(buf, "{}", entry.request.headers_size)?;
                rich_text.append_plain_text(buf);
            },
            Self::BodySize => {
                write!(buf, "{}", entry.request.body_size)?;
                rich_text.append_plain_text(buf);
            },
            Self::Comment => {
                if let Some(comment) = &entry.comment {
                    rich_text.append_plain_text(comment);
                }
            },
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseField {
    Status,
    StatusText,
    RedirectUrl,
    Content(ContentField),
    HeadersSize,
    BodySize,
    Comment,
}

impl Field for ResponseField {
    #[inline]
    fn header(&self) -> &str {
        match self {
            Self::Status           => "Status",
            Self::StatusText       => "Status Text",
            Self::RedirectUrl      => "Redirect URL",
            Self::Content(content) => content.header(),
            Self::HeadersSize      => "Response Header Size",
            Self::BodySize         => "Response Body Size",
            Self::Comment          => "Response Comment",
        }
    }

    #[inline]
    fn align(&self) -> Align {
        match self {
            Self::Status           => Align::Right,
            Self::StatusText       => Align::Left,
            Self::RedirectUrl      => Align::Left,
            Self::Content(content) => content.align(),
            Self::HeadersSize      => Align::Right,
            Self::BodySize         => Align::Right,
            Self::Comment          => Align::Left,
        }
    }

    fn write_rich_text(&self, entry: &Entry, rich_text: &mut RichText, buf: &mut String) -> std::fmt::Result {
        let Some(response) = &entry.response else {
            return Ok(())
        };

        match self {
            Self::Status => {
                write!(buf, "{}", response.status)?;
                rich_text.append_plain_text(buf);
            },
            Self::StatusText => {
                rich_text.append_plain_text(&response.status_text);
            },
            Self::RedirectUrl => {
                if let Some(redirect_url) = &response.redirect_url {
                    write!(buf, "{}", redirect_url)?;
                    rich_text.append_plain_text(buf);
                }
            },
            Self::Content(content) => {
                content.write_rich_text(entry, rich_text, buf)?;
            },
            Self::HeadersSize => {
                write!(buf, "{}", response.headers_size)?;
                rich_text.append_plain_text(buf);
            },
            Self::BodySize => {
                write!(buf, "{}", response.body_size)?;
                rich_text.append_plain_text(buf);
            },
            Self::Comment => {
                if let Some(comment) = &response.comment {
                    rich_text.append_plain_text(comment);
                }
            },
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentField {
    Size,
    Compression,
    MimeType,
    Text,
    Comment,
}

impl Field for ContentField {
    #[inline]
    fn header(&self) -> &str {
        match self {
            Self::Size        => "Content Size",
            Self::Compression => "Compression",
            Self::MimeType    => "Mime Type",
            Self::Text        => "Content Text",
            Self::Comment     => "Content Comment",
        }
    }

    #[inline]
    fn align(&self) -> Align {
        match self {
            Self::Size        => Align::Right,
            Self::Compression => Align::Right,
            Self::MimeType    => Align::Left,
            Self::Text        => Align::Left,
            Self::Comment     => Align::Left,
        }
    }

    fn write_rich_text(&self, entry: &Entry, rich_text: &mut RichText, buf: &mut String) -> std::fmt::Result {
        let Some(response) = &entry.response else {
            return Ok(());
        };

        let Some(content) = &response.content else {
            return Ok(());
        };

        match self {
            Self::Size => {
                write!(buf, "{}", content.size)?;
                rich_text.append_plain_text(buf);
            }
            Self::Compression => {
                if let Some(compression) = content.compression {
                    write!(buf, "{}", compression)?;
                    rich_text.append_plain_text(buf);
                }
            }
            Self::MimeType => {
                if let Some(mime_type) = &content.mime_type {
                    rich_text.append_plain_text(mime_type);
                }
            }
            Self::Text => {
                if let Some(text) = &content.text {
                    rich_text.append_plain_text(text);
                }
            }
            Self::Comment => {
                if let Some(comment) = &content.comment {
                    rich_text.append_plain_text(comment);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheField {
    BeforeRequest(CacheStateField),
    AfterRequest(CacheStateField),
    Comment,
}

impl Field for CacheField {
    #[inline]
    fn header(&self) -> &str {
        match self {
            Self::BeforeRequest(CacheStateField::Expires)    => "Before Request Expires",
            Self::BeforeRequest(CacheStateField::LastAccess) => "Before Request Last Access",
            Self::BeforeRequest(CacheStateField::ETag)       => "Before Request E-Tag",
            Self::BeforeRequest(CacheStateField::HitCount)   => "Before Request Hit Count",
            Self::BeforeRequest(CacheStateField::Comment)    => "Before Request Cache State Comment",

            Self::AfterRequest(CacheStateField::Expires)    => "After Request Expires",
            Self::AfterRequest(CacheStateField::LastAccess) => "After Request Last Access",
            Self::AfterRequest(CacheStateField::ETag)       => "After Request E-Tag",
            Self::AfterRequest(CacheStateField::HitCount)   => "After Request Hit Count",
            Self::AfterRequest(CacheStateField::Comment)    => "After Request Cache State Comment",

            Self::Comment => "Cache Comment",
        }
    }

    #[inline]
    fn align(&self) -> Align {
        match self {
            Self::BeforeRequest(state) => state.align(),
            Self::AfterRequest(state)  => state.align(),
            Self::Comment => Align::Left,
        }
    }

    fn write_rich_text(&self, entry: &Entry, rich_text: &mut RichText, buf: &mut String) -> std::fmt::Result {
        let Some(cache) = &entry.cache else {
            return Ok(());
        };

        match self {
            Self::BeforeRequest(state_field) => {
                if let Some(Some(state)) = &cache.before_request {
                    state_field.write_rich_text(state, rich_text, buf)?;
                }
            },
            Self::AfterRequest(state_field) => {
                if let Some(Some(state)) = &cache.after_request {
                    state_field.write_rich_text(state, rich_text, buf)?;
                }
            },
            Self::Comment => {
                if let Some(comment) = &cache.comment {
                    rich_text.append_plain_text(comment);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheStateField {
    Expires,
    LastAccess,
    ETag,
    HitCount,
    Comment,
}

impl CacheStateField {
    pub fn write_rich_text(&self, state: &CacheState, rich_text: &mut RichText, buf: &mut String) -> std::fmt::Result {
        match self {
            Self::Expires => {
                if let Some(expires) = &state.expires {
                    write!(buf, "{}", expires)?;
                    rich_text.append_plain_text(buf);
                }
            }
            Self::LastAccess => {
                write!(buf, "{}", state.last_access)?;
                rich_text.append_plain_text(buf);
            }
            Self::ETag => {
                if let Some(e_tag) = &state.e_tag {
                    rich_text.append_plain_text(e_tag);
                }
            }
            Self::HitCount => {
                write!(buf, "{}", state.hit_count)?;
                rich_text.append_plain_text(buf);
            }
            Self::Comment => {
                if let Some(comment) = &state.comment {
                    rich_text.append_plain_text(comment);
                }
            }
        }

        Ok(())
    }

    #[inline]
    pub fn header(&self) -> &str {
        match self {
            Self::Expires    => "Expires",
            Self::LastAccess => "Last Access",
            Self::ETag       => "E-Tag",
            Self::HitCount   => "Hit Count",
            Self::Comment    => "Cache State Comment",
        }
    }

    #[inline]
    pub fn align(&self) -> Align {
        match self {
            Self::Expires    => Align::Right,
            Self::LastAccess => Align::Right,
            Self::ETag       => Align::Left,
            Self::HitCount   => Align::Right,
            Self::Comment    => Align::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingsField {
    Blocked,
    DNS,
    Connect,
    Send,
    Wait,
    Receive,
    SSL,
    Comment,
}

impl Field for TimingsField {
    #[inline]
    fn header(&self) -> &str {
        match self {
            Self::Blocked => "Blocked",
            Self::DNS     => "DNS",
            Self::Connect => "Connect",
            Self::Send    => "Send",
            Self::Wait    => "Wait",
            Self::Receive => "Receive",
            Self::SSL     => "SSL",
            Self::Comment => "Timings Comment",
        }
    }

    #[inline]
    fn align(&self) -> Align {
        match self {
            Self::Blocked => Align::Right,
            Self::DNS     => Align::Right,
            Self::Connect => Align::Right,
            Self::Send    => Align::Right,
            Self::Wait    => Align::Right,
            Self::Receive => Align::Right,
            Self::SSL     => Align::Right,
            Self::Comment => Align::Left,
        }
    }

    fn write_rich_text(&self, entry: &Entry, rich_text: &mut RichText, buf: &mut String) -> std::fmt::Result {
        let Some(timings) = &entry.timings else {
            return Ok(());
        };

        match self {
            Self::Blocked => {
                write!(buf, "{}", timings.blocked)?;
                rich_text.append_plain_text(buf);
            }
            Self::DNS => {
                write!(buf, "{}", timings.dns)?;
                rich_text.append_plain_text(buf);
            }
            Self::Connect => {
                write!(buf, "{}", timings.connect)?;
                rich_text.append_plain_text(buf);
            }
            Self::Send => {
                write!(buf, "{}", timings.send)?;
                rich_text.append_plain_text(buf);

            }
            Self::Wait => {
                write!(buf, "{}", timings.wait)?;
                rich_text.append_plain_text(buf);
            }
            Self::Receive => {
                write!(buf, "{}", timings.receive)?;
                rich_text.append_plain_text(buf);
            }
            Self::SSL => {
                write!(buf, "{}", timings.ssl)?;
                rich_text.append_plain_text(buf);
            }
            Self::Comment => {
                if let Some(comment) = &timings.comment {
                    rich_text.append_plain_text(comment);
                }
            }
        }

        Ok(())
    }
}
