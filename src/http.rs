//! HTTP/HTTPS client API.
//!
//! Requires `http.enabled: true` in `manifest.yaml`.
//! Every URL must match an entry in `http.allowlist`.

use crate::abi::{call_host, raw};
use crate::types::SkillError;
use serde::{Deserialize, Serialize};

/// An ordered HTTP header name/value pair.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Header {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "value")]
    pub value: String,
}

impl Header {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Header { name: name.into(), value: value.into() }
    }
}

/// An HTTP cookie.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Cookie {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "value")]
    pub value: String,
    #[serde(rename = "domain", default, skip_serializing_if = "String::is_empty")]
    pub domain: String,
    #[serde(rename = "path", default, skip_serializing_if = "String::is_empty")]
    pub path: String,
    /// Unix timestamp; `0` = session cookie.
    #[serde(rename = "expires", default, skip_serializing_if = "is_zero_i64")]
    pub expires: i64,
    #[serde(rename = "secure", default, skip_serializing_if = "is_false")]
    pub secure: bool,
    #[serde(rename = "httponly", default, skip_serializing_if = "is_false")]
    pub http_only: bool,
}

/// Response returned by all HTTP calls.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HttpResponse {
    #[serde(rename = "status")]
    pub status: u32,
    #[serde(rename = "headers", default)]
    pub headers: Vec<Header>,
    #[serde(rename = "cookies", default)]
    pub cookies: Vec<Cookie>,
    #[serde(rename = "body", default, with = "crate::bytes_or_null")]
    pub body: Vec<u8>,
    #[serde(rename = "truncated", default)]
    pub truncated: bool,
    #[serde(rename = "error", default)]
    pub(crate) error: String,
}

/// Perform a GET request. Requires `http.enabled`.
///
/// - `timeout_ms = 0` -> 30 s default.
/// - `max_bytes = 0` -> manifest limit or 1 MiB default.
pub fn http_get(url: &str, timeout_ms: i64, max_bytes: i64) -> Result<HttpResponse, SkillError> {
    #[derive(Serialize, Default)]
    struct Req<'a> {
        url: &'a str,
        timeout_ms: i64,
        max_bytes: i64,
    }
    let resp: HttpResponse =
        call_host(raw::http_get, &Req { url, timeout_ms, max_bytes })?;
    if !resp.error.is_empty() {
        return Err(SkillError(resp.error));
    }
    Ok(resp)
}

/// Perform a POST request. Requires `http.enabled`.
///
/// `content_type` defaults to `"application/octet-stream"` when empty.
pub fn http_post(
    url: &str,
    body: Vec<u8>,
    content_type: &str,
    timeout_ms: i64,
    max_bytes: i64,
) -> Result<HttpResponse, SkillError> {
    #[derive(Serialize, Default)]
    struct Req<'a> {
        url: &'a str,
        #[serde(with = "crate::bytes_or_null")]
        body: Vec<u8>,
        content_type: &'a str,
        timeout_ms: i64,
        max_bytes: i64,
    }
    let resp: HttpResponse = call_host(
        raw::http_post,
        &Req { url, body, content_type, timeout_ms, max_bytes },
    )?;
    if !resp.error.is_empty() {
        return Err(SkillError(resp.error));
    }
    Ok(resp)
}

/// Options for [`http_request`].
#[derive(Debug, Clone, Default)]
pub struct HttpRequestOptions {
    /// HTTP method. Defaults to `"GET"`.
    pub method: String,
    pub url: String,
    /// Applied in listed order.
    pub headers: Vec<Header>,
    pub cookies: Vec<Cookie>,
    pub body: Vec<u8>,
    /// `0` -> 30 s default.
    pub timeout_ms: i64,
    /// `0` -> manifest limit (1 MiB default).
    pub max_bytes: i64,
    pub follow_redirects: bool,
    /// Enable the persistent per-skill cookie jar.
    pub use_jar: bool,
}

/// Perform a fully customised HTTP request with ordered headers,
/// persistent cookie jar, and full control over method, redirects and body.
/// Requires `http.enabled`.
pub fn http_request(opts: HttpRequestOptions) -> Result<HttpResponse, SkillError> {
    #[derive(Serialize, Default)]
    struct Req {
        method: String,
        url: String,
        headers: Vec<Header>,
        cookies: Vec<Cookie>,
        #[serde(with = "crate::bytes_or_null")]
        body: Vec<u8>,
        timeout_ms: i64,
        max_bytes: i64,
        follow_redirects: bool,
        use_jar: bool,
    }
    let resp: HttpResponse = call_host(
        raw::http_request,
        &Req {
            method: opts.method,
            url: opts.url,
            headers: opts.headers,
            cookies: opts.cookies,
            body: opts.body,
            timeout_ms: opts.timeout_ms,
            max_bytes: opts.max_bytes,
            follow_redirects: opts.follow_redirects,
            use_jar: opts.use_jar,
        },
    )?;
    if !resp.error.is_empty() {
        return Err(SkillError(resp.error));
    }
    Ok(resp)
}

/// Parse a JSON object string into an ordered `Vec<Header>`.
/// Preserves key insertion order from the source string.
pub fn headers_from_json(json: &str) -> Vec<Header> {
    let mut result = Vec::new();
    let bytes = json.as_bytes();
    let mut i = match json.find('{') {
        Some(p) => p + 1,
        None => return result,
    };
    while i < bytes.len() {
        if bytes[i] == b'}' { break; }
        if bytes[i] != b'"' { i += 1; continue; }
        i += 1; // skip opening '"'
        let key_end = find_quote(bytes, i);
        let key = &json[i..key_end];
        i = key_end + 1;
        while i < bytes.len() && bytes[i] != b'"' { i += 1; }
        if i >= bytes.len() { break; }
        i += 1; // skip opening '"'
        let val_end = find_quote(bytes, i);
        let val = &json[i..val_end];
        i = val_end + 1;
        result.push(Header::new(key, val));
    }
    result
}

fn find_quote(bytes: &[u8], start: usize) -> usize {
    let mut i = start;
    while i < bytes.len() {
        if bytes[i] == b'"' { return i; }
        if bytes[i] == b'\\' { i += 2; continue; }
        i += 1;
    }
    i
}

fn is_false(v: &bool) -> bool { !v }
fn is_zero_i64(v: &i64) -> bool { *v == 0 }
