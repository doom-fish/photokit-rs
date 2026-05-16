use core::ffi::c_char;

extern "C" {
    pub fn ph_asset_fetch_all_json(
        fetch_options_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_asset_fetch_with_media_type_json(
        media_type: i32,
        fetch_options_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_asset_fetch_with_local_identifiers_json(
        identifiers_json: *const c_char,
        fetch_options_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_asset_fetch_in_collection_json(
        collection_identifier: *const c_char,
        fetch_options_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_asset_fetch_key_assets_in_collection_json(
        collection_identifier: *const c_char,
        fetch_options_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_asset_can_perform_edit_operation(
        asset_identifier: *const c_char,
        operation: i32,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_asset_resources_json(
        asset_identifier: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
}
