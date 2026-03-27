//! Protocol types shared between the skill and the host.

use serde::{Deserialize, Serialize};

/// Protocol version.
pub const VERSION: u32 = 1;

/// Sent by the host to the skill's `skill_handle` export.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct InvokeRequest {
    #[serde(rename = "version", default)]
    pub version: u32,
    #[serde(rename = "request_id", default)]
    pub request_id: String,
    #[serde(rename = "trace_id", default)]
    pub trace_id: String,
    #[serde(rename = "session_id", default)]
    pub session_id: String,
    #[serde(rename = "llm_session_id", default)]
    pub llm_session_id: String,
    #[serde(rename = "method", default)]
    pub method: String,
    #[serde(rename = "skill_id", default)]
    pub skill_id: String,
    /// Skill ID of the caller when invoked via `call_module`; empty otherwise.
    #[serde(rename = "caller_id", default, skip_serializing_if = "String::is_empty")]
    pub caller_id: String,
    #[serde(rename = "state", default, with = "crate::bytes_or_null")]
    pub state: Vec<u8>,
    #[serde(rename = "payload", default, with = "crate::bytes_or_null")]
    pub payload: Vec<u8>,
    #[serde(rename = "deadline_ns", default)]
    pub deadline_ns: i64,
}

/// Returned by the skill's `skill_handle` export.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct InvokeResponse {
    #[serde(rename = "version", default)]
    pub version: u32,
    #[serde(rename = "state", default, with = "crate::bytes_or_null")]
    pub state: Vec<u8>,
    #[serde(rename = "payload", default, with = "crate::bytes_or_null")]
    pub payload: Vec<u8>,
    #[serde(rename = "error", default, skip_serializing_if = "String::is_empty")]
    pub error: String,
    #[serde(rename = "commands", default, skip_serializing_if = "Vec::is_empty", with = "crate::vec_or_null")]
    pub commands: Vec<Command>,
    /// Extra text appended to the MCP tool result for the LLM.
    #[serde(rename = "llm_context", default, skip_serializing_if = "String::is_empty")]
    pub llm_context: String,
}

/// Instruction returned by the skill.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Command {
    #[serde(rename = "type")]
    pub kind: CommandType,
    #[serde(rename = "payload", default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<rmpv::Value>,
}

/// Command types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CommandType {
    #[default]
    CallModule,
    BatchInvoke,
    Schedule,
    EmitEvent,
    Subscribe,
    StoreKv,
    LoadKv,
    Spawn,
    Terminate,
}

/// One call inside a `batch_invoke` command.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BatchItem {
    #[serde(rename = "index")]
    pub index: u32,
    #[serde(rename = "skill_id")]
    pub skill_id: String,
    #[serde(rename = "method")]
    pub method: String,
    #[serde(rename = "payload", with = "crate::bytes_or_null")]
    pub payload: Vec<u8>,
}

/// Outcome of one [`BatchItem`] after execution.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BatchItemResult {
    #[serde(rename = "index")]
    pub index: u32,
    #[serde(rename = "payload", default, with = "crate::bytes_or_null", skip_serializing_if = "Vec::is_empty")]
    pub payload: Vec<u8>,
    #[serde(rename = "error", default, skip_serializing_if = "String::is_empty")]
    pub error: String,
}

/// Delivered to the batch-callback method.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BatchResult {
    #[serde(rename = "batch_id")]
    pub batch_id: String,
    #[serde(rename = "items", with = "crate::vec_or_null")]
    pub items: Vec<BatchItemResult>,
}

/// One entry from the dialogue history.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HistoryMessage {
    #[serde(rename = "role")]
    pub role: String,
    #[serde(rename = "content")]
    pub content: String,
}

/// Prompt request for `prompt_complete`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PromptRequest {
    #[serde(rename = "prompt")]
    pub prompt: String,
    #[serde(rename = "temperature")]
    pub temperature: f64,
    #[serde(rename = "max_tokens")]
    pub max_tokens: u32,
}

