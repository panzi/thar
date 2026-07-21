use crate::{char_width::CharWidth, color::{Color, Color16}, colorize::colorize_json, rich_text::{DEFAULT_STYLE, RichText, RichTextStyle}, schema::{Cache, CacheState, Content, Entry, Initiator, Page, PageTiming, Request, Response, Timings}, table::{Align, ColumnDef}};

use std::{fmt::Write, marker::PhantomData};

pub trait Field: Sized + std::fmt::Display {
    type Value;

    fn header(&self) -> &str;
    fn align(&self) -> Align;
    fn write_rich_text(&self, index: usize, value: &Self::Value, rich_text: &mut RichText, buf: &mut String) -> std::fmt::Result;

    #[inline]
    fn to_column_def(&self) -> ColumnDef {
        ColumnDef {
            header: RichText::from_plain_text(self.header()),
            align: self.align(),
        }
    }

    fn parse<'a>(field: &'a str) -> Result<Self, ParserError<'a>>;
}

impl<F> From<F> for ColumnDef where F: Field {
    #[inline]
    fn from(value: F) -> Self {
        value.to_column_def()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParserError<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> ParserError<'a> {
    #[inline]
    pub fn new(input: &'a str, index: usize) -> Self {
        Self { input, index }
    }

    #[inline]
    pub fn input(&self) -> &str {
        self.input
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }
}

impl<'a> std::error::Error for ParserError<'a> {}

impl<'a> std::fmt::Display for ParserError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error parsing column at index {}:\n", self.index)?;

