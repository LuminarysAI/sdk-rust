//! Exports, handler registration, and method descriptors.

use crate::types::{InvokeRequest, InvokeResponse, VERSION};
use std::cell::RefCell;

pub const SDK_VERSION_MAJOR: u32 = 0;
pub const SDK_VERSION_MINOR: u32 = 2;
pub const SDK_VERSION_PATCH: u32 = 0;
pub const SDK_VERSION: u32 = (SDK_VERSION_MAJOR << 16) | (SDK_VERSION_MINOR << 8) | SDK_VERSION_PATCH;

/// The function signature every skill handler must implement.
pub type Handler = fn(InvokeRequest) -> InvokeResponse;

thread_local! {
    static HANDLER: RefCell<Option<Handler>> = RefCell::new(None);
    static METHODS: RefCell<Vec<MethodInfo>> = RefCell::new(Vec::new());
    static EXTRA: RefCell<Vec<(String, Handler)>> = RefCell::new(Vec::new());
    static REQUIREMENTS: RefCell<Vec<RequirementInfo>> = RefCell::new(Vec::new());
    static IDENTITY: RefCell<(String, String, String, String)> = RefCell::new(Default::default());
    static RESULT_BUF: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}

/// Set the skill's identity metadata from annotations.
pub fn set_skill_identity(id: &str, name: &str, version: &str, description: &str) {
    IDENTITY.with(|i| {
        *i.borrow_mut() = (id.into(), name.into(), version.into(), description.into());
    });
}

/// Register the main dispatch handler and method descriptors.
pub fn register(h: Option<Handler>, methods: Vec<MethodInfo>) {
    if let Some(handler) = h {
        HANDLER.with(|slot| *slot.borrow_mut() = Some(handler));
    }
    METHODS.with(|m| {
        let mut list = m.borrow_mut();
        for info in &methods {
            if let Some(fn_) = info.handler_fn {
                EXTRA.with(|e| e.borrow_mut().push((info.name.clone(), fn_)));
            }
        }
        list.extend(methods);
    });
}

/// Register additional methods without replacing the generated dispatcher.
pub fn register_methods(methods: Vec<MethodInfo>) {
    register(None, methods);
}

/// Register skill requirements (from @skill:require annotations).
pub fn register_requirements(reqs: Vec<RequirementInfo>) {
    REQUIREMENTS.with(|r| r.borrow_mut().extend(reqs));
}

/// Dispatch an [`InvokeRequest`] to the registered handler.
pub fn call_handler(req: InvokeRequest) -> InvokeResponse {
    let found = EXTRA.with(|e| {
        e.borrow()
            .iter()
            .find(|(name, _)| name == &req.method)
            .map(|(_, h)| *h)
    });
    if let Some(h) = found {
        return h(req);
    }
    HANDLER.with(|slot| {
        if let Some(h) = *slot.borrow() {
            h(req)
        } else {
            error_response("no handler registered — call register() or use skill_main!()")
        }
    })
}

/// Return the [`SkillDescriptor`] for the currently registered methods.
pub fn get_descriptor() -> SkillDescriptor {
    let (id, name, ver, desc) = IDENTITY.with(|i| i.borrow().clone());
    SkillDescriptor {
        version: VERSION,
        skill_id: id,
        skill_name: name,
        skill_version: ver,
        description: desc,
        methods: METHODS.with(|m| m.borrow().clone()),
        requirements: REQUIREMENTS.with(|r| r.borrow().clone()),
        sdk_version: SDK_VERSION,
    }
}

/// Allocate `size` bytes for the host to write the request into.
pub fn skill_alloc_impl(size: u32) -> u32 {
    if size == 0 {
        return 0;
    }
    let total = size as usize + 8;
    let mut buf: Vec<u8> = Vec::with_capacity(total);
    unsafe { buf.set_len(total) };
    buf[..8].copy_from_slice(&(total as u64).to_le_bytes());
    let data_ptr = unsafe { buf.as_mut_ptr().add(8) };
    std::mem::forget(buf);
    data_ptr as u32
}

