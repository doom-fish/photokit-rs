use core::ffi::{c_char, c_void};
use std::ptr::{self, NonNull};

use doom_fish_utils::panic_safe::catch_user_panic;
use serde::{Deserialize, Serialize};

use crate::error::PhotoKitError;
use crate::ffi;
use crate::geometry::PHRect;
use crate::image_manager::PHImageRequest;
use crate::live_photo::PHLivePhotoResult;
use crate::private::{json_cstring, parse_json_ptr, take_string};

type LivePhotoViewDelegateCallback = dyn Fn(PHLivePhotoViewDelegateEvent) -> bool + Send;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHLivePhotoViewPlaybackStyle`.
pub enum PHLivePhotoViewPlaybackStyle {
    /// Case of `PHLivePhotoViewPlaybackStyle`.
    Undefined,
    /// Case of `PHLivePhotoViewPlaybackStyle`.
    Full,
    /// Case of `PHLivePhotoViewPlaybackStyle`.
    Hint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHLivePhotoViewContentMode`.
pub enum PHLivePhotoViewContentMode {
    /// Case of `PHLivePhotoViewContentMode`.
    AspectFit,
    /// Case of `PHLivePhotoViewContentMode`.
    AspectFill,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of the mutable state exposed by `PHLivePhotoView`.
pub struct PHLivePhotoViewInfo {
    /// Corresponds to whether `PHLivePhotoView.livePhoto` is non-nil.
    pub has_live_photo: bool,
    /// Corresponds to `PHLivePhotoView.livePhoto?.size.width`.
    pub live_photo_size_width: Option<f64>,
    /// Corresponds to `PHLivePhotoView.livePhoto?.size.height`.
    pub live_photo_size_height: Option<f64>,
    /// Corresponds to `PHLivePhotoView.contentMode`.
    pub content_mode: PHLivePhotoViewContentMode,
    /// Corresponds to `PHLivePhotoView.contentsRect`.
    pub contents_rect: Option<PHRect>,
    /// Corresponds to `PHLivePhotoView.audioVolume`.
    pub audio_volume: f32,
    /// Corresponds to `PHLivePhotoView.isMuted`.
    pub muted: bool,
    /// Corresponds to whether `PHLivePhotoView.livePhotoBadgeView` is non-nil.
    pub has_live_photo_badge_view: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Event kind delivered by `PHLivePhotoViewDelegate`.
pub enum PHLivePhotoViewDelegateEventKind {
    /// Corresponds to `livePhotoView(_:canBeginPlaybackWithStyle:)`.
    CanBegin,
    /// Corresponds to `livePhotoView(_:willBeginPlaybackWithStyle:)`.
    WillBegin,
    /// Corresponds to `livePhotoView(_:didEndPlaybackWithStyle:)`.
    DidEnd,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Event delivered by `PHLivePhotoViewDelegate` callbacks.
pub struct PHLivePhotoViewDelegateEvent {
    /// Indicates which delegate method fired.
    pub kind: PHLivePhotoViewDelegateEventKind,
    /// Corresponds to the delegate method playback style.
    pub playback_style: PHLivePhotoViewPlaybackStyle,
}

#[derive(Debug)]
/// Wraps `PHLivePhotoView`.
pub struct PHLivePhotoView {
    raw: NonNull<c_void>,
}

impl PHLivePhotoView {
    /// Returns whether `PHLivePhotoView` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_live_photo_view_is_available() == ffi::status::OK }
    }

    /// Creates a new `PHLivePhotoView`.
    pub fn new() -> Result<Self, PhotoKitError> {
        let mut error = ptr::null_mut();
        let raw = unsafe { ffi::ph_live_photo_view_new(&mut error) };
        let raw = NonNull::new(raw).ok_or_else(|| unsafe {
            PhotoKitError::from_error_ptr(error, "create PHLivePhotoView failed")
        })?;
        Ok(Self { raw })
    }

