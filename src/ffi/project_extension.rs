use core::ffi::{c_char, c_void};

extern "C" {
    pub fn ph_project_type_description_is_available() -> i32;
    pub fn ph_project_type_description_data_source_is_available() -> i32;
    pub fn ph_project_type_description_invalidator_is_available() -> i32;
    pub fn ph_project_extension_context_is_available() -> i32;
    pub fn ph_project_extension_controller_is_available() -> i32;
    pub fn ph_project_extension_context_release(context: *mut c_void);
    pub fn ph_project_extension_context_project_json(
        context: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_project_extension_context_photo_library(
        context: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn ph_project_extension_context_show_editor_for_asset(
        context: *mut c_void,
        asset_identifier: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_project_extension_context_updated_project_info_json(
        context: *mut c_void,
        timeout_ms: u64,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
}
