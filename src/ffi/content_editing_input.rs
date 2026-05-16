use core::ffi::{c_char, c_void};

extern "C" {
    pub fn ph_asset_request_content_editing_input(
        asset_identifier: *const c_char,
        options_json: *const c_char,
        timeout_ms: u64,
        out_error: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn ph_content_editing_input_json(
        input: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_content_editing_input_release(input: *mut c_void);
}