    /// Returns a PhotosUI snapshot of the view state.
    pub fn snapshot(&self) -> Result<PHLivePhotoViewInfo, PhotoKitError> {
        let mut error = ptr::null_mut();
        let payload = unsafe { ffi::ph_live_photo_view_json(self.raw.as_ptr(), &mut error) };
        if payload.is_null() {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "PHLivePhotoView snapshot failed") })
        } else {
            unsafe { parse_json_ptr(payload, "PHLivePhotoView") }
        }
    }

    /// Updates `contentMode`.
    pub fn set_content_mode(
        &mut self,
        content_mode: PHLivePhotoViewContentMode,
    ) -> Result<(), PhotoKitError> {
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_live_photo_view_set_content_mode(
                self.raw.as_ptr(),
                match content_mode {
                    PHLivePhotoViewContentMode::AspectFit => 0,
                    PHLivePhotoViewContentMode::AspectFill => 1,
                },
                &mut error,
            )
        };
        if status == ffi::status::OK && error.is_null() {
            Ok(())
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "set PHLivePhotoView content mode failed") })
        }
    }

    /// Updates `contentsRect`.
    pub fn set_contents_rect(&mut self, contents_rect: PHRect) -> Result<(), PhotoKitError> {
        let rect_json = json_cstring(&contents_rect, "PHRect")?;
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_live_photo_view_set_contents_rect_json(
                self.raw.as_ptr(),
                rect_json.as_ptr(),
                &mut error,
            )
        };
        if status == ffi::status::OK && error.is_null() {
            Ok(())
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "set PHLivePhotoView contents rect failed") })
        }
    }

    /// Updates `audioVolume`.
    pub fn set_audio_volume(&mut self, audio_volume: f32) -> Result<(), PhotoKitError> {
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_live_photo_view_set_audio_volume(self.raw.as_ptr(), audio_volume, &mut error)
        };
        if status == ffi::status::OK && error.is_null() {
            Ok(())
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "set PHLivePhotoView audio volume failed") })
        }
    }

    /// Updates `muted`.
    pub fn set_muted(&mut self, muted: bool) -> Result<(), PhotoKitError> {
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_live_photo_view_set_muted(self.raw.as_ptr(), muted, &mut error)
        };
        if status == ffi::status::OK && error.is_null() {
            Ok(())
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "set PHLivePhotoView muted failed") })
        }
    }

    /// Clears the live photo displayed by the view.
    pub fn clear_live_photo(&mut self) -> Result<(), PhotoKitError> {
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_live_photo_view_clear_live_photo(self.raw.as_ptr(), &mut error)
        };
        if status == ffi::status::OK && error.is_null() {
            Ok(())
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "clear PHLivePhotoView live photo failed") })
        }
    }

    /// Requests a live photo from resource file URLs and displays it in the view.
    pub fn request_live_photo_with_resource_file_urls(
        &mut self,
        file_urls: &[String],
        request: &PHImageRequest,
        timeout_ms: u64,
    ) -> Result<PHLivePhotoResult, PhotoKitError> {
        let file_urls_json = json_cstring(file_urls, "live photo resource urls")?;
        let request_json = json_cstring(request, "PHImageRequest")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_live_photo_view_request_with_resource_file_urls(
                self.raw.as_ptr(),
                file_urls_json.as_ptr(),
                request_json.as_ptr(),
                timeout_ms,
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "request PHLivePhotoView live photo failed")
            })
        } else {
            unsafe { parse_json_ptr(payload, "PHLivePhotoResult") }
        }
    }

    /// Starts playback using the requested style.
    pub fn start_playback_with_style(
        &self,
        playback_style: PHLivePhotoViewPlaybackStyle,
    ) -> Result<(), PhotoKitError> {
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_live_photo_view_start_playback(
                self.raw.as_ptr(),
                match playback_style {
                    PHLivePhotoViewPlaybackStyle::Undefined => 0,
                    PHLivePhotoViewPlaybackStyle::Full => 1,
                    PHLivePhotoViewPlaybackStyle::Hint => 2,
                },
                &mut error,
            )
        };
        if status == ffi::status::OK && error.is_null() {
            Ok(())
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "start PHLivePhotoView playback failed") })
        }
    }

    /// Stops playback immediately.
    pub fn stop_playback(&self) -> Result<(), PhotoKitError> {
        let mut error = ptr::null_mut();
        let status = unsafe { ffi::ph_live_photo_view_stop_playback(self.raw.as_ptr(), &mut error) };
        if status == ffi::status::OK && error.is_null() {
            Ok(())
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "stop PHLivePhotoView playback failed") })
        }
    }

    /// Stops playback, optionally animating the transition back to the still image.
    pub fn stop_playback_animated(&self, animated: bool) -> Result<(), PhotoKitError> {
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_live_photo_view_stop_playback_animated(self.raw.as_ptr(), animated, &mut error)
        };
        if status == ffi::status::OK && error.is_null() {
            Ok(())
        } else {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "stop PHLivePhotoView playback animated failed")
            })
        }
    }

    /// Registers a delegate callback for live photo view playback events.
    pub fn register_delegate<F>(
        &self,
        callback: F,
    ) -> Result<PHLivePhotoViewDelegate, PhotoKitError>
    where
        F: Fn(PHLivePhotoViewDelegateEvent) -> bool + Send + 'static,
    {
        let user_info = unsafe {
            NonNull::new_unchecked(
                Box::into_raw(Box::new(Box::new(callback) as Box<LivePhotoViewDelegateCallback>))
                    .cast::<c_void>(),
            )
        };
        let mut error = ptr::null_mut();
        let raw = unsafe {
            ffi::ph_live_photo_view_register_delegate(
                self.raw.as_ptr(),
                live_photo_view_delegate_trampoline,
                user_info.as_ptr(),
                &mut error,
            )
        };
        if let Some(raw) = NonNull::new(raw) {
            Ok(PHLivePhotoViewDelegate { raw, user_info })
        } else {
            unsafe {
                drop(Box::from_raw(
                    user_info
                        .as_ptr()
                        .cast::<Box<LivePhotoViewDelegateCallback>>(),
                ));
            }
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "register PHLivePhotoViewDelegate failed")
            })
        }
    }
}

