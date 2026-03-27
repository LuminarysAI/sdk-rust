//! [`Context`] — request metadata and response annotations passed to every handler.

use crate::types::{InvokeRequest, InvokeResponse};

/// Passed to every skill handler. Provides read access to invocation metadata
/// and write access to response annotations.
pub struct Context<'a> {
    req: &'a InvokeRequest,
    res: &'a mut InvokeResponse,
}

impl<'a> Context<'a> {
    /// Create a [`Context`] bound to `req` and `res`.
    pub fn new(req: &'a InvokeRequest, res: &'a mut InvokeResponse) -> Self {
        Context { req, res }
    }

    /// Unique ID of this invocation.
    pub fn request_id(&self) -> &str { &self.req.request_id }

    /// Distributed trace ID.
    pub fn trace_id(&self) -> &str { &self.req.trace_id }

    /// User session ID.
    pub fn session_id(&self) -> &str { &self.req.session_id }

    /// LLM session ID.
    pub fn llm_session_id(&self) -> &str { &self.req.llm_session_id }

    /// ID of the skill being invoked.
    pub fn skill_id(&self) -> &str { &self.req.skill_id }

    /// Skill ID of the caller when invoked via `call_module`. Empty otherwise.
    pub fn caller_id(&self) -> &str { &self.req.caller_id }

    /// Method name being invoked.
    pub fn method(&self) -> &str { &self.req.method }

    /// Set additional text appended to the tool result for the LLM.
    pub fn set_llm_context(&mut self, text: impl Into<String>) {
        self.res.llm_context = text.into();
    }

    /// Append text to any existing `llm_context`, separated by `"\n"`.
    pub fn append_llm_context(&mut self, text: impl Into<String>) {
        let text = text.into();
        if self.res.llm_context.is_empty() {
            self.res.llm_context = text;
        } else {
            self.res.llm_context.push('\n');
            self.res.llm_context.push_str(&text);
        }
    }
}
