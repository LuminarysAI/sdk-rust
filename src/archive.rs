//! Archive operations (tar.gz, zip).

use crate::abi::{call_host, raw};
use crate::types::SkillError;
use serde::{Serialize, Deserialize};

/// Result of archive_pack.
#[derive(Deserialize, Default)]
pub struct ArchivePackResult {
    #[serde(default)]
    pub files_count: i64,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub error: String,
}

/// Archive entry in listing.
#[derive(Deserialize, Default)]
pub struct ArchiveEntry {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub is_dir: bool,
}

/// Create a tar.gz or zip archive from a directory.
pub fn archive_pack(source: &str, output: &str, format: &str, exclude: &str) -> Result<ArchivePackResult, SkillError> {
    #[derive(Serialize)]
    struct Req { source: String, output: String, format: String, exclude: String }
    let resp: ArchivePackResult = call_host(raw::archive_pack, &Req {
        source: source.into(), output: output.into(), format: format.into(),
        exclude: exclude.into(),
    })?;
    if !resp.error.is_empty() { return Err(SkillError(resp.error)); }
    Ok(resp)
}

/// List contents of a tar.gz or zip archive.
pub fn archive_list(archive: &str, format: &str, exclude: &str) -> Result<Vec<ArchiveEntry>, SkillError> {
    #[derive(Serialize)]
    struct Req { archive: String, format: String, exclude: String }
    #[derive(Deserialize, Default)]
    struct Resp { #[serde(default)] entries: Vec<ArchiveEntry>, #[serde(default)] error: String }
    let resp: Resp = call_host(raw::archive_list, &Req {
        archive: archive.into(), format: format.into(),
        exclude: exclude.into(),
    })?;
    if !resp.error.is_empty() { return Err(SkillError(resp.error)); }
    Ok(resp.entries)
}

/// Extract a tar.gz or zip archive.
pub fn archive_unpack(archive: &str, dest: &str, format: &str, exclude: &str, strip: i32) -> Result<i64, SkillError> {
    #[derive(Serialize)]
    struct Req { archive: String, dest: String, format: String, exclude: String, strip: i32 }
    #[derive(Deserialize, Default)]
    struct Resp { #[serde(default)] files_count: i64, #[serde(default)] error: String }
    let resp: Resp = call_host(raw::archive_unpack, &Req {
        archive: archive.into(), dest: dest.into(), format: format.into(),
        exclude: exclude.into(), strip,
    })?;
    if !resp.error.is_empty() { return Err(SkillError(resp.error)); }
    Ok(resp.files_count)
}
