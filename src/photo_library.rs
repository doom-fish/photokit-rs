use core::ffi::{c_char, c_void};
use std::ptr::{self, NonNull};

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
    const fn as_raw(self) -> i32 {
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
        let user_info = unsafe {
            NonNull::new_unchecked(
                Box::into_raw(Box::new(Box::new(callback) as Box<AvailabilityCallback>))
                    .cast::<c_void>(),
            )
        };
        let mut error = ptr::null_mut();
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
            unsafe {
                drop(Box::from_raw(
                    user_info.as_ptr().cast::<Box<AvailabilityCallback>>(),
                ));
            }
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
        let user_info =
            unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(callback)).cast::<c_void>()) };
        let mut error = ptr::null_mut();
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
            unsafe {
                drop(Box::from_raw(
                    user_info.as_ptr().cast::<ChangeCallbackKind>(),
                ));
            }
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
        unsafe { ffi::ph_photo_library_unregister_change_observer(self.raw.as_ptr()) };
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
        unsafe { ffi::ph_photo_library_unregister_availability_observer(self.raw.as_ptr()) };
        unsafe {
            drop(Box::from_raw(
                self.user_info.as_ptr().cast::<Box<AvailabilityCallback>>(),
            ));
        }
    }
}

unsafe extern "C" fn change_observer_trampoline(change: *mut c_void, user_info: *mut c_void) {
    let callback = &*(user_info.cast::<ChangeCallbackKind>());
    match callback {
        ChangeCallbackKind::Summary(callback) => {
            if let Some(change) = NonNull::new(change) {
                drop(PHChange::from_raw(change.as_ptr()));
            }
            callback(PHPhotoLibraryChange { change_count: 1 });
        }
        ChangeCallbackKind::Detailed(callback) => {
            if let Some(change) = NonNull::new(change) {
                callback(PHChange::from_raw(change.as_ptr()));
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

    let callback = &mut **user_info.cast::<Box<AvailabilityCallback>>();
    let payload = if payload_json.is_null() {
        PHPhotoLibraryAvailabilityChange::default()
    } else if let Some(json) = take_string(payload_json) {
        serde_json::from_str(&json).unwrap_or_default()
    } else {
        PHPhotoLibraryAvailabilityChange::default()
    };
    callback(payload);
}
