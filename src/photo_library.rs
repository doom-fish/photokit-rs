use core::ffi::{c_char, c_void};
use std::ptr::{self, NonNull};

use doom_fish_utils::panic_safe::catch_user_panic;
use serde::{Deserialize, Serialize};

use crate::asset::PHAsset;
use crate::asset_collection::PHAssetCollection;
use crate::change::PHChange;
use crate::error::{NSErrorInfo, PHAuthorizationStatus, PhotoKitError};
use crate::fetch_options::PHFetchOptions;
use crate::fetch_result::PHFetchResult;
use crate::ffi;
use crate::persistent_change::{PHPersistentChangeFetchResult, PHPersistentChangeToken};
use crate::private::{json_cstring, parse_json_ptr, take_string};

type SummaryChangeCallback = dyn Fn(PHPhotoLibraryChange) + Send;
type DetailedChangeCallback = dyn Fn(PHChange) + Send;
type AvailabilityCallback = dyn Fn(PHPhotoLibraryAvailabilityChange) + Send;

enum ChangeCallbackKind {
    Summary(Box<SummaryChangeCallback>),
    Detailed(Box<DetailedChangeCallback>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PHPhotoLibraryAvailabilityChange {
    pub unavailability_reason: Option<NSErrorInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PHAccessLevel {
    AddOnly,
    ReadWrite,
}

impl PHAccessLevel {
    pub(crate) const fn as_raw(self) -> i32 {
        match self {
            Self::AddOnly => 1,
            Self::ReadWrite => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PHPhotoLibraryChange {
    pub change_count: u64,
}

#[derive(Debug)]
pub struct PHPhotoLibrary {
    pub(crate) raw: NonNull<c_void>,
}

impl PHPhotoLibrary {
    pub fn shared() -> Result<Self, PhotoKitError> {
        let raw = NonNull::new(unsafe { ffi::ph_photo_library_shared() }).ok_or_else(|| {
            PhotoKitError::OperationFailed("failed to get shared PHPhotoLibrary".to_owned())
        })?;
        Ok(Self { raw })
    }

    pub fn authorization_status() -> PHAuthorizationStatus {
        PHAuthorizationStatus::from_raw(unsafe { ffi::ph_authorization_status() })
    }

    pub fn request_authorization() -> Result<PHAuthorizationStatus, PhotoKitError> {
        Self::request_authorization_for_access_level(PHAccessLevel::ReadWrite)
    }

    pub fn authorization_status_for_access_level(
        access_level: PHAccessLevel,
    ) -> PHAuthorizationStatus {
        PHAuthorizationStatus::from_raw(unsafe {
            ffi::ph_authorization_status_for_access_level(access_level.as_raw())
        })
    }

    pub fn request_authorization_for_access_level(
        access_level: PHAccessLevel,
    ) -> Result<PHAuthorizationStatus, PhotoKitError> {
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_request_authorization_for_access_level(access_level.as_raw(), &mut error)
        };
        if error.is_null() {
            Ok(PHAuthorizationStatus::from_raw(status))
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "request authorization failed") })
        }
    }

    pub fn fetch_asset_collections(
        &self,
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<PHAssetCollection>, PhotoKitError> {
        PHAssetCollection::fetch(fetch_options)
    }

    pub fn fetch_assets(
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<PHAsset>, PhotoKitError> {
        PHAsset::fetch(fetch_options)
    }

    pub fn unavailability_reason(&self) -> Result<Option<NSErrorInfo>, PhotoKitError> {
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_photo_library_unavailability_reason_json(self.raw.as_ptr(), &mut error)
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "unavailability reason lookup failed")
            })
        } else {
            unsafe { parse_json_ptr(payload, "PHPhotoLibrary unavailability reason") }
        }
    }

    pub fn register_change_observer<F>(
        &self,
        callback: F,
    ) -> Result<PHChangeObserver, PhotoKitError>
    where
        F: Fn(PHPhotoLibraryChange) + Send + 'static,
    {
        self.register_change_observer_impl(ChangeCallbackKind::Summary(Box::new(callback)))
    }

