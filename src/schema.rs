use serde::{Deserialize, Deserializer, de::Visitor};
use time::OffsetDateTime;
use url::Url;

// http://www.softwareishard.com/blog/har-12-spec/
#[derive(Deserialize, Clone, Default, Debug)]
pub struct HAR {
    pub log: Log,
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct Log {
    pub version: Option<String>,
    pub creator: Option<AppInfo>,
    pub browser: Option<AppInfo>,
    #[serde(default)]
    pub pages: Vec<Page>,
    #[serde(default)]
    pub entries: Vec<Entry>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub comment: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Page {
    #[serde(rename = "startedDateTime", with = "time::serde::iso8601")]
    pub started_date_time: OffsetDateTime,
    pub id: String,
    pub title: String,
    #[serde(rename = "pageTimings")]
    pub page_timings: Option<PageTiming>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct PageTiming {
    #[serde(rename = "onContentLoad")]
    pub on_content_load: Option<f64>,
    #[serde(rename = "onLoad")]
    pub on_load: Option<f64>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Entry {
    pub pageref: Option<String>,

    #[serde(rename = "startedDateTime", with = "time::serde::iso8601")]
    pub started_date_time: OffsetDateTime,

    #[serde(default = "unavailable_f64")]
    pub time: f64,
    pub request: Request,
    pub response: Option<Response>,
    pub cache: Option<Cache>,
    pub timings: Option<Timings>,

    /// Chrome extension.
    #[serde(rename = "_connectionId")]
    pub _connection_id: Option<String>,

    #[serde(rename = "serverIPAddress")]
    pub server_ip_address: Option<String>,
    pub connection: Option<String>,
    pub comment: Option<String>,

    /// Mozilla extension.
    #[serde(rename = "_securityState")]
    pub _security_state: Option<SecurityState>,

    /// Chrome extension.
    pub _initiator: Option<Initiator>,

    /// Chrome extension.
    pub _priority: Option<Priority>,

    /// Chrome extension.
    #[serde(rename = "_resourceType")]
    pub _resource_type: Option<ResourceType>,

    /// Chrome extension.
    #[serde(rename = "_fromCache")]
    pub _from_cache: Option<CacheSource>,
}

/// Mozilla extension.
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SecurityState {
    Insecure,
    Broken,
    Secure,
}

impl std::fmt::Display for SecurityState {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Insecure => "insecure".fmt(f),
            Self::Broken   => "broken".fmt(f),
            Self::Secure   => "secure".fmt(f),
        }
    }
}

/// Chrome extension.
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    Document,
    StyleSheet,
    Image,
    Media,
    Font,
    Script,
    XHR,
    Fetch,
    WebSocket,
    Manifest,
    Ping,
    Other,
}

impl std::fmt::Display for ResourceType {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Document   => "document".fmt(f),
            Self::StyleSheet => "stylesheet".fmt(f),
            Self::Image      => "image".fmt(f),
            Self::Media      => "media".fmt(f),
            Self::Font       => "font".fmt(f),
            Self::Script     => "script".fmt(f),
            Self::XHR        => "xhr".fmt(f),
            Self::Fetch      => "fetch".fmt(f),
            Self::WebSocket  => "websocket".fmt(f),
            Self::Manifest   => "manifest".fmt(f),
            Self::Ping       => "ping".fmt(f),
            Self::Other      => "other".fmt(f),
        }
    }
}

/// Chrome extension.
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum CacheSource {
    Disk,
    Memory,
}

impl std::fmt::Display for CacheSource {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disk   => "disk".fmt(f),
            Self::Memory => "memory".fmt(f),
        }
    }
}

/// Chrome extension.
#[derive(Deserialize, Clone, Debug)]
pub enum Priority {
    VeryHigh,
    High,
    Medium,
    Low,
    VeryLow,
}

impl std::fmt::Display for Priority {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VeryHigh => "VeryHigh".fmt(f),
            Self::High     => "High".fmt(f),
            Self::Medium   => "Medium".fmt(f),
            Self::Low      => "Low".fmt(f),
            Self::VeryLow  => "VeryLow".fmt(f),
        }
    }
}

/// Chromium extension.
#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum Initiator {
    #[serde(rename = "parser")]
    Parser {
        url: Url,
        #[serde(rename = "lineNumber", default = "unavailable_i64")]
        line_number: i64,
    },
    #[serde(rename = "script")]
    Script {
        stack: Option<Stack>
    },
    #[serde(rename = "other")]
    Other,
}

/// Chromium extension.
#[derive(Deserialize, Clone, Debug)]
pub struct Stack {
    #[serde(rename = "callFrames", default)]
    pub call_frames: Vec<CallFrame>,
}

/// Chromium extension.
#[derive(Deserialize, Clone, Debug)]
pub struct CallFrame {
    #[serde(rename = "functionName", default)]
    pub function_name: String,

    #[serde(rename = "scriptId", default)]
    pub script_id: String,
    pub url: Url,

    #[serde(rename = "lineNumber", default = "unavailable_i64")]
    pub line_number: i64,

    #[serde(rename = "columnNumber", default = "unavailable_i64")]
    pub column_number: i64,
}

#[inline]
fn unavailable_i64() -> i64 {
    -1
}