        let mut index = 0;
        for line in self.input.split('\n') {
            writeln!(f, "    {line}")?;
            if self.index >= index && self.index <= index + line.len() {
                let width = line[..self.index - index].char_width_ignore_unprintable();
                writeln!(f, "----{:-^width$}^", "")?;
            }
            index += 1;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryField {
    Index,
    StartedDateTime,
    Time,
    Request(RequestField),
    Response(ResponseField),
    Cache(CacheField),
    Timings(TimingsField),
    ServerIpAddress,
    Connection,
    Comment,
    _ConnectionId,
    _SecurityState,
    _Initiator(InitiatorField),
    _Priority,
    _ResourceType,
    _FromCache,
}

impl std::fmt::Display for EntryField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Index            => "index".fmt(f),
            Self::StartedDateTime  => "startedDateTime".fmt(f),
            Self::Time             => "time".fmt(f),
            Self::Request(req)     => write!(f, "request.{req}"),
            Self::Response(res)    => write!(f, "result.{res}"),
            Self::Cache(cache)     => write!(f, "cache.{cache}"),
            Self::Timings(timings) => write!(f, "timings.{timings}"),
            Self::ServerIpAddress  => "serverIPAddress".fmt(f),
            Self::Connection       => "connection".fmt(f),
            Self::Comment          => "comment".fmt(f),
            Self::_ConnectionId    => "_connectionId".fmt(f),
            Self::_SecurityState   => "_securityState".fmt(f),
            Self::_Initiator(init) => write!(f, "_initiator.{init}"),
            Self::_Priority        => "_priority".fmt(f),
            Self::_ResourceType    => "_resourceType".fmt(f),
            Self::_FromCache       => "_fromCache".fmt(f),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FieldParser<F> where F: Field {
    phantom: PhantomData<F>
}

impl<F> clap::builder::TypedValueParser for FieldParser<F>
where F: Field + Clone + Send + Sync + Sized + 'static {
    type Value = F;

    fn parse_ref(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error>
    {
        let Some(value) = value.to_str() else {
            return Err(clap::Error::new(clap::error::ErrorKind::InvalidUtf8));
        };

        match F::parse(value) {
            Ok(field) => Ok(field),
            Err(err) => Err(clap::Error::raw(clap::error::ErrorKind::InvalidValue, err))
        }
    }
}

impl clap::builder::ValueParserFactory for EntryField {
    type Parser = FieldParser<EntryField>;

    #[inline]
    fn value_parser() -> Self::Parser {
        FieldParser {
            phantom: PhantomData
        }
    }
}

impl clap::builder::ValueParserFactory for PageField {
    type Parser = FieldParser<EntryField>;

    #[inline]
    fn value_parser() -> Self::Parser {
        FieldParser {
            phantom: PhantomData
        }
    }
}

impl<'a> TryFrom<&'a str> for EntryField {
    type Error = ParserError<'a>;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

impl Field for EntryField {
    type Value = Entry;

    #[inline]
    fn header(&self) -> &str {
        match self {
            Self::Index            => "Index",
            Self::StartedDateTime  => "Started At",
            Self::Time             => "Time",
            Self::Request(req)     => req.header(),
            Self::Response(res)    => res.header(),
            Self::Cache(cache)     => cache.header(),
            Self::Timings(timings) => timings.header(),
            Self::ServerIpAddress  => "Server IP",
            Self::Connection       => "Connection",
            Self::Comment          => "Comment",
            Self::_ConnectionId    => "Connection Id",
            Self::_SecurityState   => "Security State",
            Self::_Initiator(init) => init.header(),
            Self::_Priority        => "Priority",
            Self::_ResourceType    => "Resource Type",
            Self::_FromCache       => "From Cache",
        }
    }

    #[inline]
    fn align(&self) -> Align {
        match self {
            Self::Index            => Align::Right,
            Self::StartedDateTime  => Align::Left,
            Self::Time             => Align::Right,
            Self::Request(req)     => req.align(),
            Self::Response(res)    => res.align(),
            Self::Cache(cache)     => cache.align(),
            Self::Timings(timings) => timings.align(),
            Self::ServerIpAddress  => Align::Right,
            Self::Connection       => Align::Left,
            Self::Comment          => Align::Left,
            Self::_ConnectionId    => Align::Left,
            Self::_SecurityState   => Align::Left,
            Self::_Initiator(init) => init.align(),
            Self::_Priority        => Align::Left,
            Self::_ResourceType    => Align::Left,
            Self::_FromCache       => Align::Left,
        }
    }

    fn write_rich_text(&self, index: usize, entry: &Entry, rich_text: &mut RichText, buf: &mut String) -> std::fmt::Result {
        match self {
            Self::Index => {
                write!(buf, "{index}")?;
                rich_text.append_plain_text(buf);
            }
            Self::StartedDateTime => {
                write!(buf, "{}", entry.started_date_time)?;
                rich_text.append_plain_text(buf);
            }
            Self::Time => {
                write!(buf, "{}", entry.time)?;
                rich_text.append_plain_text(buf);
            }
            Self::Request(req) => {
                req.write_rich_text(0, &entry.request, rich_text, buf)?;
            }
            Self::Response(res) => {
                if let Some(response) = &entry.response {
                    res.write_rich_text(0, response, rich_text, buf)?;
                }
            }
            Self::Cache(cache) => {
                if let Some(value) = &entry.cache {
                    cache.write_rich_text(0, value, rich_text, buf)?;
                }
            }
            Self::Timings(timings) => {
                if let Some(value) = &entry.timings {
                    timings.write_rich_text(0, value, rich_text, buf)?;
                }
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
                if let Some(comment) = &entry.comment {
                    rich_text.append_plain_text(comment);
                }
            }
            Self::_ConnectionId => {
                if let Some(value) = &entry._connection_id {
                    rich_text.append_plain_text(value);
                }
            }
            Self::_SecurityState => {
                if let Some(value) = &entry._security_state {
                    write!(buf, "{value}")?;
                    rich_text.append_plain_text(&buf);
                }
            }
            Self::_Initiator(init) => {
                if let Some(value) = &entry._initiator {
                    init.write_rich_text(0, value, rich_text, buf)?;
                }
            }
            Self::_Priority => {
                if let Some(value) = &entry._priority {
                    write!(buf, "{value}")?;
                    rich_text.append_plain_text(&buf);
                }
            }
            Self::_ResourceType => {
                if let Some(value) = &entry._resource_type {
                    write!(buf, "{value}")?;
                    rich_text.append_plain_text(&buf);
                }
            }
            Self::_FromCache => {
                if let Some(value) = &entry._from_cache {
                    write!(buf, "{value}")?;
                    rich_text.append_plain_text(&buf);
                }
            }
        }

        Ok(())
    }

    fn parse<'a>(field: &'a str) -> Result<Self, ParserError<'a>> {
        if let Some((head, tail)) = field.split_once('.') {
            let map_err = |err: ParserError| ParserError::new(field, head.len() + 1 + err.index());
            if head.eq_ignore_ascii_case("request") {
                Ok(EntryField::Request(
                    RequestField::parse(tail).map_err(map_err)?
                ))
            } else if head.eq_ignore_ascii_case("response") {
                Ok(EntryField::Response(
                    ResponseField::parse(tail).map_err(map_err)?
                ))
            } else if head.eq_ignore_ascii_case("cache") {
                Ok(EntryField::Cache(
                    CacheField::parse(tail).map_err(map_err)?
                ))
            } else if head.eq_ignore_ascii_case("timings") {
                Ok(EntryField::Timings(
                    TimingsField::parse(tail).map_err(map_err)?
                ))
            } else if head.eq_ignore_ascii_case("_initiator") {
                Ok(EntryField::_Initiator(
                    InitiatorField::parse(tail).map_err(map_err)?
                ))
            } else {
                Err(ParserError::new(field, 0))
            }
        } else if field.eq_ignore_ascii_case("index") {
            Ok(EntryField::Index)
        } else if field.eq_ignore_ascii_case("startedDateTime") {
            Ok(EntryField::StartedDateTime)
        } else if field.eq_ignore_ascii_case("time") {
            Ok(EntryField::Time)
        } else if field.eq_ignore_ascii_case("serverIpAddress") {
            Ok(EntryField::ServerIpAddress)
        } else if field.eq_ignore_ascii_case("connection") {
            Ok(EntryField::Connection)
        } else if field.eq_ignore_ascii_case("comment") {
            Ok(EntryField::Comment)
        } else if field.eq_ignore_ascii_case("_connectionId") {
            Ok(EntryField::_ConnectionId)
        } else if field.eq_ignore_ascii_case("_securityState") {
            Ok(EntryField::_SecurityState)
        } else if field.eq_ignore_ascii_case("_priority") {
            Ok(EntryField::_Priority)
        } else if field.eq_ignore_ascii_case("_resourceType") {
            Ok(EntryField::_ResourceType)
        } else if field.eq_ignore_ascii_case("_fromCache") {
            Ok(EntryField::_FromCache)
        } else {
            Err(ParserError::new(field, 0))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitiatorField {
    Type,
    Url,
    LineNumber,
}

impl std::fmt::Display for InitiatorField {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Type => "type".fmt(f),
            Self::Url  => "url".fmt(f),
            Self::LineNumber => "lineNumber".fmt(f),
        }
    }
}

impl<'a> TryFrom<&'a str> for InitiatorField {
    type Error = ParserError<'a>;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

impl Field for InitiatorField {
    type Value = Initiator;

    #[inline]
    fn header(&self) -> &str {
        match self {
            Self::Type => "Initiator Type",
            Self::Url  => "Initiator URL",
            Self::LineNumber => "Initiator Line",
        }
    }

    #[inline]
    fn align(&self) -> Align {
        match self {
            Self::Type => Align::Left,
            Self::Url  => Align::Left,
            Self::LineNumber => Align::Right,
        }
    }

    fn write_rich_text(&self, _index: usize, init: &Initiator, rich_text: &mut RichText, buf: &mut String) -> std::fmt::Result {
        match self {
            Self::Type => {
                match init {
                    Initiator::Script { .. } => {
                        rich_text.append_plain_text("script");
                    }
                    Initiator::Parser { .. } => {
                        rich_text.append_plain_text("parser");
                    }
                    Initiator::Other => {
                        rich_text.append_plain_text("other");
                    }
                }
            },
            Self::Url => {
                if let Initiator::Parser { url, .. } = init {
                    write!(buf, "{url}")?;
                    rich_text.append_plain_text(buf);
                }
            },
            Self::LineNumber => {
                if let Initiator::Parser { line_number, .. } = init {
                    write!(buf, "{line_number}")?;
                    rich_text.append_plain_text(buf);
                }
            },
        }

        Ok(())
    }

    fn parse<'a>(field: &'a str) -> Result<Self, ParserError<'a>> {
        if field.eq_ignore_ascii_case("type") {
            Ok(Self::Type)
        } else if field.eq_ignore_ascii_case("url") {
            Ok(Self::Url)
        } else if field.eq_ignore_ascii_case("lineNumber") {
            Ok(Self::LineNumber)
        } else {
            Err(ParserError::new(field, 0))
        }
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

impl std::fmt::Display for RequestField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Method      => "method".fmt(f),
            Self::Url         => "url".fmt(f),
            Self::Scheme      => "scheme".fmt(f),
            Self::Host        => "host".fmt(f),
            Self::Port        => "port".fmt(f),
            Self::Domain      => "domain".fmt(f),
            Self::Path        => "path".fmt(f),
            Self::Query       => "query".fmt(f),
            Self::Fragment    => "fragment".fmt(f),
            Self::HttpVersion => "httpVersion".fmt(f),
            Self::HeadersSize => "header ize".fmt(f),
            Self::BodySize    => "bodySize".fmt(f),
            Self::Comment     => "comment".fmt(f),
        }
    }
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

impl<'a> TryFrom<&'a str> for RequestField {
    type Error = ParserError<'a>;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

impl Field for RequestField {
    type Value = Request;

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
            Self::HeadersSize => "Request Headers Size",
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

