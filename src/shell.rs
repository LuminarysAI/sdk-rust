//! Shell execution. Requires `shell.enabled`.

use crate::abi::{call_host, raw};
use crate::types::*;

/// Execute a shell command and return output and exit code. Requires `shell.enabled`.
pub fn shell_exec(req: &ShellExecRequest) -> Result<ShellExecResult, SkillError> {
    let resp: ShellExecResult = call_host(raw::shell_exec, req)?;
    if !resp.error.is_empty() {
        return Err(SkillError(resp.error));
    }
    Ok(resp)
}
