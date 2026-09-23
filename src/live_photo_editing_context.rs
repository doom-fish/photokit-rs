use core::ffi::{c_char, c_void};
use std::ffi::CStr;
use std::ops::Deref;
use std::ptr::{self, NonNull};
use std::sync::{Mutex, PoisonError};

use doom_fish_utils::callback_context::CallbackContext;
use serde::{Deserialize, Serialize};

use crate::content_editing_input::PHContentEditingInput;
use crate::content_editing_output::PHContentEditingOutput;
use crate::error::PhotoKitError;
use crate::ffi;
use crate::live_photo::PHLivePhotoResult;
use crate::private::parse_json_ptr;

type FrameProcessorCallback =
    dyn FnMut(PHLivePhotoFrame) -> PHLivePhotoFrameProcessingDecision + Send;
type FrameProcessorContext = CallbackContext<Mutex<Box<FrameProcessorCallback>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
/// Wraps `PHLivePhotoFrameType`.
pub struct PHLivePhotoFrameType(
    /// Raw value for `PHLivePhotoFrameType`.
    pub i64,
);

impl PHLivePhotoFrameType {
    /// Constant on `PHLivePhotoFrameType`.
    pub const PHOTO: Self = Self(0);
    /// Constant on `PHLivePhotoFrameType`.
    pub const VIDEO: Self = Self(1);
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Serialized frame payload delivered to a `PHLivePhotoEditingContext` frame processor.
pub struct PHLivePhotoFrame {
    /// Corresponds to `PHLivePhotoFrame.frameType`.
    pub frame_type: PHLivePhotoFrameType,
    /// Corresponds to `PHLivePhotoFrame.timeSeconds`.
    pub time_seconds: f64,
    /// Corresponds to `PHLivePhotoFrame.renderScale`.
    pub render_scale: f64,
    /// Corresponds to `PHLivePhotoFrame.imageWidth`.
    pub image_width: f64,
    /// Corresponds to `PHLivePhotoFrame.imageHeight`.
    pub image_height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Decision returned from a `PHLivePhotoEditingContext` frame processor.
pub enum PHLivePhotoFrameProcessingDecision {
    /// Case of `PHLivePhotoFrameProcessingDecision`.
    KeepOriginal,
    /// Case of `PHLivePhotoFrameProcessingDecision`.
    SkipFrame,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Serialized snapshot of `PHLivePhotoEditingContext` properties.
pub struct PHLivePhotoEditingContextInfo {
    /// Corresponds to `PHLivePhotoEditingContextInfo.fullSizeImageWidth`.
    pub full_size_image_width: f64,
    /// Corresponds to `PHLivePhotoEditingContextInfo.fullSizeImageHeight`.
    pub full_size_image_height: f64,
    /// Corresponds to `PHLivePhotoEditingContextInfo.durationSeconds`.
    pub duration_seconds: f64,
    /// Corresponds to `PHLivePhotoEditingContextInfo.photoTimeSeconds`.
    pub photo_time_seconds: f64,
    /// Corresponds to `PHLivePhotoEditingContextInfo.audioVolume`.
    pub audio_volume: f32,
    /// Corresponds to `PHLivePhotoEditingContextInfo.orientation`.
    pub orientation: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Serialized result from `PHLivePhotoEditingContext.saveLivePhoto`.
pub struct PHLivePhotoEditingSaveResult {
    /// Corresponds to `PHLivePhotoEditingSaveResult.success`.
    pub success: bool,
}

/// Wraps `PHLivePhotoEditingContext`.
pub struct PHLivePhotoEditingContext {
    raw: NonNull<c_void>,
    info: PHLivePhotoEditingContextInfo,
    frame_processor: Option<FrameProcessorContext>,
}

impl PHLivePhotoEditingContext {
    /// Creates a helper value for the related Photos framework API.
    pub fn new(input: &PHContentEditingInput) -> Result<Self, PhotoKitError> {
        let mut error = ptr::null_mut();
        let raw =
            unsafe { ffi::ph_live_photo_editing_context_new(input.raw.as_ptr(), &raw mut error) };
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
            frame_processor: None,
        };
        context.refresh_info()?;
        Ok(context)
    }

    /// Returns the cached Photos framework snapshot for `PHLivePhotoEditingContext`.
    pub fn snapshot(&self) -> &PHLivePhotoEditingContextInfo {
        &self.info
    }