    fn write_rich_text(&self, _index: usize, request: &Request, rich_text: &mut RichText, buf: &mut String) -> std::fmt::Result {
        match self {
            Self::Method => {
                let style = RichTextStyle::build().foreground(get_method_color(&request.method)).into_inner();
                rich_text.append_text(&style, &request.method);
            },
            Self::Url => {
                write!(buf, "{}", request.url)?;
                rich_text.append_plain_text(buf);
            },
            Self::Scheme => {
                rich_text.append_plain_text(request.url.scheme());
            },
            Self::Host => {
                if let Some(host) = request.url.host_str() {
                    rich_text.append_plain_text(host);
                }
            },
            Self::Port => {
                if let Some(port) = request.url.port() {
                    write!(buf, "{}", port)?;
                    rich_text.append_plain_text(buf);
                }
            },
            Self::Domain => {
                if let Some(domain) = request.url.domain() {
                    rich_text.append_plain_text(domain);
                }
            },
            Self::Path => {
                rich_text.append_plain_text(request.url.path());
            },
            Self::Query => {
                if let Some(query) = request.url.query() {
                    rich_text.append_plain_text(query);
                }
            },
            Self::Fragment => {
                if let Some(fragment) = request.url.fragment() {
                    rich_text.append_plain_text(fragment);
                }
            },
            Self::HttpVersion => {
                rich_text.append_plain_text(&request.http_version);
            },
            Self::HeadersSize => {
                write!(buf, "{}", request.headers_size)?;
                rich_text.append_plain_text(buf);
            },
            Self::BodySize => {
                write!(buf, "{}", request.body_size)?;
                rich_text.append_plain_text(buf);
            },
            Self::Comment => {
                if let Some(comment) = &request.comment {
                    rich_text.append_plain_text(comment);
                }
            },
        }

        Ok(())
    }

