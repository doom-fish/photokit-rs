use core::ffi::{c_char, c_void};
use std::ptr::{self, NonNull};

use doom_fish_utils::callback_context::CallbackContext;
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

type SummaryChangeCallback = dyn Fn(PHPhotoLibraryChange) + Send + Sync;
type DetailedChangeCallback = dyn Fn(PHChange) + Send + Sync;
type AvailabilityCallback = dyn Fn(PHPhotoLibraryAvailabilityChange) + Send + Sync;

enum ChangeCallbackKind {
    Summary(Box<SummaryChangeCallback>),
    Detailed(Box<DetailedChangeCallback>),
}

type ChangeContext = CallbackContext<ChangeCallbackKind>;
type AvailabilityContext = CallbackContext<Box<AvailabilityCallback>>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
/// Serialized payload delivered by `PHPhotoLibrary` availability observers.
pub struct PHPhotoLibraryAvailabilityChange {
    /// Corresponds to `PHPhotoLibraryAvailabilityChange.unavailabilityReason`.
    pub unavailability_reason: Option<NSErrorInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHAccessLevel`.
pub enum PHAccessLevel {
    /// Case of `PHAccessLevel`.
    AddOnly,
    /// Case of `PHAccessLevel`.
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
/// Summary payload delivered by `PHPhotoLibrary` change observers.
pub struct PHPhotoLibraryChange {
    /// Corresponds to `PHPhotoLibraryChange.changeCount`.
    pub change_count: u64,
}

#[derive(Debug)]
/// Wraps `PHPhotoLibrary`.
pub struct PHPhotoLibrary {
    pub(crate) raw: NonNull<c_void>,
}

impl PHPhotoLibrary {
    /// Returns the shared Photos framework `PHPhotoLibrary` instance.
    pub fn shared() -> Result<Self, PhotoKitError> {
        let raw = NonNull::new(unsafe { ffi::ph_photo_library_shared() }).ok_or_else(|| {
            PhotoKitError::OperationFailed("failed to get shared PHPhotoLibrary".to_owned())
        })?;
        Ok(Self { raw })
    }

    /// Wraps a Photos framework operation on `PHPhotoLibrary`.
    pub fn authorization_status() -> PHAuthorizationStatus {
        PHAuthorizationStatus::from_raw(unsafe { ffi::ph_authorization_status() })
    }

    /// Wraps a Photos framework request operation on `PHPhotoLibrary`.
    pub fn request_authorization() -> Result<PHAuthorizationStatus, PhotoKitError> {
        Self::request_authorization_for_access_level(PHAccessLevel::ReadWrite)
    }

    /// Wraps a Photos framework operation on `PHPhotoLibrary`.
    pub fn authorization_status_for_access_level(
        access_level: PHAccessLevel,
    ) -> PHAuthorizationStatus {
        PHAuthorizationStatus::from_raw(unsafe {
            ffi::ph_authorization_status_for_access_level(access_level.as_raw())
        })
    }

    /// Wraps a Photos framework request operation on `PHPhotoLibrary`.
    pub fn request_authorization_for_access_level(
        access_level: PHAccessLevel,
    ) -> Result<PHAuthorizationStatus, PhotoKitError> {
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_request_authorization_for_access_level(access_level.as_raw(), &raw mut error)
        };
        if error.is_null() {
            Ok(PHAuthorizationStatus::from_raw(status))
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "request authorization failed") })
        }
    }

    /// Wraps a Photos framework fetch operation on `PHPhotoLibrary`.
    pub fn fetch_asset_collections(
        &self,
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<PHAssetCollection>, PhotoKitError> {
        PHAssetCollection::fetch(fetch_options)
    }

    /// Wraps a Photos framework fetch operation on `PHPhotoLibrary`.
    pub fn fetch_assets(
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<PHAsset>, PhotoKitError> {
        PHAsset::fetch(fetch_options)
    }

    /// Wraps a Photos framework operation on `PHPhotoLibrary`.
    pub fn unavailability_reason(&self) -> Result<Option<NSErrorInfo>, PhotoKitError> {
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_photo_library_unavailability_reason_json(self.raw.as_ptr(), &raw mut error)
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "unavailability reason lookup failed")
            })
        } else {
            unsafe { parse_json_ptr(payload, "PHPhotoLibrary unavailability reason") }
        }
    }

    /// Wraps a Photos framework operation on `PHPhotoLibrary`.
    pub fn register_change_observer<F>(
        &self,
        callback: F,
    ) -> Result<PHChangeObserver, PhotoKitError>
    where
        F: Fn(PHPhotoLibraryChange) + Send + Sync + 'static,
    {
        self.register_change_observer_impl(ChangeCallbackKind::Summary(Box::new(callback)))
    }

    /// Wraps a Photos framework operation on `PHPhotoLibrary`.
    pub fn register_detailed_change_observer<F>(
        &self,
        callback: F,
    ) -> Result<PHChangeObserver, PhotoKitError>
    where
        F: Fn(PHChange) + Send + Sync + 'static,
    {
        self.register_change_observer_impl(ChangeCallbackKind::Detailed(Box::new(callback)))
    }

    /// Wraps a Photos framework operation on `PHPhotoLibrary`.
    pub fn register_availability_observer<F>(
        &self,
        callback: F,
    ) -> Result<PHAvailabilityObserver, PhotoKitError>
    where
        F: Fn(PHPhotoLibraryAvailabilityChange) + Send + Sync + 'static,
    {
        let context = AvailabilityContext::new(Box::new(callback));
        let mut error = ptr::null_mut();
        let raw = unsafe {
            ffi::ph_photo_library_register_availability_observer(
                self.raw.as_ptr(),
                availability_observer_trampoline,
                context.as_ptr(),
                AvailabilityContext::RETAIN,
                AvailabilityContext::RELEASE,
                &raw mut error,
            )
        };
        match NonNull::new(raw) {
            Some(raw) => Ok(PHAvailabilityObserver { raw, context }),
            None => Err(unsafe {
                PhotoKitError::from_error_ptr(error, "register availability observer failed")
            }),
        }
    }

    /// Wraps a Photos framework operation on `PHPhotoLibrary`.
    pub fn current_change_token(&self) -> Result<PHPersistentChangeToken, PhotoKitError> {
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_photo_library_current_change_token_json(self.raw.as_ptr(), &raw mut error)
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "current change token lookup failed")
            })
        } else {
            unsafe { parse_json_ptr(payload, "PHPersistentChangeToken") }
        }
    }

    /// Wraps a Photos framework fetch operation on `PHPhotoLibrary`.
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
                &raw mut error,
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
        let context = ChangeContext::new(callback);
        let mut error = ptr::null_mut();
        let raw = unsafe {
            ffi::ph_photo_library_register_change_observer(
                self.raw.as_ptr(),
                change_observer_trampoline,
                context.as_ptr(),
                ChangeContext::RETAIN,
                ChangeContext::RELEASE,
                &raw mut error,
            )
        };
        match NonNull::new(raw) {
            Some(raw) => Ok(PHChangeObserver { raw, context }),
            None => Err(unsafe {
                PhotoKitError::from_error_ptr(error, "registerChangeObserver failed")
            }),
        }
    }
}

impl Drop for PHPhotoLibrary {
    fn drop(&mut self) {
        unsafe { ffi::ph_photo_library_release(self.raw.as_ptr()) };
    }
}

/// RAII registration token for `PHPhotoLibrary` change observations.
pub struct PHChangeObserver {
    raw: NonNull<c_void>,
    context: ChangeContext,
}

impl core::fmt::Debug for PHChangeObserver {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PHChangeObserver").finish_non_exhaustive()
    }
}

impl Drop for PHChangeObserver {
    fn drop(&mut self) {
        self.context.deactivate();
        unsafe { ffi::ph_photo_library_unregister_change_observer(self.raw.as_ptr()) };
    }
}

/// RAII registration token for `PHPhotoLibrary` availability observations.
pub struct PHAvailabilityObserver {
    raw: NonNull<c_void>,
    context: AvailabilityContext,
}

impl core::fmt::Debug for PHAvailabilityObserver {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PHAvailabilityObserver")
            .finish_non_exhaustive()
    }
}

impl Drop for PHAvailabilityObserver {
    fn drop(&mut self) {
        self.context.deactivate();
        unsafe { ffi::ph_photo_library_unregister_availability_observer(self.raw.as_ptr()) };
    }
}

unsafe extern "C" fn change_observer_trampoline(change: *mut c_void, user_info: *mut c_void) {
    let change = NonNull::new(change).map(|change| PHChange::from_raw(change.as_ptr()));
    let _ = ChangeContext::with(user_info, "change_observer_trampoline", move |callback| {
        match callback {
            ChangeCallbackKind::Summary(callback) => {
                drop(change);
                callback(PHPhotoLibraryChange { change_count: 1 });
            }
            ChangeCallbackKind::Detailed(callback) => {
                if let Some(change) = change {
                    callback(change);
                }
            }
        }
    });
}

unsafe extern "C" fn availability_observer_trampoline(
    payload_json: *mut c_char,
    user_info: *mut c_void,
) {
    let payload = if payload_json.is_null() {
        PHPhotoLibraryAvailabilityChange::default()
    } else if let Some(json) = take_string(payload_json) {
        serde_json::from_str(&json).unwrap_or_default()
    } else {
        PHPhotoLibraryAvailabilityChange::default()
    };
    let _ = AvailabilityContext::with(user_info, "availability_observer_trampoline", |callback| {
        callback(payload);
    });
}

#[cfg(test)]
mod tests {
    use std::ptr;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use super::{change_observer_trampoline, ChangeCallbackKind, ChangeContext};

    #[test]
    fn change_notifications_stop_once_the_observer_is_dropped() {
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&calls);
        let context = ChangeContext::new(ChangeCallbackKind::Summary(Box::new(move |change| {
            if change.change_count == 1 {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        })));
        let swift_reference = context.retained_ptr();

        unsafe { change_observer_trampoline(ptr::null_mut(), swift_reference) };
        context.deactivate();
        unsafe { change_observer_trampoline(ptr::null_mut(), swift_reference) };
        drop(context);
        unsafe { change_observer_trampoline(ptr::null_mut(), swift_reference) };

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(Arc::strong_count(&calls), 2);
        unsafe { (ChangeContext::RELEASE)(swift_reference) };
        assert_eq!(Arc::strong_count(&calls), 1);
    }

    #[test]
    fn detailed_observers_ignore_a_missing_change() {
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&calls);
        let context = ChangeContext::new(ChangeCallbackKind::Detailed(Box::new(move |_change| {
            counter.fetch_add(1, Ordering::SeqCst);
        })));

        unsafe { change_observer_trampoline(ptr::null_mut(), context.as_ptr()) };

        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
}
