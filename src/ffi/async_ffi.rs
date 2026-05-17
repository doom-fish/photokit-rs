//! FFI declarations for async (callback-based) Swift bridge thunks.
use core::ffi::{c_char, c_void};

/// Callback type for all async PhotoKit thunks.
/// Arguments: (result_json, error_cstr, ctx) — one of the first two is non-null.
pub type AsyncCallback =
    unsafe extern "C" fn(result: *const c_char, error: *const c_char, ctx: *mut c_void);

extern "C" {
    pub fn ph_photo_library_request_authorization_async(
        access_level: i32,
        cb: AsyncCallback,
        ctx: *mut c_void,
    );
    pub fn ph_asset_change_request_perform_async(
        payload_json: *const c_char,
        cb: AsyncCallback,
        ctx: *mut c_void,
    );
    pub fn ph_asset_collection_change_request_perform_async(
        payload_json: *const c_char,
        cb: AsyncCallback,
        ctx: *mut c_void,
    );
    pub fn ph_collection_list_change_request_perform_async(
        payload_json: *const c_char,
        cb: AsyncCallback,
        ctx: *mut c_void,
    );
    pub fn ph_image_manager_request_image_async(
        manager: *mut c_void,
        asset_identifier: *const c_char,
        request_json: *const c_char,
        cb: AsyncCallback,
        ctx: *mut c_void,
    );
    pub fn ph_image_manager_request_image_data_async(
        manager: *mut c_void,
        asset_identifier: *const c_char,
        request_json: *const c_char,
        cb: AsyncCallback,
        ctx: *mut c_void,
    );
    pub fn ph_live_photo_editing_context_save_async(
        context: *mut c_void,
        output: *mut c_void,
        cb: AsyncCallback,
        ctx: *mut c_void,
    );
    pub fn ph_live_photo_editing_context_prepare_async(
        context: *mut c_void,
        target_width: f64,
        target_height: f64,
        cb: AsyncCallback,
        ctx: *mut c_void,
    );
}