    /// Updates the wrapped Photos framework value on `PHLivePhotoEditingContext`.
    pub fn set_audio_volume(&mut self, audio_volume: f32) -> Result<(), PhotoKitError> {
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_live_photo_editing_context_set_audio_volume(
                self.raw.as_ptr(),
                audio_volume,
                &raw mut error,
            )
        };
        if status == ffi::status::OK && error.is_null() {
            self.refresh_info()
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "set audio volume failed") })
        }
    }

    /// Updates the wrapped Photos framework value on `PHLivePhotoEditingContext`.
    pub fn set_frame_processor<F>(&mut self, callback: F) -> Result<(), PhotoKitError>
    where
        F: FnMut(PHLivePhotoFrame) -> PHLivePhotoFrameProcessingDecision + Send + 'static,
    {
        self.clear_frame_processor();
        let callback: Box<FrameProcessorCallback> = Box::new(callback);
        let context = FrameProcessorContext::new(Mutex::new(callback));
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_live_photo_editing_context_set_frame_processor(
                self.raw.as_ptr(),
                live_photo_frame_processor_trampoline,
                context.as_ptr(),
                FrameProcessorContext::RETAIN,
                FrameProcessorContext::RELEASE,
                &raw mut error,
            )
        };
        if status == ffi::status::OK && error.is_null() {
            self.frame_processor = Some(context);
            Ok(())
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "set frame processor failed") })
        }
    }

    /// Clears Photos framework state on `PHLivePhotoEditingContext`.
    pub fn clear_frame_processor(&mut self) {
        if let Some(context) = self.frame_processor.take() {
            context.deactivate();
            unsafe { ffi::ph_live_photo_editing_context_clear_frame_processor(self.raw.as_ptr()) };
        }
    }

    /// Wraps a Photos framework operation on `PHLivePhotoEditingContext`.
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
                &raw mut error,
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

    /// Wraps a Photos framework operation on `PHLivePhotoEditingContext`.
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
                &raw mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "save live photo failed") })
        } else {
            unsafe { parse_json_ptr(payload, "PHLivePhotoEditingSaveResult") }
        }
    }

    /// Cancels the Photos framework operation represented by `PHLivePhotoEditingContext`.
    pub fn cancel(&self) {
        unsafe { ffi::ph_live_photo_editing_context_cancel(self.raw.as_ptr()) };
    }

    #[cfg(feature = "async")]
    pub(crate) fn as_raw(&self) -> *mut c_void {
        self.raw.as_ptr()
    }

    fn refresh_info(&mut self) -> Result<(), PhotoKitError> {
        let mut error = ptr::null_mut();
        let payload =
            unsafe { ffi::ph_live_photo_editing_context_json(self.raw.as_ptr(), &raw mut error) };
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
    if frame_json.is_null() {
        return 0;
    }

    FrameProcessorContext::with(user_info, "live_photo_frame_processor_trampoline", |callback| {
        let frame_json = CStr::from_ptr(frame_json).to_string_lossy();
        let Ok(frame) = serde_json::from_str::<PHLivePhotoFrame>(&frame_json) else {
            return 0;
        };
        let mut callback = callback.lock().unwrap_or_else(PoisonError::into_inner);
        match callback(frame) {
            PHLivePhotoFrameProcessingDecision::KeepOriginal => 0,
            PHLivePhotoFrameProcessingDecision::SkipFrame => 1,
        }
    })
    .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use super::{
        live_photo_frame_processor_trampoline, FrameProcessorCallback, FrameProcessorContext,
        PHLivePhotoFrame, PHLivePhotoFrameProcessingDecision, PHLivePhotoFrameType,
    };

    fn frame_json() -> CString {
        CString::new(
            r#"{"frameType":1,"timeSeconds":0.5,"renderScale":1.0,"imageWidth":4.0,"imageHeight":3.0}"#,
        )
        .unwrap()
    }

    fn processor(
        decision: PHLivePhotoFrameProcessingDecision,
        calls: &Arc<AtomicUsize>,
    ) -> FrameProcessorContext {
        let calls = Arc::clone(calls);
        let callback: Box<FrameProcessorCallback> = Box::new(move |frame: PHLivePhotoFrame| {
            assert_eq!(frame.frame_type, PHLivePhotoFrameType::VIDEO);
            calls.fetch_add(1, Ordering::SeqCst);
            decision
        });
        FrameProcessorContext::new(Mutex::new(callback))
    }

    #[test]
    fn trampoline_forwards_decisions_while_active() {
        let calls = Arc::new(AtomicUsize::new(0));
        let json = frame_json();
        let skip = processor(PHLivePhotoFrameProcessingDecision::SkipFrame, &calls);
        let keep = processor(PHLivePhotoFrameProcessingDecision::KeepOriginal, &calls);

        let skipped = unsafe { live_photo_frame_processor_trampoline(json.as_ptr(), skip.as_ptr()) };
        let kept = unsafe { live_photo_frame_processor_trampoline(json.as_ptr(), keep.as_ptr()) };

        assert_eq!(skipped, 1);
        assert_eq!(kept, 0);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn in_flight_render_after_clear_neither_runs_nor_frees_the_callback() {
        let calls = Arc::new(AtomicUsize::new(0));
        let json = frame_json();
        let context = processor(PHLivePhotoFrameProcessingDecision::SkipFrame, &calls);
        let block_copy = context.retained_ptr();

        drop(context);
        assert_eq!(Arc::strong_count(&calls), 2);

        let decision = unsafe { live_photo_frame_processor_trampoline(json.as_ptr(), block_copy) };
        assert_eq!(decision, 0);
        assert_eq!(calls.load(Ordering::SeqCst), 0);

        unsafe { (FrameProcessorContext::RELEASE)(block_copy) };
        assert_eq!(Arc::strong_count(&calls), 1);
    }

    #[test]
    fn trampoline_ignores_null_and_malformed_input() {
        let calls = Arc::new(AtomicUsize::new(0));
        let context = processor(PHLivePhotoFrameProcessingDecision::SkipFrame, &calls);
        let malformed = CString::new("{not json").unwrap();
        let json = frame_json();

        let results = unsafe {
            [
                live_photo_frame_processor_trampoline(std::ptr::null(), context.as_ptr()),
                live_photo_frame_processor_trampoline(json.as_ptr(), std::ptr::null_mut()),
                live_photo_frame_processor_trampoline(malformed.as_ptr(), context.as_ptr()),
            ]
        };

        assert_eq!(results, [0, 0, 0]);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn trampoline_contains_callback_panics() {
        let json = frame_json();
        let callback: Box<FrameProcessorCallback> = Box::new(|_frame| panic!("frame processor panic"));
        let context = FrameProcessorContext::new(Mutex::new(callback));

        let first = unsafe { live_photo_frame_processor_trampoline(json.as_ptr(), context.as_ptr()) };
        let second = unsafe { live_photo_frame_processor_trampoline(json.as_ptr(), context.as_ptr()) };

        assert_eq!((first, second), (0, 0));
        assert!(context.is_active());
    }
}
