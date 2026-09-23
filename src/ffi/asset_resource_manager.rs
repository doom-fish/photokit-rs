use core::ffi::{c_char, c_void};

use super::{ContextCallback, DataChunkCallback};

extern "C" {
    pub fn ph_asset_resource_manager_request_data(
        resource_json: *const c_char,
        options_json: *const c_char,
        timeout_ms: u64,
        chunk_callback: DataChunkCallback,
        sink_context: *mut c_void,
        context_retain: ContextCallback,
        context_release: ContextCallback,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_asset_resource_manager_write_data_json(
        resource_json: *const c_char,
        file_url: *const c_char,
        options_json: *const c_char,
        timeout_ms: u64,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
}
