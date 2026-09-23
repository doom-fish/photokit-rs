use core::ffi::{c_char, c_void};
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicUsize, Ordering};

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

/// Reference-counted FFI context shared between the Rust observer token and the
/// Swift bridge observer object.
///
/// The Swift observer takes a `+1` reference for its own lifetime (released in
/// its `deinit`), so a `photoLibraryDidChange` / availability callback already
/// dispatched on a background queue can never observe a freed callback box.
/// The Rust observer holds the initial reference and releases it on drop; the
/// box is only freed once both sides have released.
struct RefCounted<T> {
    value: T,
    ref_count: AtomicUsize,
}

impl<T> RefCounted<T> {
    fn into_raw(value: T) -> *mut Self {
        Box::into_raw(Box::new(Self {
            value,
            ref_count: AtomicUsize::new(1),
        }))
    }

    /// Increment the reference count.
    ///
    /// # Safety
    /// `ptr` must point to a valid, live `RefCounted<T>`.
    unsafe fn retain(ptr: *mut Self) {
        (*ptr).ref_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement the reference count, freeing the box if it reaches zero.
    ///
    /// # Safety
    /// `ptr` must point to a valid, live `RefCounted<T>`. After this call,
    /// `ptr` must not be used if the box was freed.
    unsafe fn release(ptr: *mut Self) {
        if ptr.is_null() {
            return;
        }
        if (*ptr).ref_count.fetch_sub(1, Ordering::Release) == 1 {
            // Acquire fence pairs with the Release stores from other threads'
            // `fetch_sub` calls so the freeing thread observes all prior writes.
            // This is the canonical Arc-style refcount drop; the fence is
            // required for soundness on weakly-ordered architectures.
            std::sync::atomic::fence(Ordering::Acquire);
            drop(Box::from_raw(ptr));
        }
    }
}

// C trampolines handed to Swift so each bridge observer object can take a +1
// reference on the Rust callback context for its own lifetime, then drop it in
// `deinit`. `release` null-checks internally.
extern "C" fn change_context_retain(user_info: *mut c_void) {
    if !user_info.is_null() {
        unsafe { RefCounted::<ChangeCallbackKind>::retain(user_info.cast()) };
    }
}

extern "C" fn change_context_release(user_info: *mut c_void) {
    unsafe { RefCounted::<ChangeCallbackKind>::release(user_info.cast()) };
}

extern "C" fn availability_context_retain(user_info: *mut c_void) {
    if !user_info.is_null() {
        unsafe { RefCounted::<Box<AvailabilityCallback>>::retain(user_info.cast()) };
    }
}

extern "C" fn availability_context_release(user_info: *mut c_void) {
    unsafe { RefCounted::<Box<AvailabilityCallback>>::release(user_info.cast()) };
}

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
            ffi::ph_request_authorization_for_access_level(access_level.as_raw(), &mut error)
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

    /// Wraps a Photos framework operation on `PHPhotoLibrary`.
    pub fn register_change_observer<F>(
        &self,
        callback: F,
    ) -> Result<PHChangeObserver, PhotoKitError>
    where
        F: Fn(PHPhotoLibraryChange) + Send + 'static,
    {
        self.register_change_observer_impl(ChangeCallbackKind::Summary(Box::new(callback)))
    }

    /// Wraps a Photos framework operation on `PHPhotoLibrary`.
    pub fn register_detailed_change_observer<F>(
        &self,
        callback: F,
    ) -> Result<PHChangeObserver, PhotoKitError>
    where
        F: Fn(PHChange) + Send + 'static,
    {
        self.register_change_observer_impl(ChangeCallbackKind::Detailed(Box::new(callback)))
    }