    fn parse<'a>(field: &'a str) -> Result<Self, ParserError<'a>> {
        if field.eq_ignore_ascii_case("method") {
            Ok(Self::Method)
        } else if field.eq_ignore_ascii_case("url") {
            Ok(Self::Url)
        } else if field.eq_ignore_ascii_case("scheme") {
            Ok(Self::Scheme)
        } else if field.eq_ignore_ascii_case("host") {
            Ok(Self::Host)
        } else if field.eq_ignore_ascii_case("port") {
            Ok(Self::Port)
        } else if field.eq_ignore_ascii_case("domain") {
            Ok(Self::Domain)
        } else if field.eq_ignore_ascii_case("path") {
            Ok(Self::Path)
        } else if field.eq_ignore_ascii_case("query") {
            Ok(Self::Query)
        } else if field.eq_ignore_ascii_case("fragment") {
            Ok(Self::Fragment)
        } else if field.eq_ignore_ascii_case("httpVersion") {
            Ok(Self::HttpVersion)
        } else if field.eq_ignore_ascii_case("headersSize") {
            Ok(Self::HeadersSize)
        } else if field.eq_ignore_ascii_case("bodySize") {
            Ok(Self::BodySize)
        } else if field.eq_ignore_ascii_case("comment") {
            Ok(Self::Comment)
        } else {
            Err(ParserError::new(field, 0))
        }
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
    _TransferSize,
    _Error,
    _FetchedViaServiceWorker,
}

fn get_staus_color(status: u32) -> Color {
    if status < 100 {
        Color::Color16(Color16::Magenta)
    } else if status < 200 {
        Color::Color16(Color16::Cyan)
    } else if status < 300 {
        Color::Color16(Color16::Green)
    } else if status < 400 {
        Color::Color16(Color16::Blue)
    } else if status < 500 {
        Color::Color16(Color16::Yellow)
    } else if status < 600 {
        Color::Color16(Color16::Red)
    } else {
        Color::Color16(Color16::Magenta)
    }
}

impl std::fmt::Display for ResponseField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Status           => "status".fmt(f),
            Self::StatusText       => "statusText".fmt(f),
            Self::RedirectUrl      => "redirectURL".fmt(f),
            Self::Content(content) => write!(f, "content.{content}"),
            Self::HeadersSize      => "headerSize".fmt(f),
            Self::BodySize         => "bodySize".fmt(f),
            Self::Comment          => "comment".fmt(f),
            Self::_TransferSize    => "_transferSize".fmt(f),
            Self::_Error           => "_error".fmt(f),
            Self::_FetchedViaServiceWorker => "_fetchedViaServiceWorker".fmt(f),
        }
    }
}

impl<'a> TryFrom<&'a str> for ResponseField {
    type Error = ParserError<'a>;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

const ERROR_STYLE: RichTextStyle = RichTextStyle {
    foreground: Color::Color16(Color16::Red),
    ..DEFAULT_STYLE
};

impl Field for ResponseField {
    type Value = Response;

    #[inline]
    fn header(&self) -> &str {
        match self {
            Self::Status           => "Status",
            Self::StatusText       => "Status Text",
            Self::RedirectUrl      => "Redirect URL",
            Self::Content(content) => content.header(),
            Self::HeadersSize      => "Response Headers Size",
            Self::BodySize         => "Response Body Size",
            Self::Comment          => "Response Comment",
            Self::_TransferSize    => "Transfer Size",
            Self::_Error           => "Error",
            Self::_FetchedViaServiceWorker => "Fetched Via Service Worker",
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
            Self::_TransferSize    => Align::Right,
            Self::_Error           => Align::Left,
            Self::_FetchedViaServiceWorker => Align::Left,
        }
    }