/// Response from `prompt_complete`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PromptResponse {
    #[serde(rename = "content")]
    pub content: String,
    #[serde(rename = "error", default, skip_serializing_if = "String::is_empty")]
    pub error: String,
}

/// Generic FS request.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct FsRequest {
    #[serde(rename = "path")]
    pub path: String,
    #[serde(rename = "content", default, with = "crate::bytes_or_null", skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<u8>,
}

/// Generic FS response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct FsResponse {
    #[serde(rename = "content", default, with = "crate::bytes_or_null", skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<u8>,
    #[serde(rename = "error", default, skip_serializing_if = "String::is_empty")]
    pub error: String,
}

/// Request for `fs_mkdir`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct FsMkdirRequest {
    #[serde(rename = "path")]
    pub path: String,
}

/// Request for `fs_ls`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct FsLsRequest {
    #[serde(rename = "path")]
    pub path: String,
    /// When `true`, populates `mod_time`, `mode`, and `mode_str`.
    #[serde(rename = "long")]
    pub long: bool,
}

/// One entry returned by `fs_ls`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DirEntry {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "size")]
    pub size: i64,
    #[serde(rename = "is_dir")]
    pub is_dir: bool,
    #[serde(rename = "mod_time", default, skip_serializing_if = "is_zero_i64")]
    pub mod_time: i64,
    #[serde(rename = "mode", default, skip_serializing_if = "is_zero_u32")]
    pub mode: u32,
    #[serde(rename = "mode_str", default, skip_serializing_if = "String::is_empty")]
    pub mode_str: String,
}

/// Response from `fs_ls`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct FsLsResponse {
    #[serde(rename = "entries", with = "crate::vec_or_null")]
    pub entries: Vec<DirEntry>,
    #[serde(rename = "error", default, skip_serializing_if = "String::is_empty")]
    pub error: String,
}

/// Request for `fs_chmod`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct FsChmodRequest {
    #[serde(rename = "path")]
    pub path: String,
    #[serde(rename = "mode")]
    pub mode: u32,
    #[serde(rename = "recursive")]
    pub recursive: bool,
}

/// Unix permission bit constants.
pub mod perm {
    pub const OWNER_READ: u32 = 0o400;
    pub const OWNER_WRITE: u32 = 0o200;
    pub const OWNER_EXEC: u32 = 0o100;
    pub const GROUP_READ: u32 = 0o040;
    pub const GROUP_WRITE: u32 = 0o020;
    pub const GROUP_EXEC: u32 = 0o010;
    pub const OTHER_READ: u32 = 0o004;
    pub const OTHER_WRITE: u32 = 0o002;
    pub const OTHER_EXEC: u32 = 0o001;

    pub const READ_ONLY: u32 = 0o444;
    pub const DEFAULT: u32 = 0o644;
    pub const DEFAULT_DIR: u32 = 0o755;
    pub const PRIVATE: u32 = 0o600;
    pub const PRIVATE_DIR: u32 = 0o700;
}

/// Request for `fs_read_lines`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct FsReadLinesRequest {
    #[serde(rename = "path")]
    pub path: String,
    #[serde(rename = "offset", default)]
    pub offset: i64,
    #[serde(rename = "limit", default)]
    pub limit: i64,
}

/// Response from `fs_read_lines`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TextFileContent {
    #[serde(rename = "lines", with = "crate::vec_or_null")]
    pub lines: Vec<String>,
    #[serde(rename = "total_lines")]
    pub total_lines: i64,
    #[serde(rename = "offset")]
    pub offset: i64,
    #[serde(rename = "is_truncated")]
    pub is_truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub(crate) struct TextFileContentResponse {
    #[serde(rename = "lines", with = "crate::vec_or_null")]
    pub lines: Vec<String>,
    #[serde(rename = "total_lines")]
    pub total_lines: i64,
    #[serde(rename = "offset")]
    pub offset: i64,
    #[serde(rename = "is_truncated")]
    pub is_truncated: bool,
    #[serde(rename = "error", default)]
    pub error: String,
}

