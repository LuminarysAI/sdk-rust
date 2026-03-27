//! WebSocket API. Requires `http.enabled` and `http.allow_websocket`.

use crate::abi::{call_host, raw};
use crate::tcp::ErrorKind;
use crate::types::SkillError;
use serde::{Deserialize, Serialize};

/// WebSocket message type constants.
pub mod ws_message {
    pub const TEXT: &str = "text";
    pub const BINARY: &str = "binary";
    pub const CLOSE: &str = "close";
}

/// Payload delivered to the skill's WebSocket callback.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct WsEvent {
    #[serde(rename = "conn_id")]
    pub conn_id: String,
    #[serde(rename = "data", default, with = "crate::bytes_or_null", skip_serializing_if = "Vec::is_empty")]
    pub data: Vec<u8>,
    /// `"text"`, `"binary"`, or `"close"`.
    #[serde(rename = "message_type", default, skip_serializing_if = "String::is_empty")]
    pub message_type: String,
    #[serde(rename = "close_code", default, skip_serializing_if = "is_zero_u32")]
    pub close_code: u32,
    #[serde(rename = "close_text", default, skip_serializing_if = "String::is_empty")]
    pub close_text: String,
    #[serde(rename = "error_kind", default)]
    pub error_kind: ErrorKind,
    #[serde(rename = "error_msg", default, skip_serializing_if = "String::is_empty")]
    pub error_msg: String,
}

/// Deserialise a [`WsEvent`] from the raw payload.
pub fn unmarshal_ws_event(payload: &[u8]) -> Result<WsEvent, SkillError> {
    Ok(rmp_serde::from_slice(payload)?)
}

/// Text message (UTF-8).
pub const WS_MESSAGE_TEXT: &str = "text";
/// Binary message.
pub const WS_MESSAGE_BINARY: &str = "binary";
/// Close frame.
pub const WS_MESSAGE_CLOSE: &str = "close";

/// Dial a WebSocket connection. Requires `http.enabled` and `http.allow_websocket`.
pub fn ws_connect(
    url: &str,
    headers: Vec<crate::http::Header>,
    timeout_ms: i64,
    callback: &str,
    insecure: bool,
) -> Result<String, SkillError> {
    #[derive(Serialize, Default)]
    struct Req<'a> {
        url: &'a str,
        headers: Vec<crate::http::Header>,
        timeout_ms: i64,
        callback: &'a str,
        insecure: bool,
    }
    #[derive(Deserialize, Default)]
    struct Resp {
        #[serde(default)]
        conn_id: String,
        #[serde(default)]
        error: String,
    }
    let resp: Resp =
        call_host(raw::ws_connect, &Req { url, headers, timeout_ms, callback, insecure })?;
    if !resp.error.is_empty() {
        return Err(SkillError(resp.error));
    }
    Ok(resp.conn_id)
}

/// Send a message over an existing WebSocket connection.
pub fn ws_send(conn_id: &str, data: Vec<u8>, message_type: &str) -> Result<(), SkillError> {
    #[derive(Serialize, Default)]
    struct Req<'a> {
        conn_id: &'a str,
        #[serde(with = "crate::bytes_or_null")]
        data: Vec<u8>,
        message_type: &'a str,
    }
    #[derive(Deserialize, Default)]
    #[serde(default)]
    struct Resp {
        #[serde(default)]
        error: String,
    }
    let resp: Resp = call_host(raw::ws_send, &Req { conn_id, data, message_type })?;
    if !resp.error.is_empty() {
        return Err(SkillError(resp.error));
    }
    Ok(())
}

/// Send a Close frame and remove the connection.
pub fn ws_close(conn_id: &str, code: u32, reason: &str) -> Result<(), SkillError> {
    #[derive(Serialize, Default)]
    struct Req<'a> {
        conn_id: &'a str,
        code: u32,
        reason: &'a str,
    }
    #[derive(Deserialize, Default)]
    #[serde(default)]
    struct Resp {
        #[serde(default)]
        error: String,
    }
    let resp: Resp = call_host(raw::ws_close, &Req { conn_id, code, reason })?;
    if !resp.error.is_empty() {
        return Err(SkillError(resp.error));
    }
    Ok(())
}

fn is_zero_u32(v: &u32) -> bool { *v == 0 }