/// Free a buffer previously returned by `skill_alloc` or the result buffer.
pub fn skill_free_impl(ptr: u32) {
    if ptr == 0 {
        return;
    }

    let is_result = RESULT_BUF.with(|buf| {
        let b = buf.borrow();
        if b.is_empty() {
            return false;
        }
        b.as_ptr() as u32 == ptr
    });
    if is_result {
        RESULT_BUF.with(|buf| {
            *buf.borrow_mut() = Vec::new();
        });
        return;
    }

    unsafe {
        let header_ptr = (ptr as usize - 8) as *mut u8;
        let total_len =
            u64::from_le_bytes(*(header_ptr as *const [u8; 8])) as usize;
        drop(Vec::from_raw_parts(header_ptr, total_len, total_len));
    }
}

/// Deserialise the request, dispatch, serialise the response.
pub fn skill_handle_impl(req_ptr: u32, req_len: u32) -> u64 {
    let bytes = unsafe {
        std::slice::from_raw_parts(req_ptr as *const u8, req_len as usize)
    };
    let req: InvokeRequest = match rmp_serde::from_slice(bytes) {
        Ok(r) => r,
        Err(e) => {
            return write_result(marshal_error(&format!("unmarshal InvokeRequest: {e}")));
        }
    };
    let mut resp = call_handler(req);
    resp.version = VERSION;
    let encoded = match rmp_serde::to_vec_named(&resp) {
        Ok(b) => b,
        Err(e) => marshal_error(&format!("marshal InvokeResponse: {e}")),
    };
    write_result(encoded)
}

/// Serialise the registered [`SkillDescriptor`] and return a packed pointer.
pub fn skill_describe_impl() -> u64 {
    let desc = get_descriptor();
    let encoded = rmp_serde::to_vec_named(&desc).unwrap_or_default();
    write_result(encoded)
}

fn write_result(bytes: Vec<u8>) -> u64 {
    RESULT_BUF.with(|buf| {
        *buf.borrow_mut() = bytes;
        let b = buf.borrow();
        if b.is_empty() {
            return 0;
        }
        let ptr = b.as_ptr() as u64;
        let len = b.len() as u64;
        (ptr << 32) | len
    })
}

/// Emit the four required ABI exports for this skill.
///
/// Place at the crate root (usually `src/lib.rs`):
///
/// ```rust,ignore
/// use luminarys_sdk::prelude::*;
/// skill_main!();
/// ```
#[macro_export]
macro_rules! skill_main {
    () => {
        #[no_mangle]
        pub extern "C" fn skill_alloc(size: u32) -> u32 {
            $crate::entrypoint::skill_alloc_impl(size)
        }

        #[no_mangle]
        pub extern "C" fn skill_free(ptr: u32) {
            $crate::entrypoint::skill_free_impl(ptr);
        }

        #[no_mangle]
        pub extern "C" fn skill_handle(req_ptr: u32, req_len: u32) -> u64 {
            $crate::entrypoint::skill_handle_impl(req_ptr, req_len)
        }

        #[no_mangle]
        pub extern "C" fn skill_describe() -> u64 {
            $crate::entrypoint::skill_describe_impl()
        }
    };
}

/// Describes a single exported method.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct MethodInfo {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "description")]
    pub description: String,
    #[serde(rename = "params", default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<ParamInfo>,
    #[serde(rename = "mcp_hidden", default, skip_serializing_if = "is_false")]
    pub mcp_hidden: bool,
    #[serde(rename = "private_callback", default, skip_serializing_if = "is_false")]
    pub private_callback: bool,
    #[serde(skip)]
    pub(crate) handler_fn: Option<Handler>,
}

/// Start building a [`MethodInfo`].
pub fn method(name: impl Into<String>, description: impl Into<String>) -> MethodInfo {
    MethodInfo {
        name: name.into(),
        description: description.into(),
        params: vec![],
        mcp_hidden: false,
        private_callback: false,
        handler_fn: None,
    }
}

