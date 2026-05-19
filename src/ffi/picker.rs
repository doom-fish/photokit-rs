use core::ffi::{c_char, c_void};

use super::JsonCallback;

extern "C" {
    pub fn ph_picker_configuration_is_available() -> i32;
    pub fn ph_picker_filter_is_available() -> i32;
    pub fn ph_picker_result_is_available() -> i32;
    pub fn ph_picker_view_controller_is_available() -> i32;
    pub fn ph_picker_view_controller_delegate_is_available() -> i32;
    pub fn ph_picker_filter_description_json(
        filter_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_picker_view_controller_new(
        configuration_json: *const c_char,
        photo_library: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn ph_picker_view_controller_release(controller: *mut c_void);
    pub fn ph_picker_view_controller_register_delegate(
        controller: *mut c_void,
        callback: JsonCallback,
        user_info: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn ph_picker_view_controller_unregister_delegate(delegate: *mut c_void);
    pub fn ph_picker_view_controller_update_picker_json(
        controller: *mut c_void,
        configuration_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_picker_view_controller_deselect_assets_json(
        controller: *mut c_void,
        identifiers_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_picker_view_controller_move_asset(
        controller: *mut c_void,
        identifier: *const c_char,
        after_identifier: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_picker_view_controller_scroll_to_initial_position(
        controller: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_picker_view_controller_zoom_in(
        controller: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_picker_view_controller_zoom_out(
        controller: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;
}
