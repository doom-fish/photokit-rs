use core::ffi::{c_char, c_void};

extern "C" {
    pub fn ph_change_asset_change_details_json(
        change: *mut c_void,
        asset_identifier: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_change_asset_collection_change_details_json(
        change: *mut c_void,
        collection_identifier: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_change_collection_list_change_details_json(
        change: *mut c_void,
        collection_identifier: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
}
