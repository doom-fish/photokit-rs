use core::ffi::{c_char, c_void};
use std::ffi::CStr;
use std::ops::Deref;
use std::ptr::{self, NonNull};

use serde::{Deserialize, Serialize};

use crate::content_editing_input::PHContentEditingInput;
use crate::content_editing_output::PHContentEditingOutput;
use crate::error::PhotoKitError;
use crate::ffi;
use crate::live_photo::PHLivePhotoResult;
use crate::private::parse_json_ptr;

type FrameProcessorCallback =
    dyn FnMut(PHLivePhotoFrame) -> PHLivePhotoFrameProcessingDecision + Send;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PHLivePhotoFrameType(pub i64);

impl PHLivePhotoFrameType {
    pub const PHOTO: Self = Self(0);
    pub const VIDEO: Self = Self(1);
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHLivePhotoFrame {
    pub frame_type: PHLivePhotoFrameType,
    pub time_seconds: f64,
    pub render_scale: f64,
    pub image_width: f64,
    pub image_height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PHLivePhotoFrameProcessingDecision {
    KeepOriginal,
    SkipFrame,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHLivePhotoEditingContextInfo {
    pub full_size_image_width: f64,
    pub full_size_image_height: f64,
    pub duration_seconds: f64,
    pub photo_time_seconds: f64,
    pub audio_volume: f32,
    pub orientation: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHLivePhotoEditingSaveResult {
    pub success: bool,
}

pub struct PHLivePhotoEditingContext {
    raw: NonNull<c_void>,
    info: PHLivePhotoEditingContextInfo,
    frame_processor_user_info: Option<NonNull<c_void>>,
}

impl PHLivePhotoEditingContext {
    pub fn new(input: &PHContentEditingInput) -> Result<Self, PhotoKitError> {
        let mut error = ptr::null_mut();
        let raw = unsafe { ffi::ph_live_photo_editing_context_new(input.raw.as_ptr(), &mut error) };
        let raw = NonNull::new(raw).ok_or_else(|| unsafe {
            PhotoKitError::from_error_ptr(error, "create live photo editing context failed")
        })?;
        let mut context = Self {
            raw,
            info: PHLivePhotoEditingContextInfo {
                full_size_image_width: 0.0,
                full_size_image_height: 0.0,
                duration_seconds: 0.0,
                photo_time_seconds: 0.0,
                audio_volume: 1.0,
                orientation: 0,
            },
            frame_processor_user_info: None,
        };
        context.refresh_info()?;
        Ok(context)
    }

    pub fn snapshot(&self) -> &PHLivePhotoEditingContextInfo {
        &self.info
    }

    pub fn set_audio_volume(&mut self, audio_volume: f32) -> Result<(), PhotoKitError> {
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_live_photo_editing_context_set_audio_volume(
                self.raw.as_ptr(),
                audio_volume,
                &mut error,
            )
        };
        if status == ffi::status::OK && error.is_null() {
            self.refresh_info()
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "set audio volume failed") })
        }
    }

    pub fn set_frame_processor<F>(&mut self, callback: F) -> Result<(), PhotoKitError>
    where
        F: FnMut(PHLivePhotoFrame) -> PHLivePhotoFrameProcessingDecision + Send + 'static,
    {
        self.clear_frame_processor();
        let callback: Box<FrameProcessorCallback> = Box::new(callback);
        let user_info =
            unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(callback)).cast::<c_void>()) };
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_live_photo_editing_context_set_frame_processor(
                self.raw.as_ptr(),
                live_photo_frame_processor_trampoline,
                user_info.as_ptr(),
                &mut error,
            )
        };
        if status == ffi::status::OK && error.is_null() {
            self.frame_processor_user_info = Some(user_info);
            Ok(())
        } else {
            unsafe {
                drop(Box::from_raw(
                    user_info.as_ptr().cast::<Box<FrameProcessorCallback>>(),
                ));
            }
            Err(unsafe { PhotoKitError::from_error_ptr(error, "set frame processor failed") })
        }
    }

    pub fn clear_frame_processor(&mut self) {
        unsafe { ffi::ph_live_photo_editing_context_clear_frame_processor(self.raw.as_ptr()) };
        if let Some(user_info) = self.frame_processor_user_info.take() {
            unsafe {
                drop(Box::from_raw(
                    user_info.as_ptr().cast::<Box<FrameProcessorCallback>>(),
                ));
            }
        }
    }

    pub fn prepare_live_photo_for_playback(
        &self,
        target_width: f64,
        target_height: f64,
        timeout_ms: u64,
    ) -> Result<PHLivePhotoResult, PhotoKitError> {
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_live_photo_editing_context_prepare_live_photo_json(
                self.raw.as_ptr(),
                target_width,
                target_height,
                timeout_ms,
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "prepare live photo for playback failed")
            })
        } else {
            unsafe { parse_json_ptr(payload, "PHLivePhotoResult") }
        }
    }

    pub fn save_live_photo_to_output(
        &self,
        output: &PHContentEditingOutput,
        timeout_ms: u64,
    ) -> Result<PHLivePhotoEditingSaveResult, PhotoKitError> {
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_live_photo_editing_context_save_json(
                self.raw.as_ptr(),
                output.as_raw(),
                timeout_ms,
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "save live photo failed") })
        } else {
            unsafe { parse_json_ptr(payload, "PHLivePhotoEditingSaveResult") }
        }
    }

    pub fn cancel(&self) {
        unsafe { ffi::ph_live_photo_editing_context_cancel(self.raw.as_ptr()) };
    }

    fn refresh_info(&mut self) -> Result<(), PhotoKitError> {
        let mut error = ptr::null_mut();
        let payload =
            unsafe { ffi::ph_live_photo_editing_context_json(self.raw.as_ptr(), &mut error) };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "live photo context snapshot failed")
            })
        } else {
            self.info = unsafe { parse_json_ptr(payload, "PHLivePhotoEditingContext") }?;
            Ok(())
        }
    }
}

impl Deref for PHLivePhotoEditingContext {
    type Target = PHLivePhotoEditingContextInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl core::fmt::Debug for PHLivePhotoEditingContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PHLivePhotoEditingContext")
            .field("info", &self.info)
            .finish_non_exhaustive()
    }
}

impl Drop for PHLivePhotoEditingContext {
    fn drop(&mut self) {
        self.clear_frame_processor();
        unsafe { ffi::ph_live_photo_editing_context_release(self.raw.as_ptr()) };
    }
}

unsafe extern "C" fn live_photo_frame_processor_trampoline(
    frame_json: *const c_char,
    user_info: *mut c_void,
) -> i32 {
    if frame_json.is_null() || user_info.is_null() {
        return 0;
    }

    let frame_json = CStr::from_ptr(frame_json).to_string_lossy();
    let Ok(frame) = serde_json::from_str::<PHLivePhotoFrame>(&frame_json) else {
        return 0;
    };
    let callback = &mut **user_info.cast::<Box<FrameProcessorCallback>>();
    match callback(frame) {
        PHLivePhotoFrameProcessingDecision::KeepOriginal => 0,
        PHLivePhotoFrameProcessingDecision::SkipFrame => 1,
    }
}
