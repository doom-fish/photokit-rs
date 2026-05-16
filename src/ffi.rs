#![allow(missing_docs)]

use core::ffi::{c_char, c_void};

extern "C" {
    pub fn ph_string_free(string: *mut c_char);

    pub fn ph_authorization_status() -> i32;
    pub fn ph_request_authorization(out_error: *mut *mut c_char) -> i32;

    pub fn ph_photo_library_shared() -> *mut c_void;
    pub fn ph_photo_library_release(library: *mut c_void);
    pub fn ph_photo_library_fetch_asset_collections_json(
        fetch_options_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_photo_library_fetch_assets_json(
        fetch_options_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_photo_library_register_change_observer(
        library: *mut c_void,
        callback: ChangeObserverCallback,
        user_info: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn ph_photo_library_unregister_change_observer(observer: *mut c_void);

    pub fn ph_asset_resources_json(
        asset_identifier: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;

    pub fn ph_image_manager_default() -> *mut c_void;
    pub fn ph_caching_image_manager_new() -> *mut c_void;
    pub fn ph_image_manager_release(manager: *mut c_void);
    pub fn ph_image_manager_request_image(
        manager: *mut c_void,
        asset_identifier: *const c_char,
        request_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn ph_image_manager_request_image_data(
        manager: *mut c_void,
        asset_identifier: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn ph_image_manager_request_live_photo(
        manager: *mut c_void,
        asset_identifier: *const c_char,
        request_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn ph_image_request_wait_json(
        request: *mut c_void,
        timeout_ms: u64,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_image_request_cancel(request: *mut c_void);
    pub fn ph_image_request_release(request: *mut c_void);

    pub fn ph_caching_image_manager_start_caching(
        manager: *mut c_void,
        asset_identifiers_json: *const c_char,
        request_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_caching_image_manager_stop_caching(
        manager: *mut c_void,
        asset_identifiers_json: *const c_char,
        request_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_caching_image_manager_stop_caching_all(manager: *mut c_void);
}

pub type ChangeObserverCallback = unsafe extern "C" fn(user_info: *mut c_void);

pub mod status {
    pub const OK: i32 = 0;
}