    /// Wraps a Photos framework operation on `PHPhotoLibrary`.
    pub fn register_availability_observer<F>(
        &self,
        callback: F,
    ) -> Result<PHAvailabilityObserver, PhotoKitError>
    where
        F: Fn(PHPhotoLibraryAvailabilityChange) + Send + 'static,
    {
        // SAFETY: `RefCounted::into_raw` produces a valid, non-null pointer.
        let user_info = unsafe {
            NonNull::new_unchecked(
                RefCounted::into_raw(Box::new(callback) as Box<AvailabilityCallback>)
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
                availability_context_retain,
                availability_context_release,
                &mut error,
            )
        };
        if let Some(raw) = NonNull::new(raw) {
            Ok(PHAvailabilityObserver { raw, user_info })
        } else {
            // SAFETY: Registration failed; the Swift bridge never took a
            // reference, so this drops the initial `+1` from `into_raw`.
            unsafe {
                RefCounted::<Box<AvailabilityCallback>>::release(user_info.as_ptr().cast());
            }
            // SAFETY: `error` is a non-null pointer set by the Swift bridge on failure.
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "register availability observer failed")
            })
        }
    }

    /// Wraps a Photos framework operation on `PHPhotoLibrary`.
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
        // SAFETY: `RefCounted::into_raw` produces a valid, non-null pointer.
        let user_info =
            unsafe { NonNull::new_unchecked(RefCounted::into_raw(callback).cast::<c_void>()) };
        let mut error = ptr::null_mut();
        // SAFETY: `self.raw` is a valid PHPhotoLibrary pointer, the trampoline
        // is a valid `extern "C"` fn, and `user_info` is a live heap allocation
        // whose lifetime is managed by `PHChangeObserver`.
        let raw = unsafe {
            ffi::ph_photo_library_register_change_observer(
                self.raw.as_ptr(),
                change_observer_trampoline,
                user_info.as_ptr(),
                change_context_retain,
                change_context_release,
                &mut error,
            )
        };

        if let Some(raw) = NonNull::new(raw) {
            Ok(PHChangeObserver { raw, user_info })
        } else {
            // SAFETY: Registration failed; the Swift bridge never took a
            // reference, so this drops the initial `+1` from `into_raw`.
            unsafe {
                RefCounted::<ChangeCallbackKind>::release(user_info.as_ptr().cast());
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

/// RAII registration token for `PHPhotoLibrary` change observations.
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
        // Swift bridge; unregister before dropping our reference below.
        unsafe { ffi::ph_photo_library_unregister_change_observer(self.raw.as_ptr()) };
        // SAFETY: `self.user_info` is a `RefCounted<ChangeCallbackKind>` created
        // via `into_raw`. This drops the initial `+1`; the Swift observer holds
        // its own reference (taken via `change_context_retain`) which it releases
        // in `deinit`, so an in-flight callback can never observe a freed box.
        unsafe {
            RefCounted::<ChangeCallbackKind>::release(self.user_info.as_ptr().cast());
        }
    }
}

/// RAII registration token for `PHPhotoLibrary` availability observations.
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
        // the Swift bridge can no longer call the trampoline before we drop
        // our reference below.
        unsafe { ffi::ph_photo_library_unregister_availability_observer(self.raw.as_ptr()) };
        // SAFETY: `self.user_info` is a `RefCounted<Box<AvailabilityCallback>>`
        // created via `into_raw`. This drops the initial `+1`; the Swift observer
        // releases its own reference in `deinit`, so an in-flight callback can
        // never observe a freed box.
        unsafe {
            RefCounted::<Box<AvailabilityCallback>>::release(self.user_info.as_ptr().cast());
        }
    }
}

unsafe extern "C" fn change_observer_trampoline(change: *mut c_void, user_info: *mut c_void) {
    // SAFETY: `user_info` is a `RefCounted<ChangeCallbackKind>` kept alive by
    // both the `PHChangeObserver` token and the Swift observer object (which
    // holds a `+1` for the duration of any in-flight callback). The trampoline
    // only borrows the inner callback.
    let callback = &(*(user_info.cast::<RefCounted<ChangeCallbackKind>>())).value;
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

    // SAFETY: `user_info` is a `RefCounted<Box<AvailabilityCallback>>` kept
    // alive by both the `PHAvailabilityObserver` token and the Swift observer
    // object (which holds a `+1` for the duration of any in-flight callback).
    let callback = &(*(user_info.cast::<RefCounted<Box<AvailabilityCallback>>>())).value;
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