    fn write_rich_text(&self, _index: usize, response: &Response, rich_text: &mut RichText, buf: &mut String) -> std::fmt::Result {
        match self {
            Self::Status => {
                let style = RichTextStyle::build().foreground(get_staus_color(response.status)).into();
                write!(buf, "{}", response.status)?;
                rich_text.append_text(&style, buf);
            },
            Self::StatusText => {
                let style = RichTextStyle::build().foreground(get_staus_color(response.status)).into();
                rich_text.append_text(&style, &response.status_text);
            },
            Self::RedirectUrl => {
                if let Some(redirect_url) = &response.redirect_url {
                    write!(buf, "{}", redirect_url)?;
                    rich_text.append_plain_text(buf);
                }
            },
            Self::Content(content) => {
                if let Some(value) = &response.content {
                    content.write_rich_text(0, value, rich_text, buf)?;
                }
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
            Self::_TransferSize => {
                write!(buf, "{}", response._transfer_size)?;
                rich_text.append_plain_text(buf);
            },
            Self::_Error => {
                if let Some(error) = &response._error {
                    rich_text.append_text(&ERROR_STYLE, error);
                }
            },
            Self::_FetchedViaServiceWorker => {
                if let &Some(value) = &response._fetched_via_service_worker {
                    rich_text.append_plain_text(if value { "true" } else { "false" });
                }
            },
        }

        Ok(())
    }

    fn parse<'a>(field: &'a str) -> Result<Self, ParserError<'a>> {
        if let Some((head, tail)) = field.split_once('.') {
            let map_err = |err: ParserError| ParserError::new(field, head.len() + 1 + err.index());
            if head.eq_ignore_ascii_case("content") {
                Ok(ResponseField::Content(
                    ContentField::parse(tail).map_err(map_err)?
                ))
            } else {
                Err(ParserError::new(field, 0))
            }
        } else if field.eq_ignore_ascii_case("status") {
            Ok(Self::Status)
        } else if field.eq_ignore_ascii_case("statusText") {
            Ok(Self::StatusText)
        } else if field.eq_ignore_ascii_case("redirectUrl") {
            Ok(Self::RedirectUrl)
        } else if field.eq_ignore_ascii_case("headersSize") {
            Ok(Self::HeadersSize)
        } else if field.eq_ignore_ascii_case("bodySize") {
            Ok(Self::BodySize)
        } else if field.eq_ignore_ascii_case("comment") {
            Ok(Self::Comment)
        } else if field.eq_ignore_ascii_case("_transferSize") {
            Ok(Self::_TransferSize)
        } else if field.eq_ignore_ascii_case("_error") {
            Ok(Self::_Error)
        } else if field.eq_ignore_ascii_case("_fetchedViaServiceWorker") {
            Ok(Self::_FetchedViaServiceWorker)
        } else {
            Err(ParserError::new(field, 0))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentField {
    Size,
    Compression,
    MimeType,
    Text,
    Encoding,
    Comment,
}

impl std::fmt::Display for ContentField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Size        => "size".fmt(f),
            Self::Compression => "compression".fmt(f),
            Self::MimeType    => "mimeType".fmt(f),
            Self::Text        => "text".fmt(f),
            Self::Encoding    => "encoding".fmt(f),
            Self::Comment     => "comment".fmt(f),
        }
    }
}

impl<'a> TryFrom<&'a str> for ContentField {
    type Error = ParserError<'a>;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

impl Field for ContentField {
    type Value = Content;

    #[inline]
    fn header(&self) -> &str {
        match self {
            Self::Size        => "Content Size",
            Self::Compression => "Compression",
            Self::MimeType    => "Mime Type",
            Self::Text        => "Content Text",
            Self::Encoding    => "Encoding",
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
            Self::Encoding    => Align::Left,
            Self::Comment     => Align::Left,
        }
    }

