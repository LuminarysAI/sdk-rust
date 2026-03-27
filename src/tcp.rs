//! TCP / TLS connection API. Requires `tcp.enabled`.

use crate::abi::{call_host, raw};
use crate::types::SkillError;
use serde::{Deserialize, Serialize};

/// Classifies a TCP connection error.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorKind {
    /// Data event — no error.
    #[default]
    #[serde(rename = "")]
    None,
    /// Remote peer closed the connection gracefully.
    Eof,
    /// Connection reset by peer.
    Reset,
    /// Read deadline exceeded.
    Timeout,
    /// TLS handshake or record-layer error.
    Tls,
    /// Generic I/O error.
    Io,
}

/// Payload delivered to the skill's TCP read callback.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConnEvent {
    #[serde(rename = "conn_id")]
    pub conn_id: String,
    #[serde(rename = "data", default, with = "crate::bytes_or_null", skip_serializing_if = "Vec::is_empty")]
    pub data: Vec<u8>,
    #[serde(rename = "error_kind", default)]
    pub error_kind: ErrorKind,
    #[serde(rename = "error_msg", default, skip_serializing_if = "String::is_empty")]
    pub error_msg: String,
}

/// Deserialise a [`ConnEvent`] from the raw payload.
pub fn unmarshal_conn_event(payload: &[u8]) -> Result<ConnEvent, SkillError> {
    Ok(rmp_serde::from_slice(payload)?)
}

/// Options for [`tcp_connect`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TcpConnectOptions {
    /// Host:port to connect to (required).
    #[serde(rename = "addr")]
    pub addr: String,
    /// Callback method name called with [`ConnEvent`] on reads. `""` = drain silently.
    #[serde(rename = "callback", default)]
    pub callback: String,
    /// Use TLS encryption.
    #[serde(rename = "tls", default, skip_serializing_if = "is_false")]
    pub tls: bool,
    /// Skip TLS certificate verification (dev only).
    #[serde(rename = "insecure", default, skip_serializing_if = "is_false")]
    pub insecure: bool,
    /// Override TLS SNI hostname.
    #[serde(rename = "server_name", default, skip_serializing_if = "String::is_empty")]
    pub server_name: String,
    /// Dial timeout in milliseconds. `0` = 30 s default.
    #[serde(rename = "timeout_ms", default)]
    pub timeout_ms: i64,
}

/// Perform a synchronous TCP request: connect, send, read, close. Requires `tcp.enabled`.
pub fn tcp_request(opts: &crate::types::TcpRequestOptions) -> Result<crate::types::TcpRequestResult, SkillError> {
    let resp: crate::types::TcpRequestResult = call_host(raw::tcp_request, opts)?;
    if !resp.error.is_empty() {
        return Err(SkillError(resp.error));
    }
    Ok(resp)
}

/// Dial a TCP connection (plain or TLS). Requires `tcp.enabled`.
pub fn tcp_connect(opts: TcpConnectOptions) -> Result<String, SkillError> {
    extract_conn_id(
        call_host(raw::tcp_connect, &opts),
        "tcp_connect",
    )
}

/// Update the read callback for an existing connection.
pub fn tcp_set_callback(conn_id: &str, callback: &str) -> Result<(), SkillError> {
    #[derive(Serialize, Default)]
    struct Req<'a> { conn_id: &'a str, callback: &'a str }
    extract_error(call_host(raw::tcp_set_callback, &Req { conn_id, callback }), "tcp_set_callback")
}

/// Send data over an existing connection.
pub fn tcp_write(conn_id: &str, data: Vec<u8>) -> Result<(), SkillError> {
    #[derive(Serialize, Default)]
    struct Req<'a> {
        conn_id: &'a str,
        #[serde(with = "crate::bytes_or_null")]
        data: Vec<u8>,
    }
    extract_error(call_host(raw::tcp_write, &Req { conn_id, data }), "tcp_write")
}

/// Close the connection. Idempotent.
pub fn tcp_close(conn_id: &str) -> Result<(), SkillError> {
    #[derive(Serialize, Default)]
    struct Req<'a> { conn_id: &'a str }
    extract_error(call_host(raw::tcp_close, &Req { conn_id }), "tcp_close")
}

#[derive(Deserialize, Default)]
struct ConnResp {
    #[serde(default)]
    conn_id: String,
    #[serde(default)]
    error: String,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct ErrResp {
    #[serde(default)]
    error: String,
}

fn is_false(v: &bool) -> bool { !v }

fn extract_conn_id(result: Result<ConnResp, SkillError>, op: &str) -> Result<String, SkillError> {
    match result {
        Err(e) => Err(SkillError(format!("{op}: {e}"))),
        Ok(r) if !r.error.is_empty() => Err(SkillError(r.error)),
        Ok(r) => Ok(r.conn_id),
    }
}

fn extract_error(result: Result<ErrResp, SkillError>, op: &str) -> Result<(), SkillError> {
    match result {
        Err(e) => Err(SkillError(format!("{op}: {e}"))),
        Ok(r) if !r.error.is_empty() => Err(SkillError(r.error)),
        Ok(_) => Ok(()),
    }
}