impl MethodInfo {
    /// Add a parameter descriptor.
    pub fn param(mut self, name: impl Into<String>, p: ParamInfo) -> Self {
        let mut pi = p;
        pi.name = name.into();
        self.params.push(pi);
        self
    }
    /// Hide from MCP `tools/list`; still callable via `call_module`.
    pub fn set_internal(mut self) -> Self { self.mcp_hidden = true; self }
    /// Private callback — hidden from MCP AND only callable by this skill.
    pub fn callback(mut self) -> Self {
        self.mcp_hidden = true;
        self.private_callback = true;
        self
    }
    /// Attach an inline handler.
    pub fn handle(mut self, f: Handler) -> Self { self.handler_fn = Some(f); self }
}

/// Describes one parameter of a method.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct ParamInfo {
    #[serde(rename = "name", default)]
    pub name: String,
    #[serde(rename = "description", default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(rename = "type")]
    pub type_val: String,
    #[serde(rename = "required", default, skip_serializing_if = "is_false")]
    pub is_required: bool,
    #[serde(rename = "enum", default, skip_serializing_if = "Vec::is_empty")]
    pub enum_vals: Vec<String>,
    #[serde(rename = "items", default, skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<ParamInfo>>,
}

impl ParamInfo {
    pub fn required(mut self) -> Self { self.is_required = true; self }
    pub fn desc(mut self, d: impl Into<String>) -> Self { self.description = d.into(); self }
    pub fn enum_vals(mut self, vals: Vec<impl Into<String>>) -> Self {
        self.enum_vals = vals.into_iter().map(|v| v.into()).collect();
        self
    }
}

fn mk_param(type_val: &str) -> ParamInfo {
    ParamInfo {
        name: String::new(),
        description: String::new(),
        type_val: type_val.into(),
        is_required: false,
        enum_vals: vec![],
        items: None,
    }
}

pub fn type_string() -> ParamInfo { mk_param("string") }
pub fn type_int()    -> ParamInfo { mk_param("integer") }
pub fn type_number() -> ParamInfo { mk_param("number") }
pub fn type_bool()   -> ParamInfo { mk_param("boolean") }
pub fn type_object() -> ParamInfo { mk_param("object") }
pub fn type_array(items: ParamInfo) -> ParamInfo {
    ParamInfo { type_val: "array".into(), items: Some(Box::new(items)), ..mk_param("array") }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct SkillDescriptor {
    #[serde(rename = "version")]
    pub version: u32,
    #[serde(rename = "skill_id", default, skip_serializing_if = "String::is_empty")]
    pub skill_id: String,
    #[serde(rename = "skill_name", default, skip_serializing_if = "String::is_empty")]
    pub skill_name: String,
    #[serde(rename = "skill_version", default, skip_serializing_if = "String::is_empty")]
    pub skill_version: String,
    #[serde(rename = "description", default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(rename = "methods")]
    pub methods: Vec<MethodInfo>,
    #[serde(rename = "requirements", default, skip_serializing_if = "Vec::is_empty")]
    pub requirements: Vec<RequirementInfo>,
    #[serde(rename = "sdk_version", default)]
    pub sdk_version: u32,
}

/// Declares one permission the skill expects from its manifest.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct RequirementInfo {
    #[serde(rename = "kind")]
    pub kind: String,
    #[serde(rename = "pattern", default, skip_serializing_if = "String::is_empty")]
    pub pattern: String,
    #[serde(rename = "mode", default, skip_serializing_if = "String::is_empty")]
    pub mode: String,
}

/// Build a successful [`InvokeResponse`] with an encoded payload.
pub fn ok_response<T: serde::Serialize>(value: &T) -> InvokeResponse {
    let payload = rmp_serde::to_vec_named(value).unwrap_or_default();
    InvokeResponse { version: VERSION, payload, ..Default::default() }
}

/// Build an error [`InvokeResponse`].
pub fn error_response(msg: &str) -> InvokeResponse {
    InvokeResponse { version: VERSION, error: msg.to_owned(), ..Default::default() }
}

/// Encode an error-only [`InvokeResponse`] to bytes.
pub fn marshal_error(msg: &str) -> Vec<u8> {
    rmp_serde::to_vec_named(&error_response(msg)).unwrap_or_default()
}

fn is_false(v: &bool) -> bool { !v }