    fn write_rich_text(&self, _index: usize, content: &Content, rich_text: &mut RichText, buf: &mut String) -> std::fmt::Result {
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
                    if let Some(mime_type) = &content.mime_type {
                        let mime_type = mime_type.split(';').next().unwrap_or(&mime_type);

                        if mime_type == "text/html" || mime_type.ends_with("+xml") {
                            // TODO
                            rich_text.append_plain_text(text);
                        } else if mime_type == "application/json" || mime_type == "test/javascript" || mime_type.ends_with("+json") {
                            colorize_json(text, rich_text);
                        } else {
                            rich_text.append_plain_text(text);
                        }
                    } else {
                        rich_text.append_plain_text(text);
                    }
                }
            }
            Self::Encoding => {
                if let Some(encoding) = &content.encoding {
                    rich_text.append_plain_text(encoding);
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

    fn parse<'a>(field: &'a str) -> Result<Self, ParserError<'a>> {
        if field.eq_ignore_ascii_case("size") {
            Ok(Self::Size)
        } else if field.eq_ignore_ascii_case("compression") {
            Ok(Self::Compression)
        } else if field.eq_ignore_ascii_case("mimeType") {
            Ok(Self::MimeType)
        } else if field.eq_ignore_ascii_case("text") {
            Ok(Self::Text)
        } else if field.eq_ignore_ascii_case("encoding") {
            Ok(Self::Encoding)
        } else if field.eq_ignore_ascii_case("comment") {
            Ok(Self::Comment)
        } else {
            Err(ParserError::new(field, 0))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheField {
    BeforeRequest(CacheStateField),
    AfterRequest(CacheStateField),
    Comment,
}

impl std::fmt::Display for CacheField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BeforeRequest(value) => write!(f, "beforeRequest.{value}"),
            Self::AfterRequest(value)  => write!(f, "afterRequest.{value}"),
            Self::Comment              => "comment".fmt(f),
        }
    }
}

impl<'a> TryFrom<&'a str> for CacheField {
    type Error = ParserError<'a>;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

impl Field for CacheField {
    type Value = Cache;

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

    fn write_rich_text(&self, _index: usize, cache: &Cache, rich_text: &mut RichText, buf: &mut String) -> std::fmt::Result {
        match self {
            Self::BeforeRequest(state_field) => {
                if let Some(Some(state)) = &cache.before_request {
                    state_field.write_rich_text(0, state, rich_text, buf)?;
                }
            },
            Self::AfterRequest(state_field) => {
                if let Some(Some(state)) = &cache.after_request {
                    state_field.write_rich_text(0, state, rich_text, buf)?;
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

    fn parse<'a>(field: &'a str) -> Result<Self, ParserError<'a>> {
        if let Some((head, tail)) = field.split_once('.') {
            let map_err = |err: ParserError| ParserError::new(field, head.len() + 1 + err.index());
            if head.eq_ignore_ascii_case("beforeRequest") {
                Ok(CacheField::BeforeRequest(
                    CacheStateField::parse(tail).map_err(map_err)?
                ))
            } else if head.eq_ignore_ascii_case("afterRequest") {
                Ok(CacheField::AfterRequest(
                    CacheStateField::parse(tail).map_err(map_err)?
                ))
            } else {
                Err(ParserError::new(field, 0))
            }
        } else if field.eq_ignore_ascii_case("comment") {
            Ok(Self::Comment)
        } else {
            Err(ParserError::new(field, 0))
        }
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

impl std::fmt::Display for CacheStateField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expires    => "expires".fmt(f),
            Self::LastAccess => "lastAccess".fmt(f),
            Self::ETag       => "etag".fmt(f),
            Self::HitCount   => "hitCount".fmt(f),
            Self::Comment    => "comment".fmt(f),
        }
    }
}

impl<'a> TryFrom<&'a str> for CacheStateField {
    type Error = ParserError<'a>;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

impl Field for CacheStateField {
    type Value = CacheState;

