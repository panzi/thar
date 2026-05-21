use serde::Deserialize;
use time::OffsetDateTime;

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
    pub pages: Option<Vec<Page>>,
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
    pub page_timings: Vec<PageTiming>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct PageTiming {
    #[serde(rename = "onContentLoad")]
    pub on_content_load: Option<i64>,
    #[serde(rename = "onLoad")]
    pub on_load: Option<i64>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Entry {
    pub pageref: Option<String>,
    #[serde(rename = "startedDateTime", with = "time::serde::iso8601")]
    pub started_date_time: OffsetDateTime,
    pub time: i64,
    pub request: Request,
    pub response: Option<Response>,
    pub cache: Option<Cache>,
    pub timings: Option<Timings>,
    #[serde(rename = "serverIPAddress")]
    pub server_ip_address: Option<String>,
    pub connection: Option<String>,
    pub comment: Option<String>,
}

#[inline]
fn unavailable_i64() -> i64 {
    -1
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Request {
    pub method: String,
    pub url: String,
    #[serde(rename = "httpVersion")]
    pub http_version: String,
    #[serde(default)]
    pub cookies: Vec<Cookie>,
    #[serde(default)]
    pub headers: Vec<Header>,
    #[serde(rename = "queryString")]
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
    #[serde(rename = "redirectURL")]
    pub redirect_url: Option<String>,
    #[serde(rename = "headersSize", default = "unavailable_i64")]
    pub headers_size: i64,
    #[serde(rename = "bodySize", default = "unavailable_i64")]
    pub body_size: i64,
    pub comment: Option<String>,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Content {
    pub size: u64,
    pub compression: Option<u64>,
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    pub text: Option<String>,
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
    #[serde(default = "unavailable_i64")]
    pub blocked: i64,
    #[serde(default = "unavailable_i64")]
    pub dns: i64,
    #[serde(default = "unavailable_i64")]
    pub connect: i64,
    #[serde(default = "unavailable_i64")]
    pub send: i64,
    #[serde(default = "unavailable_i64")]
    pub wait: i64,
    #[serde(default = "unavailable_i64")]
    pub receive: i64,
    #[serde(default = "unavailable_i64")]
    pub ssl: i64,
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
