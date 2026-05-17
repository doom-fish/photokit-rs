use core::ffi::{c_char, c_void};

use super::LivePhotoFrameProcessorCallback;

extern "C" {
    pub fn ph_live_photo_editing_context_new(
        input: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn ph_live_photo_editing_context_release(context: *mut c_void);
    pub fn ph_live_photo_editing_context_json(
        context: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_live_photo_editing_context_set_audio_volume(
        context: *mut c_void,
        audio_volume: f32,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_live_photo_editing_context_set_frame_processor(
        context: *mut c_void,
        callback: LivePhotoFrameProcessorCallback,
        user_info: *mut c_void,
        out_error: *mut *mut c_char,
    ) -> i32;
    pub fn ph_live_photo_editing_context_clear_frame_processor(context: *mut c_void);
    pub fn ph_live_photo_editing_context_prepare_live_photo_json(
        context: *mut c_void,
        target_width: f64,
        target_height: f64,
        timeout_ms: u64,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_live_photo_editing_context_save_json(
        context: *mut c_void,
        output: *mut c_void,
        timeout_ms: u64,
        out_error: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn ph_live_photo_editing_context_cancel(context: *mut c_void);
}
