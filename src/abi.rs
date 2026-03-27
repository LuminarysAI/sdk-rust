//! Raw platform imports and internal helpers.

use crate::types::*;

#[cfg(target_arch = "wasm32")]
pub(crate) mod raw {
    #[link(wasm_import_module = "env")]
    extern "C" {
        pub fn history_get(ptr: u32, len: u32) -> u64;
        pub fn prompt_complete(ptr: u32, len: u32) -> u64;
        pub fn env_get(ptr: u32, len: u32) -> u64;
        pub fn fs_read(ptr: u32, len: u32) -> u64;
        pub fn fs_write(ptr: u32, len: u32) -> u64;
        pub fn fs_create(ptr: u32, len: u32) -> u64;
        pub fn fs_delete(ptr: u32, len: u32) -> u64;
        pub fn fs_mkdir(ptr: u32, len: u32) -> u64;
        pub fn fs_ls(ptr: u32, len: u32) -> u64;
        pub fn fs_chmod(ptr: u32, len: u32) -> u64;
        pub fn fs_read_lines(ptr: u32, len: u32) -> u64;
        pub fn fs_grep(ptr: u32, len: u32) -> u64;
        pub fn fs_glob(ptr: u32, len: u32) -> u64;
        pub fn http_get(ptr: u32, len: u32) -> u64;
        pub fn http_post(ptr: u32, len: u32) -> u64;
        pub fn http_request(ptr: u32, len: u32) -> u64;
        pub fn ws_connect(ptr: u32, len: u32) -> u64;
        pub fn ws_send(ptr: u32, len: u32) -> u64;
        pub fn ws_close(ptr: u32, len: u32) -> u64;
        pub fn tcp_connect(ptr: u32, len: u32) -> u64;
        pub fn tcp_set_callback(ptr: u32, len: u32) -> u64;
        pub fn tcp_write(ptr: u32, len: u32) -> u64;
        pub fn tcp_close(ptr: u32, len: u32) -> u64;
        pub fn tcp_request(ptr: u32, len: u32) -> u64;
        pub fn shell_exec(ptr: u32, len: u32) -> u64;
        pub fn log_write(ptr: u32, len: u32) -> u64;
        pub fn sys_info(ptr: u32, len: u32) -> u64;
        pub fn time_now(ptr: u32, len: u32) -> u64;
        pub fn disk_usage(ptr: u32, len: u32) -> u64;
        pub fn fs_allowed_dirs(ptr: u32, len: u32) -> u64;
        pub fn fs_copy(ptr: u32, len: u32) -> u64;
        pub fn archive_pack(ptr: u32, len: u32) -> u64;
        pub fn archive_unpack(ptr: u32, len: u32) -> u64;
        pub fn archive_list(ptr: u32, len: u32) -> u64;
        pub fn file_transfer_send(ptr: u32, len: u32) -> u64;
        pub fn file_transfer_recv(ptr: u32, len: u32) -> u64;
        pub fn cluster_node_list(ptr: u32, len: u32) -> u64;
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod raw {
    macro_rules! stub {
        ($($name:ident),* $(,)?) => {
            $(pub unsafe extern "C" fn $name(_ptr: u32, _len: u32) -> u64 {
                panic!(concat!(stringify!($name), " called outside WASM"))
            })*
        };
    }
    stub!(
        history_get, prompt_complete, env_get,
        fs_read, fs_write, fs_create, fs_delete, fs_mkdir,
        fs_ls, fs_chmod, fs_read_lines, fs_grep, fs_glob, fs_copy,
        fs_allowed_dirs,
        http_get, http_post, http_request,
        ws_connect, ws_send, ws_close,
        tcp_connect, tcp_set_callback, tcp_write, tcp_close, tcp_request,
        shell_exec,
        log_write,
        sys_info, time_now, disk_usage,
        archive_pack, archive_unpack, archive_list,
        file_transfer_send, file_transfer_recv, cluster_node_list,
    );
}

pub(crate) fn call_host<S, R>(
    host_fn: unsafe extern "C" fn(u32, u32) -> u64,
    value: &S,
) -> Result<R, SkillError>
where
    S: serde::Serialize,
    R: serde::de::DeserializeOwned,
{
    let encoded = rmp_serde::to_vec_named(value)?;
    let result = unsafe { host_fn(encoded.as_ptr() as u32, encoded.len() as u32) };
    let bytes = unpack_result(result);

    #[derive(serde::Deserialize)]
    struct ErrorCheck {
        #[serde(default)]
        error: String,
    }
    if let Ok(check) = rmp_serde::from_slice::<ErrorCheck>(&bytes) {
        if !check.error.is_empty() {
            return Err(SkillError(check.error));
        }
    }

    Ok(rmp_serde::from_slice(&bytes)?)
}

fn unpack_result(packed: u64) -> Vec<u8> {
    let ptr = (packed >> 32) as u32;
    let len = (packed & 0xFFFF_FFFF) as u32;
    if len == 0 {
        return Vec::new();
    }
    unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize).to_vec() }
}