/// Options for `fs_grep`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GrepOptions {
    #[serde(rename = "pattern")]
    pub pattern: String,
    #[serde(rename = "path", default, skip_serializing_if = "String::is_empty")]
    pub path: String,
    #[serde(rename = "fixed", default, skip_serializing_if = "is_false")]
    pub fixed: bool,
    #[serde(rename = "case_insensitive", default, skip_serializing_if = "is_false")]
    pub case_insensitive: bool,
    #[serde(rename = "max_depth", default, skip_serializing_if = "is_zero_i64")]
    pub max_depth: i64,
    #[serde(rename = "max_count", default, skip_serializing_if = "is_zero_i64")]
    pub max_count: i64,
    #[serde(rename = "workers", default, skip_serializing_if = "is_zero_i64")]
    pub workers: i64,
    #[serde(rename = "type_filter", default, skip_serializing_if = "String::is_empty")]
    pub type_filter: String,
    #[serde(rename = "include", default, skip_serializing_if = "Vec::is_empty", with = "crate::vec_or_null")]
    pub include: Vec<String>,
    #[serde(rename = "exclude", default, skip_serializing_if = "Vec::is_empty", with = "crate::vec_or_null")]
    pub exclude: Vec<String>,
    #[serde(rename = "ignore_dirs", default, skip_serializing_if = "Option::is_none")]
    pub ignore_dirs: Option<Vec<String>>,
    #[serde(rename = "with_lines", default, skip_serializing_if = "is_false")]
    pub with_lines: bool,
    #[serde(rename = "filename_only", default, skip_serializing_if = "is_false")]
    pub filename_only: bool,
}

/// All matches found in one file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct GrepFileMatch {
    #[serde(rename = "path")]
    pub path: String,
    #[serde(rename = "matches", with = "crate::vec_or_null")]
    pub matches: Vec<GrepLineMatch>,
}

/// One matching line inside a file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct GrepLineMatch {
    /// 1-based line number.
    #[serde(rename = "line_num")]
    pub line_num: i64,
    #[serde(rename = "line", default, skip_serializing_if = "String::is_empty")]
    pub line: String,
    #[serde(rename = "ranges", default, skip_serializing_if = "Vec::is_empty", with = "crate::vec_or_null")]
    pub ranges: Vec<GrepRange>,
}

/// `[start, end)` byte-offset pair within a line.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct GrepRange {
    #[serde(rename = "start")]
    pub start: i64,
    #[serde(rename = "end")]
    pub end: i64,
}

/// Options for `fs_glob`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GlobOptions {
    #[serde(rename = "patterns", with = "crate::vec_or_null")]
    pub patterns: Vec<String>,
    #[serde(rename = "path", default, skip_serializing_if = "String::is_empty")]
    pub path: String,
    #[serde(rename = "match_hidden", default, skip_serializing_if = "is_false")]
    pub match_hidden: bool,
    #[serde(rename = "ignore_dirs", default, skip_serializing_if = "Option::is_none")]
    pub ignore_dirs: Option<Vec<String>>,
    #[serde(rename = "max_depth", default, skip_serializing_if = "is_zero_i64")]
    pub max_depth: i64,
    #[serde(rename = "only_files", default, skip_serializing_if = "is_false")]
    pub only_files: bool,
    #[serde(rename = "only_dirs", default, skip_serializing_if = "is_false")]
    pub only_dirs: bool,
}

/// One result from `fs_glob`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct GlobEntry {
    #[serde(rename = "path")]
    pub path: String,
    #[serde(rename = "is_dir")]
    pub is_dir: bool,
}

