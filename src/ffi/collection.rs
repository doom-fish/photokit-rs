use core::ffi::c_char;

extern "C" {
    pub fn ph_collection_fetch_in_collection_list_json(
        collection_list_identifier: *const c_char,
        fetch_options_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_collection_fetch_top_level_user_collections_json(
        fetch_options_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_collection_can_perform_edit_operation(
        collection_identifier: *const c_char,
        operation: i32,
        out_error: *mut *mut c_char,
    ) -> i32;
}