    fn write_rich_text(&self, _index: usize, state: &CacheState, rich_text: &mut RichText, buf: &mut String) -> std::fmt::Result {
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
    fn header(&self) -> &str {
        match self {
            Self::Expires    => "Expires",
            Self::LastAccess => "Last Access",
            Self::ETag       => "E-Tag",
            Self::HitCount   => "Hit Count",
            Self::Comment    => "Cache State Comment",
        }
    }

    #[inline]
    fn align(&self) -> Align {
        match self {
            Self::Expires    => Align::Right,
            Self::LastAccess => Align::Right,
            Self::ETag       => Align::Left,
            Self::HitCount   => Align::Right,
            Self::Comment    => Align::Left,
        }
    }

    fn parse<'a>(field: &'a str) -> Result<Self, ParserError<'a>> {
        if field.eq_ignore_ascii_case("expires") {
            Ok(Self::Expires)
        } else if field.eq_ignore_ascii_case("lastAccess") {
            Ok(Self::LastAccess)
        } else if field.eq_ignore_ascii_case("eTag") {
            Ok(Self::ETag)
        } else if field.eq_ignore_ascii_case("hitCount") {
            Ok(Self::HitCount)
        } else if field.eq_ignore_ascii_case("comment") {
            Ok(Self::Comment)
        } else {
            Err(ParserError::new(field, 0))
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
    _BlockedQueueing,
    _WorkerStart,
    _WorkerReady,
    _WorkerFetchStart,
    _WorkerRespondWithSettled,
}

impl std::fmt::Display for TimingsField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blocked => "blocked".fmt(f),
            Self::DNS     => "dns".fmt(f),
            Self::Connect => "connect".fmt(f),
            Self::Send    => "send".fmt(f),
            Self::Wait    => "wait".fmt(f),
            Self::Receive => "receive".fmt(f),
            Self::SSL     => "ssl".fmt(f),
            Self::Comment => "comment".fmt(f),
            Self::_BlockedQueueing  => "_blocked_queueing".fmt(f),
            Self::_WorkerStart      => "_workerStart".fmt(f),
            Self::_WorkerReady      => "_workerReady".fmt(f),
            Self::_WorkerFetchStart => "_workerFetchStart".fmt(f),
            Self::_WorkerRespondWithSettled => "_workerRespondWithSettled".fmt(f),
        }
    }
}

impl<'a> TryFrom<&'a str> for TimingsField {
    type Error = ParserError<'a>;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

impl Field for TimingsField {
    type Value = Timings;

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
            Self::_BlockedQueueing  => "Blocked Queueing",
            Self::_WorkerStart      => "Worker Start",
            Self::_WorkerReady      => "Worker Ready",
            Self::_WorkerFetchStart => "Worker Fetch Start",
            Self::_WorkerRespondWithSettled => "Worker Respond With Settled",
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
            Self::_BlockedQueueing  => Align::Right,
            Self::_WorkerStart      => Align::Right,
            Self::_WorkerReady      => Align::Right,
            Self::_WorkerFetchStart => Align::Right,
            Self::_WorkerRespondWithSettled => Align::Right,
        }
    }

    fn write_rich_text(&self, _index: usize, timings: &Timings, rich_text: &mut RichText, buf: &mut String) -> std::fmt::Result {
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
            Self::_BlockedQueueing => {
                write!(buf, "{}", timings._blocked_queueing)?;
                rich_text.append_plain_text(buf);
            }
            Self::_WorkerStart => {
                write!(buf, "{}", timings._worker_start)?;
                rich_text.append_plain_text(buf);
            }
            Self::_WorkerReady => {
                write!(buf, "{}", timings._worker_ready)?;
                rich_text.append_plain_text(buf);
            }
            Self::_WorkerFetchStart => {
                write!(buf, "{}", timings._worker_fetch_start)?;
                rich_text.append_plain_text(buf);
            }
            Self::_WorkerRespondWithSettled => {
                write!(buf, "{}", timings._worker_respond_with_settled)?;
                rich_text.append_plain_text(buf);
            }
        }

        Ok(())
    }

    fn parse<'a>(field: &'a str) -> Result<Self, ParserError<'a>> {
        if field.eq_ignore_ascii_case("blocked") {
            Ok(Self::Blocked)
        } else if field.eq_ignore_ascii_case("dns") {
            Ok(Self::DNS)
        } else if field.eq_ignore_ascii_case("connect") {
            Ok(Self::Connect)
        } else if field.eq_ignore_ascii_case("send") {
            Ok(Self::Send)
        } else if field.eq_ignore_ascii_case("wait") {
            Ok(Self::Wait)
        } else if field.eq_ignore_ascii_case("receive") {
            Ok(Self::Receive)
        } else if field.eq_ignore_ascii_case("ssl") {
            Ok(Self::SSL)
        } else if field.eq_ignore_ascii_case("_blocked_queueing") || field.eq_ignore_ascii_case("_blockedQueueing") {
            Ok(Self::_BlockedQueueing)
        } else if field.eq_ignore_ascii_case("_workerStart") {
            Ok(Self::_WorkerStart)
        } else if field.eq_ignore_ascii_case("_workerReady") {
            Ok(Self::_WorkerReady)
        } else if field.eq_ignore_ascii_case("_workerFetchStart") {
            Ok(Self::_WorkerFetchStart)
        } else if field.eq_ignore_ascii_case("_workerRespondWithSettled") {
            Ok(Self::_WorkerRespondWithSettled)
        } else if field.eq_ignore_ascii_case("comment") {
            Ok(Self::Comment)
        } else {
            Err(ParserError::new(field, 0))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageField {
    Index,
    StartedDateTime,
    Id,
    Title,
    PageTimings(PageTimingsField),
    Comment,
}

impl std::fmt::Display for PageField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Index           => "index".fmt(f),
            Self::StartedDateTime => "startedDateTime".fmt(f),
            Self::Id              => "id".fmt(f),
            Self::Title           => "title".fmt(f),
            Self::PageTimings(timings) => write!(f, "pageTimings.{timings}"),
            Self::Comment         => "comment".fmt(f),
        }
    }
}

