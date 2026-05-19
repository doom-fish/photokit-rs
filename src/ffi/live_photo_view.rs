use core::ffi::{c_char, c_void};

use super::BoolJsonCallback;

extern "C" {
    pub fn ph_live_photo_view_is_available() -> i32;
    pub fn ph_live_photo_view_delegate_is_available() -> i32;
    pub fn ph_live_photo_view_new(out_error: *mut *mut c_char) -> *mut c_void;
    pub fn ph_live_photo_view_release(view: *mut c_void);
    pub fn ph_live_photo_view_json(
        view: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_live_photo_view_set_content_mode(
        view: *mut c_void,
        content_mode: i32,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_live_photo_view_set_contents_rect_json(
        view: *mut c_void,
        rect_json: *const c_char,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_live_photo_view_set_audio_volume(
        view: *mut c_void,
        audio_volume: f32,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_live_photo_view_set_muted(
        view: *mut c_void,
        muted: bool,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_live_photo_view_clear_live_photo(
        view: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_live_photo_view_request_with_resource_file_urls(
        view: *mut c_void,
        file_urls_json: *const c_char,
        request_json: *const c_char,
        timeout_ms: u64,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_live_photo_view_start_playback(
        view: *mut c_void,
        playback_style: i32,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_live_photo_view_stop_playback(
        view: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_live_photo_view_stop_playback_animated(
        view: *mut c_void,
        animated: bool,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_live_photo_view_register_delegate(
        view: *mut c_void,
        callback: BoolJsonCallback,
        user_info: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn ph_live_photo_view_unregister_delegate(delegate: *mut c_void);
}
