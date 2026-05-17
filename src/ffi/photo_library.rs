use core::ffi::{c_char, c_void};

use super::{ChangeObserverCallback, JsonCallback};

extern "C" {
    pub fn ph_authorization_status() -> i32;
    pub fn ph_authorization_status_for_access_level(access_level: i32) -> i32;
    pub fn ph_request_authorization_for_access_level(
        access_level: i32,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_photo_library_shared() -> *mut c_void;
    pub fn ph_photo_library_release(library: *mut c_void);
    pub fn ph_photo_library_unavailability_reason_json(
        library: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_photo_library_register_change_observer(
        library: *mut c_void,
        callback: ChangeObserverCallback,
        user_info: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn ph_photo_library_unregister_change_observer(observer: *mut c_void);
    pub fn ph_photo_library_register_availability_observer(
        library: *mut c_void,
        callback: JsonCallback,
        user_info: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn ph_photo_library_unregister_availability_observer(observer: *mut c_void);
    pub fn ph_photo_library_current_change_token_json(
        library: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_photo_library_fetch_persistent_changes_since_token_json(
        library: *mut c_void,
        token_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
}