impl Drop for PHLivePhotoView {
    fn drop(&mut self) {
        unsafe { ffi::ph_live_photo_view_release(self.raw.as_ptr()) };
    }
}

/// RAII registration token for `PHLivePhotoViewDelegate`.
pub struct PHLivePhotoViewDelegate {
    raw: NonNull<c_void>,
    user_info: NonNull<c_void>,
}

impl PHLivePhotoViewDelegate {
    /// Returns whether `PHLivePhotoViewDelegate` is available on the current SDK.
    pub fn is_available() -> bool {
        unsafe { ffi::ph_live_photo_view_delegate_is_available() == ffi::status::OK }
    }
}

impl core::fmt::Debug for PHLivePhotoViewDelegate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PHLivePhotoViewDelegate")
            .finish_non_exhaustive()
    }
}

impl Drop for PHLivePhotoViewDelegate {
    fn drop(&mut self) {
        unsafe { ffi::ph_live_photo_view_unregister_delegate(self.raw.as_ptr()) };
        unsafe {
            drop(Box::from_raw(
                self.user_info
                    .as_ptr()
                    .cast::<Box<LivePhotoViewDelegateCallback>>(),
            ));
        }
    }
}

unsafe extern "C" fn live_photo_view_delegate_trampoline(
    payload_json: *mut c_char,
    user_info: *mut c_void,
) -> i32 {
    if user_info.is_null() {
        return ffi::status::OK;
    }

    let callback = &mut **user_info.cast::<Box<LivePhotoViewDelegateCallback>>();
    let event = if payload_json.is_null() {
        PHLivePhotoViewDelegateEvent {
            kind: PHLivePhotoViewDelegateEventKind::CanBegin,
            playback_style: PHLivePhotoViewPlaybackStyle::Undefined,
        }
    } else if let Some(json) = take_string(payload_json) {
        serde_json::from_str(&json).unwrap_or(PHLivePhotoViewDelegateEvent {
            kind: PHLivePhotoViewDelegateEventKind::CanBegin,
            playback_style: PHLivePhotoViewPlaybackStyle::Undefined,
        })
    } else {
        PHLivePhotoViewDelegateEvent {
            kind: PHLivePhotoViewDelegateEventKind::CanBegin,
            playback_style: PHLivePhotoViewPlaybackStyle::Undefined,
        }
    };
    let mut decision = true;
    catch_user_panic("live_photo_view_delegate_trampoline", || {
        decision = callback(event);
    });
    i32::from(decision)
}
