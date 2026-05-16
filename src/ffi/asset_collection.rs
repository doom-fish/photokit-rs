use core::ffi::c_char;

extern "C" {
    pub fn ph_asset_collection_fetch_all_json(
        fetch_options_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_asset_collection_fetch_with_type_json(
        collection_type: i32,
        collection_subtype: i64,
        fetch_options_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_asset_collection_fetch_with_local_identifiers_json(
        identifiers_json: *const c_char,
        fetch_options_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_asset_collection_fetch_containing_asset_json(
        asset_identifier: *const c_char,
        collection_type: i32,
        fetch_options_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_asset_collection_can_perform_edit_operation(
        collection_identifier: *const c_char,
        operation: i32,
        out_error: *mut *mut c_char,
    ) -> i32;
}
