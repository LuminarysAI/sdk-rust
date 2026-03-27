//! System info, time, disk usage, environment, and logging.

use crate::abi::{call_host, raw};
use crate::types::*;
use std::collections::HashMap;

pub const LOG_DEBUG: &str = "debug";
pub const LOG_INFO: &str = "info";
pub const LOG_WARN: &str = "warn";
pub const LOG_ERROR: &str = "error";

/// Host OS and hardware info.
pub fn sys_info() -> Result<SysInfoResult, SkillError> {
    #[derive(serde::Serialize)]
    struct Empty {}
    call_host(raw::sys_info, &Empty {})
}

/// Current host time in multiple formats.
pub fn time_now() -> Result<TimeNowResult, SkillError> {
    #[derive(serde::Serialize)]
    struct Empty {}
    call_host(raw::time_now, &Empty {})
}

/// Disk space info for a path. Requires `fs.enabled`.
pub fn disk_usage(path: &str) -> Result<DiskUsageResult, SkillError> {
    #[derive(serde::Serialize)]
    struct Req<'a> { path: &'a str }
    let resp: DiskUsageResult = call_host(raw::disk_usage, &Req { path })?;
    if !resp.error.is_empty() {
        return Err(SkillError(resp.error));
    }
    Ok(resp)
}

/// Read a named env value declared in the skill's manifest.
pub fn get_env(key: &str) -> String {
    #[derive(serde::Serialize)]
    struct Req<'a> { key: &'a str }
    #[derive(serde::Deserialize)]
    struct Resp { value: String }
    call_host::<_, Resp>(raw::env_get, &Req { key })
        .map(|r| r.value)
        .unwrap_or_default()
}

/// Write a structured log message.
pub fn log_msg(level: &str, message: &str, fields: &[(&str, &str)]) {
    let mut f: HashMap<&str, &str> = HashMap::new();
    for (k, v) in fields {
        f.insert(k, v);
    }
    #[derive(serde::Serialize)]
    struct Req<'a> {
        level: &'a str,
        message: &'a str,
        fields: HashMap<&'a str, &'a str>,
    }
    #[derive(serde::Deserialize)]
    struct Empty {}
    let _ = call_host::<_, Empty>(raw::log_write, &Req { level, message, fields: f });
}

/// Shorthand: debug log.
pub fn log_debug(message: &str, fields: &[(&str, &str)]) { log_msg("debug", message, fields); }
/// Shorthand: info log.
pub fn log_info(message: &str, fields: &[(&str, &str)]) { log_msg("info", message, fields); }
/// Shorthand: warn log.
pub fn log_warn(message: &str, fields: &[(&str, &str)]) { log_msg("warn", message, fields); }
/// Shorthand: error log.
pub fn log_error(message: &str, fields: &[(&str, &str)]) { log_msg("error", message, fields); }