#[inline]
fn unavailable_f64() -> f64 {
    -1.0
}

#[derive(Deserialize, Clone, Debug)]
pub struct Request {
    pub method: String,
    pub url: Url,
    #[serde(rename = "httpVersion")]
    pub http_version: String,
    #[serde(default)]
    pub cookies: Vec<Cookie>,
    #[serde(default)]
    pub headers: Vec<Header>,
    #[serde(rename = "queryString", default)]
    pub query_string: Vec<QueryParam>,
    #[serde(rename = "postData")]
    pub post_data: Option<PostData>,
    #[serde(rename = "headersSize", default = "unavailable_i64")]
    pub headers_size: i64,
    #[serde(rename = "bodySize", default = "unavailable_i64")]
    pub body_size: i64,
    pub comment: Option<String>,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Response {
    pub status: u32,

    #[serde(rename = "statusText")]
    pub status_text: String,

    #[serde(rename = "httpVersion")]
    pub http_version: String,

    #[serde(default)]
    pub cookies: Vec<Cookie>,

    #[serde(default)]
    pub headers: Vec<Header>,

    pub content: Option<Content>,

    #[serde(rename = "redirectURL", deserialize_with = "deserialize_optional_url", default)]
    pub redirect_url: Option<Url>,

    #[serde(rename = "headersSize", default = "unavailable_i64")]
    pub headers_size: i64,

    #[serde(rename = "bodySize", default = "unavailable_i64")]
    pub body_size: i64,

    pub comment: Option<String>,

    /// Chrome extension.
    #[serde(rename = "_transferSize", default = "unavailable_i64")]
    pub _transfer_size: i64,

    /// Chrome extension.
    pub _error: Option<String>,

    /// Chrome extension.
    #[serde(rename = "_fetchedViaServiceWorker")]
    pub _fetched_via_service_worker: Option<bool>,
}

fn deserialize_optional_url<'de, D>(deserializer: D) -> Result<Option<Url>, D::Error>
where D: Deserializer<'de> {
    struct OptionalUrlVisitor;

    impl<'de> Visitor<'de> for OptionalUrlVisitor {
        type Value = Option<Url>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("URL")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where E: serde::de::Error {
            Ok(None)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where E: serde::de::Error {
            if v.is_empty() {
                return Ok(None);
            }

            match Url::parse(v) {
                Ok(url) => Ok(Some(url)),
                Err(err) => Err(E::custom(err.to_string()))
            }
        }
    }

    deserializer.deserialize_any(OptionalUrlVisitor)
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Content {
    pub size: u64,
    pub compression: Option<u64>,
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    pub text: Option<String>,
    pub encoding: Option<String>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Cache {
    #[serde(rename = "beforeRequest")]
    pub before_request: Option<Option<CacheState>>,
    #[serde(rename = "afterRequest")]
    pub after_request: Option<Option<CacheState>>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CacheState {
    #[serde(rename = "expires", with = "time::serde::iso8601::option")]
    pub expires: Option<OffsetDateTime>,
    #[serde(rename = "lastAccess", with = "time::serde::iso8601")]
    pub last_access: OffsetDateTime,
    #[serde(rename = "eTag")]
    pub e_tag: Option<String>,
    #[serde(rename = "hitCount")]
    pub hit_count: u64,
    pub comment: Option<String>,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Timings {
    #[serde(default = "unavailable_f64")]
    pub blocked: f64,

    #[serde(default = "unavailable_f64")]
    pub dns: f64,

    #[serde(default = "unavailable_f64")]
    pub connect: f64,

    #[serde(default = "unavailable_f64")]
    pub send: f64,

    #[serde(default = "unavailable_f64")]
    pub wait: f64,

    #[serde(default = "unavailable_f64")]
    pub receive: f64,

    #[serde(default = "unavailable_f64")]
    pub ssl: f64,

    #[serde(default = "unavailable_f64")]
    pub _blocked_queueing: f64,

    /// Chrome extension.
    #[serde(rename = "_workerStart", default = "unavailable_f64")]
    pub _worker_start: f64,

    /// Chrome extension.
    #[serde(rename = "_workerReady", default = "unavailable_f64")]
    pub _worker_ready: f64,

    /// Chrome extension.
    #[serde(rename = "_workerFetchStart", default = "unavailable_f64")]
    pub _worker_fetch_start: f64,

    /// Chrome extension.
    #[serde(rename = "_workerRespondWithSettled", default = "unavailable_f64")]
    pub _worker_respond_with_settled: f64,

    pub comment: Option<String>,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub path: Option<String>,
    pub domain: Option<String>,
    #[serde(with = "time::serde::iso8601::option", default)]
    pub expires: Option<OffsetDateTime>,
    #[serde(rename = "httpOnly", default)]
    pub http_only: bool,
    #[serde(default)]
    pub secure: bool,
    pub comment: Option<String>,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct PostData {
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub params: Vec<PostParam>,
    pub text: Option<String>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct QueryParam {
    pub name: String,
    pub value: String,
    pub comment: Option<String>,
}

pub type Header = QueryParam;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct PostParam {
    pub name: String,
    pub value: Option<String>,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
    pub comment: Option<String>,
}
