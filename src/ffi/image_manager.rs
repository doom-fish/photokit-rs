use core::ffi::{c_char, c_void};

extern "C" {
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
        request_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn ph_image_manager_request_live_photo(
        manager: *mut c_void,
        asset_identifier: *const c_char,
        request_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn ph_image_manager_request_player_item_for_video_json(
        manager: *mut c_void,
        asset_identifier: *const c_char,
        options_json: *const c_char,
        timeout_ms: u64,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_image_manager_request_export_session_for_video_json(
        manager: *mut c_void,
        asset_identifier: *const c_char,
        options_json: *const c_char,
        export_preset: *const c_char,
        timeout_ms: u64,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_image_manager_request_av_asset_for_video_json(
        manager: *mut c_void,
        asset_identifier: *const c_char,
        options_json: *const c_char,
        timeout_ms: u64,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
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
