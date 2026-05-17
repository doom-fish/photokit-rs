use core::ffi::c_char;

extern "C" {
    pub fn ph_project_fetch_top_level_json(
        fetch_options_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_project_fetch_with_local_identifiers_json(
        identifiers_json: *const c_char,
        fetch_options_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_project_change_request_perform_json(
        payload_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
}