/// Options for `tcp_request`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TcpRequestOptions {
    #[serde(rename = "addr")]
    pub addr: String,
    #[serde(rename = "data", default, with = "crate::bytes_or_null")]
    pub data: Vec<u8>,
    #[serde(rename = "tls", default)]
    pub tls: bool,
    #[serde(rename = "insecure", default)]
    pub insecure: bool,
    #[serde(rename = "timeout_ms", default)]
    pub timeout_ms: i64,
    #[serde(rename = "max_bytes", default)]
    pub max_bytes: i64,
}

/// Result from `tcp_request`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TcpRequestResult {
    #[serde(rename = "data", default, with = "crate::bytes_or_null")]
    pub data: Vec<u8>,
    #[serde(rename = "error", default)]
    pub error: String,
}

/// Request for `shell_exec`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShellExecRequest {
    /// The shell command to execute (required).
    #[serde(rename = "command")]
    pub command: String,
    /// Working directory (optional).
    #[serde(rename = "workdir", default)]
    pub workdir: String,
    /// Timeout in milliseconds (0 = 30s default).
    #[serde(rename = "timeout_ms", default)]
    pub timeout_ms: i64,
    /// Return only the last N lines of output. 0 = all lines.
    #[serde(rename = "tail", default)]
    pub tail: i64,
    /// Filter output lines by regex pattern.
    #[serde(rename = "grep", default)]
    pub grep: String,
    /// Start the process in background and return immediately with PID.
    #[serde(rename = "as_daemon", default, skip_serializing_if = "is_false")]
    pub as_daemon: bool,
    /// Output file for daemon mode.
    #[serde(rename = "log_file", default, skip_serializing_if = "String::is_empty")]
    pub log_file: String,
}

/// Result from `shell_exec`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShellExecResult {
    /// Combined stdout+stderr output.
    #[serde(rename = "output", default)]
    pub output: String,
    /// Process exit code (0 = success).
    #[serde(rename = "exit_code", default)]
    pub exit_code: i64,
    /// Process ID (only set when as_daemon=true).
    #[serde(rename = "pid", default)]
    pub pid: i64,
    /// Log file path (only set when as_daemon=true).
    #[serde(rename = "log_file", default, skip_serializing_if = "String::is_empty")]
    pub log_file: String,
    #[serde(rename = "error", default)]
    pub error: String,
}

/// Host OS and hardware information.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SysInfoResult {
    pub os: String,
    pub arch: String,
    pub hostname: String,
    pub num_cpu: i64,
}

/// Current host time.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TimeNowResult {
    pub unix: i64,
    pub unix_nano: i64,
    pub rfc3339: String,
    pub timezone: String,
    pub utc_offset: i64,
}

/// Disk usage information.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DiskUsageResult {
    pub total_bytes: i64,
    pub free_bytes: i64,
    pub used_bytes: i64,
    pub used_pct: f64,
    #[serde(default)]
    pub error: String,
}

/// Allowed directory entry.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AllowedDir {
    pub path: String,
    pub mode: String,
}

/// Error returned from a host call.
#[derive(Debug, Clone)]
pub struct SkillError(pub String);

impl std::fmt::Display for SkillError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "abi error: {}", self.0)
    }
}

impl std::error::Error for SkillError {}

impl From<rmp_serde::encode::Error> for SkillError {
    fn from(e: rmp_serde::encode::Error) -> Self {
        SkillError(format!("msgpack encode: {e}"))
    }
}

impl From<rmp_serde::decode::Error> for SkillError {
    fn from(e: rmp_serde::decode::Error) -> Self {
        SkillError(format!("msgpack decode: {e}"))
    }
}

#[inline]
fn is_false(v: &bool) -> bool { !v }

#[inline]
fn is_zero_i64(v: &i64) -> bool { *v == 0 }

#[inline]
fn is_zero_u32(v: &u32) -> bool { *v == 0 }
