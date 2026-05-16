use core::ffi::{c_char, c_void};

extern "C" {
    pub fn ph_live_photo_request_with_resource_file_urls(
        file_urls_json: *const c_char,
        request_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_void;
}