    pub fn register_detailed_change_observer<F>(
        &self,
        callback: F,
    ) -> Result<PHChangeObserver, PhotoKitError>
    where
        F: Fn(PHChange) + Send + 'static,
    {
        self.register_change_observer_impl(ChangeCallbackKind::Detailed(Box::new(callback)))
    }

    pub fn register_availability_observer<F>(
        &self,
        callback: F,
    ) -> Result<PHAvailabilityObserver, PhotoKitError>
    where
        F: Fn(PHPhotoLibraryAvailabilityChange) + Send + 'static,
    {
        // SAFETY: `Box::into_raw` produces a valid, non-null pointer.
        // The box is immediately wrapped in `NonNull::new_unchecked`, which is
        // safe because `Box::into_raw` never returns null.
        let user_info = unsafe {
            NonNull::new_unchecked(
                Box::into_raw(Box::new(Box::new(callback) as Box<AvailabilityCallback>))
                    .cast::<c_void>(),
            )
        };
        let mut error = ptr::null_mut();
        // SAFETY: `self.raw` is a valid PHPhotoLibrary pointer, the trampoline
        // function pointer is a valid `extern "C"` fn, and `user_info` is a
        // live heap allocation whose lifetime is managed by `PHAvailabilityObserver`.
        let raw = unsafe {
            ffi::ph_photo_library_register_availability_observer(
                self.raw.as_ptr(),
                availability_observer_trampoline,
                user_info.as_ptr(),
                &mut error,
            )
        };
        if let Some(raw) = NonNull::new(raw) {
            Ok(PHAvailabilityObserver { raw, user_info })
        } else {
            // SAFETY: Registration failed; `user_info` was never handed to the
            // Swift bridge so this is the only `from_raw` call on this pointer.
            unsafe {
                drop(Box::from_raw(
                    user_info.as_ptr().cast::<Box<AvailabilityCallback>>(),
                ));
            }
            // SAFETY: `error` is a non-null pointer set by the Swift bridge on failure.
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "register availability observer failed")
            })
        }
    }

    pub fn current_change_token(&self) -> Result<PHPersistentChangeToken, PhotoKitError> {
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_photo_library_current_change_token_json(self.raw.as_ptr(), &mut error)
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "current change token lookup failed")
            })
        } else {
            unsafe { parse_json_ptr(payload, "PHPersistentChangeToken") }
        }
    }

    pub fn fetch_persistent_changes_since_token(
        &self,
        token: &PHPersistentChangeToken,
    ) -> Result<PHPersistentChangeFetchResult, PhotoKitError> {
        let token_json = json_cstring(token, "PHPersistentChangeToken")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_photo_library_fetch_persistent_changes_since_token_json(
                self.raw.as_ptr(),
                token_json.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "persistent change fetch failed") })
        } else {
            unsafe { parse_json_ptr(payload, "PHPersistentChangeFetchResult") }
        }
    }

    fn register_change_observer_impl(
        &self,
        callback: ChangeCallbackKind,
    ) -> Result<PHChangeObserver, PhotoKitError> {
        // SAFETY: `Box::into_raw` produces a valid, non-null pointer.
        let user_info =
            unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(callback)).cast::<c_void>()) };
        let mut error = ptr::null_mut();
        // SAFETY: `self.raw` is a valid PHPhotoLibrary pointer, the trampoline
        // is a valid `extern "C"` fn, and `user_info` is a live heap allocation
        // whose lifetime is managed by `PHChangeObserver`.
        let raw = unsafe {
            ffi::ph_photo_library_register_change_observer(
                self.raw.as_ptr(),
                change_observer_trampoline,
                user_info.as_ptr(),
                &mut error,
            )
        };

        if let Some(raw) = NonNull::new(raw) {
            Ok(PHChangeObserver { raw, user_info })
        } else {
            // SAFETY: Registration failed; `user_info` was never handed to the
            // Swift bridge so this is the only `from_raw` call on this pointer.
            unsafe {
                drop(Box::from_raw(
                    user_info.as_ptr().cast::<ChangeCallbackKind>(),
                ));
            }
            // SAFETY: `error` is a non-null pointer set by the Swift bridge on failure.
            Err(unsafe { PhotoKitError::from_error_ptr(error, "registerChangeObserver failed") })
        }
    }
}

