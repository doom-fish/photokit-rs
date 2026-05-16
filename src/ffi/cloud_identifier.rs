use core::ffi::{c_char, c_void};

extern "C" {
    pub fn ph_photo_library_cloud_identifier_mappings_json(
        library: *mut c_void,
        local_identifiers_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_photo_library_local_identifier_mappings_json(
        library: *mut c_void,
        cloud_identifiers_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
}