impl<'a> TryFrom<&'a str> for PageField {
    type Error = ParserError<'a>;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

impl Field for PageField {
    type Value = Page;

    #[inline]
    fn header(&self) -> &str {
        match self {
            Self::Index           => "Index",
            Self::StartedDateTime => "Started At",
            Self::Id              => "Id",
            Self::Title           => "Title",
            Self::PageTimings(timings) => timings.header(),
            Self::Comment         => "Comment",
        }
    }

    #[inline]
    fn align(&self) -> Align {
        match self {
            Self::Index           => Align::Right,
            Self::StartedDateTime => Align::Left,
            Self::Id              => Align::Left,
            Self::Title           => Align::Left,
            Self::PageTimings(timings) => timings.align(),
            Self::Comment         => Align::Left,
        }
    }

    fn write_rich_text(&self, index: usize, page: &Page, rich_text: &mut RichText, buf: &mut String) -> std::fmt::Result {
        match self {
            Self::Index => {
                write!(buf, "{index}")?;
                rich_text.append_plain_text(buf);
            }
            Self::StartedDateTime => {
                write!(buf, "{}", page.started_date_time)?;
                rich_text.append_plain_text(buf);
            }
            Self::Id => {
                rich_text.append_plain_text(&page.id);
            }
            Self::Title => {
                rich_text.append_plain_text(&page.title);
            }
            Self::PageTimings(timings) => {
                if let Some(value) = &page.page_timings {
                    timings.write_rich_text(0, value, rich_text, buf)?;
                }
            }
            Self::Comment => {
                if let Some(connection) = &page.comment {
                    rich_text.append_plain_text(connection);
                }
            }
        }

        Ok(())
    }

    fn parse<'a>(field: &'a str) -> Result<Self, ParserError<'a>> {
        if let Some((head, tail)) = field.split_once('.') {
            let map_err = |err: ParserError| ParserError::new(field, head.len() + 1 + err.index());
            if head.eq_ignore_ascii_case("pageTimings") {
                Ok(PageField::PageTimings(
                    PageTimingsField::parse(tail).map_err(map_err)?
                ))
            } else {
                Err(ParserError::new(field, 0))
            }
        } else if field.eq_ignore_ascii_case("index") {
            Ok(Self::Index)
        } else if field.eq_ignore_ascii_case("startedDateTime") {
            Ok(Self::StartedDateTime)
        } else if field.eq_ignore_ascii_case("id") {
            Ok(Self::Id)
        } else if field.eq_ignore_ascii_case("title") {
            Ok(Self::Title)
        } else if field.eq_ignore_ascii_case("comment") {
            Ok(Self::Comment)
        } else {
            Err(ParserError::new(field, 0))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageTimingsField {
    OnContentLoad,
    OnLoad,
    Comment,
}

impl std::fmt::Display for PageTimingsField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OnContentLoad => "onContentLoad".fmt(f),
            Self::OnLoad        => "onLoad".fmt(f),
            Self::Comment       => "comment".fmt(f),
        }
    }
}

impl<'a> TryFrom<&'a str> for PageTimingsField {
    type Error = ParserError<'a>;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

impl Field for PageTimingsField {
    type Value = PageTiming;

    #[inline]
    fn header(&self) -> &str {
        match self {
            Self::OnContentLoad => "On Content Load",
            Self::OnLoad        => "On Load",
            Self::Comment       => "Comment",
        }
    }

    #[inline]
    fn align(&self) -> Align {
        match self {
            Self::OnContentLoad => Align::Left,
            Self::OnLoad        => Align::Left,
            Self::Comment       => Align::Left,
        }
    }

    fn write_rich_text(&self, _index: usize, timing: &PageTiming, rich_text: &mut RichText, buf: &mut String) -> std::fmt::Result {
        match self {
            Self::OnContentLoad => {
                if let Some(value) = &timing.on_content_load {
                    write!(buf, "{value}")?;
                    rich_text.append_plain_text(buf);
                }
            }
            Self::OnLoad => {
                if let Some(value) = &timing.on_load {
                    write!(buf, "{value}")?;
                    rich_text.append_plain_text(buf);
                }
            }
            Self::Comment => {
                if let Some(connection) = &timing.comment {
                    rich_text.append_plain_text(connection);
                }
            }
        }

        Ok(())
    }

    fn parse<'a>(field: &'a str) -> Result<Self, ParserError<'a>> {
        if field.eq_ignore_ascii_case("OnContentLoad") {
            Ok(Self::OnContentLoad)
        } else if field.eq_ignore_ascii_case("OnLoad") {
            Ok(Self::OnLoad)
        } else if field.eq_ignore_ascii_case("Comment") {
            Ok(Self::Comment)
        } else {
            Err(ParserError::new(field, 0))
        }
    }
}