impl Drop for PHPhotoLibrary {
    fn drop(&mut self) {
        unsafe { ffi::ph_photo_library_release(self.raw.as_ptr()) };
    }
}

pub struct PHChangeObserver {
    raw: NonNull<c_void>,
    user_info: NonNull<c_void>,
}

impl core::fmt::Debug for PHChangeObserver {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PHChangeObserver").finish_non_exhaustive()
    }
}

impl Drop for PHChangeObserver {
    fn drop(&mut self) {
        // SAFETY: `self.raw` is a valid observer handle registered with the
        // Swift bridge; unregister before freeing the callback storage below.
        unsafe { ffi::ph_photo_library_unregister_change_observer(self.raw.as_ptr()) };
        // SAFETY: `self.user_info` was created from a `Box<ChangeCallbackKind>`
        // via `Box::into_raw` and this is the only `from_raw` call on it (the
        // Swift bridge never calls `from_raw`; the trampoline only borrows it).
        unsafe {
            drop(Box::from_raw(
                self.user_info.as_ptr().cast::<ChangeCallbackKind>(),
            ));
        }
    }
}

pub struct PHAvailabilityObserver {
    raw: NonNull<c_void>,
    user_info: NonNull<c_void>,
}

impl core::fmt::Debug for PHAvailabilityObserver {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PHAvailabilityObserver")
            .finish_non_exhaustive()
    }
}

impl Drop for PHAvailabilityObserver {
    fn drop(&mut self) {
        // SAFETY: `self.raw` is a valid observer handle; unregister first so
        // the Swift bridge can no longer call the trampoline before we free
        // the callback storage below.
        unsafe { ffi::ph_photo_library_unregister_availability_observer(self.raw.as_ptr()) };
        // SAFETY: `self.user_info` was created from `Box<Box<AvailabilityCallback>>`
        // via `Box::into_raw` and this is the only `from_raw` call on it.
        unsafe {
            drop(Box::from_raw(
                self.user_info.as_ptr().cast::<Box<AvailabilityCallback>>(),
            ));
        }
    }
}

unsafe extern "C" fn change_observer_trampoline(change: *mut c_void, user_info: *mut c_void) {
    // SAFETY: `user_info` is a `Box<ChangeCallbackKind>` kept alive by the
    // `PHChangeObserver` that owns this callback registration.  The trampoline
    // only borrows it; the box is freed in `PHChangeObserver::drop` after
    // `ph_photo_library_unregister_change_observer` returns.
    let callback = &*(user_info.cast::<ChangeCallbackKind>());
    match callback {
        ChangeCallbackKind::Summary(callback) => {
            if let Some(change) = NonNull::new(change) {
                // SAFETY: `change` is a valid PHChange pointer provided by the
                // Photos framework; `PHChange::from_raw` takes ownership and
                // we drop it immediately since the summary callback does not
                // expose the raw change object.
                drop(PHChange::from_raw(change.as_ptr()));
            }
            catch_user_panic("change_observer_trampoline(summary)", || {
                callback(PHPhotoLibraryChange { change_count: 1 });
            });
        }
        ChangeCallbackKind::Detailed(callback) => {
            if let Some(change) = NonNull::new(change) {
                // SAFETY: `change` is a valid PHChange pointer from Photos.
                let change_obj = PHChange::from_raw(change.as_ptr());
                catch_user_panic("change_observer_trampoline(detailed)", || {
                    callback(change_obj);
                });
            }
        }
    }
}

unsafe extern "C" fn availability_observer_trampoline(
    payload_json: *mut c_char,
    user_info: *mut c_void,
) {
    if user_info.is_null() {
        return;
    }

    // SAFETY: `user_info` is a `Box<Box<AvailabilityCallback>>` kept alive by
    // the `PHAvailabilityObserver` that owns this registration.
    let callback = &mut **user_info.cast::<Box<AvailabilityCallback>>();
    let payload = if payload_json.is_null() {
        PHPhotoLibraryAvailabilityChange::default()
    } else if let Some(json) = take_string(payload_json) {
        serde_json::from_str(&json).unwrap_or_default()
    } else {
        PHPhotoLibraryAvailabilityChange::default()
    };
    catch_user_panic("availability_observer_trampoline", || {
        callback(payload);
    });
}
