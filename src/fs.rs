//! File-system operations.

use crate::abi::{call_host, raw};
use crate::types::*;
use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub(crate) struct GrepResponse {
    #[serde(rename = "matches", with = "crate::vec_or_null")]
    pub matches: Vec<GrepFileMatch>,
    #[serde(rename = "error", default)]
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub(crate) struct GlobResponse {
    #[serde(rename = "matches", with = "crate::vec_or_null")]
    pub matches: Vec<GlobEntry>,
    #[serde(rename = "error", default)]
    pub error: String,
}


fn check_fs_error(err: String) -> Result<(), SkillError> {
    if err.is_empty() { Ok(()) } else { Err(SkillError(err)) }
}


/// Read a file and return its contents as bytes.
pub fn fs_read(path: &str) -> Result<Vec<u8>, SkillError> {
    let resp: FsResponse = call_host(raw::fs_read, &FsRequest { path: path.into(), content: vec![] })?;
    check_fs_error(resp.error)?;
    Ok(resp.content)
}

/// Write (overwrite) a file.
pub fn fs_write(path: &str, content: Vec<u8>) -> Result<(), SkillError> {
    let resp: FsResponse = call_host(raw::fs_write, &FsRequest { path: path.into(), content })?;
    check_fs_error(resp.error)
}

/// Create a new file (fails if already exists).
pub fn fs_create(path: &str, content: Vec<u8>) -> Result<(), SkillError> {
    let resp: FsResponse = call_host(raw::fs_create, &FsRequest { path: path.into(), content })?;
    check_fs_error(resp.error)
}

/// Delete a file or directory.
pub fn fs_delete(path: &str) -> Result<(), SkillError> {
    let resp: FsResponse = call_host(raw::fs_delete, &FsRequest { path: path.into(), content: vec![] })?;
    check_fs_error(resp.error)
}

/// Create one or more directories (always recursive). Supports brace expansion.
pub fn fs_mkdir(path: &str) -> Result<(), SkillError> {
    let resp: FsResponse = call_host(raw::fs_mkdir, &FsMkdirRequest { path: path.into() })?;
    check_fs_error(resp.error)
}

/// List directory contents.
///
/// When `long = true` every entry is stat-ed and `mod_time`, `mode`,
/// and `mode_str` are populated.
pub fn fs_ls(path: &str, long: bool) -> Result<Vec<DirEntry>, SkillError> {
    let resp: FsLsResponse = call_host(raw::fs_ls, &FsLsRequest { path: path.into(), long })?;
    check_fs_error(resp.error)?;
    Ok(resp.entries)
}

/// Change Unix permission bits.
pub fn fs_chmod(path: &str, mode: u32, recursive: bool) -> Result<(), SkillError> {
    let resp: FsResponse = call_host(raw::fs_chmod, &FsChmodRequest { path: path.into(), mode, recursive })?;
    check_fs_error(resp.error)
}

/// Read a slice of lines from a text file with optional pagination.
pub fn fs_read_lines(req: FsReadLinesRequest) -> Result<TextFileContent, SkillError> {
    let resp: TextFileContentResponse = call_host(raw::fs_read_lines, &req)?;
    if !resp.error.is_empty() {
        return Err(SkillError(resp.error));
    }
    Ok(TextFileContent {
        lines: resp.lines,
        total_lines: resp.total_lines,
        offset: resp.offset,
        is_truncated: resp.is_truncated,
    })
}

/// Search files using a regex (or fixed string).
pub fn fs_grep(opts: GrepOptions) -> Result<Vec<GrepFileMatch>, SkillError> {
    let resp: GrepResponse = call_host(raw::fs_grep, &opts)?;
    if !resp.error.is_empty() {
        return Err(SkillError(resp.error));
    }
    Ok(resp.matches)
}

/// Find files and directories matching glob patterns.
pub fn fs_glob(opts: GlobOptions) -> Result<Vec<GlobEntry>, SkillError> {
    let resp: GlobResponse = call_host(raw::fs_glob, &opts)?;
    if !resp.error.is_empty() {
        return Err(SkillError(resp.error));
    }
    Ok(resp.matches)
}

/// List of directories the skill is allowed to access. Requires fs.enabled.
pub fn fs_allowed_dirs() -> Result<Vec<AllowedDir>, SkillError> {
    #[derive(serde::Serialize)]
    struct Empty {}
    #[derive(serde::Deserialize)]
    struct Resp {
        #[serde(default)]
        dirs: Vec<AllowedDir>,
        #[serde(default)]
        error: String,
    }
    let resp: Resp = call_host(raw::fs_allowed_dirs, &Empty {})?;
    if !resp.error.is_empty() {
        return Err(SkillError(resp.error));
    }
    Ok(resp.dirs)
}

/// Copy a file within allowed directories.
pub fn fs_copy(source: &str, dest: &str) -> Result<(), SkillError> {
    #[derive(Serialize)]
    struct Req { source: String, dest: String }
    let resp: FsResponse = call_host(raw::fs_copy, &Req {
        source: source.into(), dest: dest.into(),
    })?;
    check_fs_error(resp.error)
}
