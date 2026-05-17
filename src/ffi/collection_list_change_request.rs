use core::ffi::c_char;

extern "C" {
    pub fn ph_collection_list_change_request_perform_json(
        payload_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_collection_list_change_request_delete_json(
        identifiers_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
}
