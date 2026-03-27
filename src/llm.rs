//! LLM API — history, prompt completion, batch results.

use crate::abi::{call_host, raw};
use crate::types::*;

/// Fetch dialogue history matching `filter` (e.g. `"last:10"`, `"role:user"`).
pub fn history_get(filter: &str) -> Result<Vec<HistoryMessage>, SkillError> {
    #[derive(serde::Serialize)]
    struct Req<'a> { filter: &'a str }
    call_host(raw::history_get, &Req { filter })
}

/// Send a prompt to the LLM.
pub fn prompt_complete(req: PromptRequest) -> Result<PromptResponse, SkillError> {
    let resp: PromptResponse = call_host(raw::prompt_complete, &req)?;
    if !resp.error.is_empty() {
        return Err(SkillError(resp.error));
    }
    Ok(resp)
}

/// Decode a batch-callback payload into a [`BatchResult`].
pub fn unmarshal_batch_result(payload: &[u8]) -> Result<BatchResult, SkillError> {
    Ok(rmp_serde::from_slice(payload)?)
}
