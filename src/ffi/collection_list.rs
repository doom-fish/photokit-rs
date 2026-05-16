use core::ffi::c_char;

extern "C" {
    pub fn ph_collection_list_fetch_containing_collection_json(
        collection_identifier: *const c_char,
        fetch_options_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_collection_list_fetch_with_local_identifiers_json(
        identifiers_json: *const c_char,
        fetch_options_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_collection_list_fetch_with_type_json(
        collection_list_type: i32,
        collection_list_subtype: i64,
        fetch_options_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_collection_list_can_perform_edit_operation(
        collection_identifier: *const c_char,
        operation: i32,
        out_error: *mut *mut c_char,
    ) -> i32;
}
