use core::ffi::c_void;
use std::ptr::{self, NonNull};

use crate::error::{PHAuthorizationStatus, PhotoKitError};
use crate::ffi;
use crate::private::{cstring_from_str, json_cstring, parse_json_ptr};
use crate::types::{
    PHAsset, PHAssetCollection, PHAssetResource, PHFetchOptions, PHFetchResult, PHImageDataResult,
    PHImageRequest, PHImageResult, PHLivePhotoResult, PHPhotoLibraryChange,
};

type ChangeCallback = dyn Fn(PHPhotoLibraryChange) + Send;
type ChangeCallbackBox = Box<ChangeCallback>;

#[derive(Debug)]
pub struct PHPhotoLibrary {
    raw: NonNull<c_void>,
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
        let mut error = ptr::null_mut();
        let status = unsafe { ffi::ph_request_authorization(&mut error) };
        if error.is_null() {
            Ok(PHAuthorizationStatus::from_raw(status))
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "requestAuthorization failed") })
        }
    }

    pub fn fetch_asset_collections(
        &self,
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<PHAssetCollection>, PhotoKitError> {
        let options_json = json_cstring(fetch_options, "PHFetchOptions")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_photo_library_fetch_asset_collections_json(options_json.as_ptr(), &mut error)
        };
        if payload.is_null() {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "fetch asset collections failed") })
        } else {
            let collections: Vec<PHAssetCollection> =
                unsafe { parse_json_ptr(payload, "PHAssetCollection list") }?;
            Ok(collections.into())
        }
    }

    pub fn fetch_assets(
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<PHAsset>, PhotoKitError> {
        let options_json = json_cstring(fetch_options, "PHFetchOptions")?;
        let mut error = ptr::null_mut();
        let payload =
            unsafe { ffi::ph_photo_library_fetch_assets_json(options_json.as_ptr(), &mut error) };
        if payload.is_null() {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "fetch assets failed") })
        } else {
            let assets: Vec<PHAsset> = unsafe { parse_json_ptr(payload, "PHAsset list") }?;
            Ok(assets.into())
        }
    }

    pub fn register_change_observer<F>(
        &self,
        callback: F,
    ) -> Result<PHChangeObserver, PhotoKitError>
    where
        F: Fn(PHPhotoLibraryChange) + Send + 'static,
    {
        let callback: ChangeCallbackBox = Box::new(callback);
        let user_info = unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(callback)).cast::<c_void>())
        };
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
                drop(Box::from_raw(user_info.as_ptr().cast::<ChangeCallbackBox>()));
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
                self.user_info.as_ptr().cast::<ChangeCallbackBox>(),
            ));
        }
    }
}

unsafe extern "C" fn change_observer_trampoline(user_info: *mut c_void) {
    let callback = &*(user_info.cast::<ChangeCallbackBox>());
    callback(PHPhotoLibraryChange { change_count: 1 });
}

#[derive(Debug)]
pub struct PHImageManager {
    raw: NonNull<c_void>,
}

impl PHImageManager {
    pub fn shared() -> Result<Self, PhotoKitError> {
        let raw = NonNull::new(unsafe { ffi::ph_image_manager_default() }).ok_or_else(|| {
            PhotoKitError::OperationFailed("failed to create PHImageManager".to_owned())
        })?;
        Ok(Self { raw })
    }

    pub fn request_image(
        &self,
        asset: &PHAsset,
        request: PHImageRequest,
    ) -> Result<PHImageRequestHandle, PhotoKitError> {
        let asset_identifier = cstring_from_str(&asset.local_identifier, "asset local identifier")?;
        let request_json = json_cstring(&request, "PHImageRequest")?;
        let mut error = ptr::null_mut();
        let raw = unsafe {
            ffi::ph_image_manager_request_image(
                self.raw.as_ptr(),
                asset_identifier.as_ptr(),
                request_json.as_ptr(),
                &mut error,
            )
        };
        NonNull::new(raw)
            .map(|raw| PHImageRequestHandle { raw })
            .ok_or_else(|| unsafe { PhotoKitError::from_error_ptr(error, "requestImage failed") })
    }

    pub fn request_image_data(
        &self,
        asset: &PHAsset,
    ) -> Result<PHImageDataRequestHandle, PhotoKitError> {
        let asset_identifier = cstring_from_str(&asset.local_identifier, "asset local identifier")?;
        let mut error = ptr::null_mut();
        let raw = unsafe {
            ffi::ph_image_manager_request_image_data(
                self.raw.as_ptr(),
                asset_identifier.as_ptr(),
                &mut error,
            )
        };
        NonNull::new(raw)
            .map(|raw| PHImageDataRequestHandle { raw })
            .ok_or_else(|| unsafe {
                PhotoKitError::from_error_ptr(error, "requestImageData failed")
            })
    }

    pub fn request_live_photo(
        &self,
        asset: &PHAsset,
        request: PHImageRequest,
    ) -> Result<PHLivePhotoRequestHandle, PhotoKitError> {
        let asset_identifier = cstring_from_str(&asset.local_identifier, "asset local identifier")?;
        let request_json = json_cstring(&request, "PHImageRequest")?;
        let mut error = ptr::null_mut();
        let raw = unsafe {
            ffi::ph_image_manager_request_live_photo(
                self.raw.as_ptr(),
                asset_identifier.as_ptr(),
                request_json.as_ptr(),
                &mut error,
            )
        };
        NonNull::new(raw)
            .map(|raw| PHLivePhotoRequestHandle { raw })
            .ok_or_else(|| unsafe {
                PhotoKitError::from_error_ptr(error, "requestLivePhoto failed")
            })
    }
}

impl Drop for PHImageManager {
    fn drop(&mut self) {
        unsafe { ffi::ph_image_manager_release(self.raw.as_ptr()) };
    }
}

#[derive(Debug)]
pub struct PHCachingImageManager {
    raw: NonNull<c_void>,
}

impl PHCachingImageManager {
    pub fn new() -> Result<Self, PhotoKitError> {
        let raw =
            NonNull::new(unsafe { ffi::ph_caching_image_manager_new() }).ok_or_else(|| {
                PhotoKitError::OperationFailed("failed to create PHCachingImageManager".to_owned())
            })?;
        Ok(Self { raw })
    }

    pub fn start_caching_images(
        &self,
        assets: &[PHAsset],
        request: &PHImageRequest,
    ) -> Result<(), PhotoKitError> {
        let identifiers: Vec<String> = assets
            .iter()
            .map(|asset| asset.local_identifier.clone())
            .collect();
        let identifiers_json = json_cstring(&identifiers, "asset identifiers")?;
        let request_json = json_cstring(request, "PHImageRequest")?;
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_caching_image_manager_start_caching(
                self.raw.as_ptr(),
                identifiers_json.as_ptr(),
                request_json.as_ptr(),
                &mut error,
            )
        };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "startCachingImages failed") })
        }
    }

    pub fn stop_caching_images(
        &self,
        assets: &[PHAsset],
        request: &PHImageRequest,
    ) -> Result<(), PhotoKitError> {
        let identifiers: Vec<String> = assets
            .iter()
            .map(|asset| asset.local_identifier.clone())
            .collect();
        let identifiers_json = json_cstring(&identifiers, "asset identifiers")?;
        let request_json = json_cstring(request, "PHImageRequest")?;
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_caching_image_manager_stop_caching(
                self.raw.as_ptr(),
                identifiers_json.as_ptr(),
                request_json.as_ptr(),
                &mut error,
            )
        };
        if status == ffi::status::OK {
            Ok(())
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "stopCachingImages failed") })
        }
    }

    pub fn stop_caching_images_for_all_assets(&self) {
        unsafe { ffi::ph_caching_image_manager_stop_caching_all(self.raw.as_ptr()) };
    }
}

impl Drop for PHCachingImageManager {
    fn drop(&mut self) {
        unsafe { ffi::ph_image_manager_release(self.raw.as_ptr()) };
    }
}

#[derive(Debug)]
pub struct PHImageRequestHandle {
    raw: NonNull<c_void>,
}

impl PHImageRequestHandle {
    pub fn wait(&self, timeout_ms: u64) -> Result<PHImageResult, PhotoKitError> {
        wait_for_request(self.raw, timeout_ms, "PHImageResult")
    }

    pub fn cancel(&self) {
        unsafe { ffi::ph_image_request_cancel(self.raw.as_ptr()) };
    }
}

impl Drop for PHImageRequestHandle {
    fn drop(&mut self) {
        unsafe { ffi::ph_image_request_release(self.raw.as_ptr()) };
    }
}

#[derive(Debug)]
pub struct PHImageDataRequestHandle {
    raw: NonNull<c_void>,
}

impl PHImageDataRequestHandle {
    pub fn wait(&self, timeout_ms: u64) -> Result<PHImageDataResult, PhotoKitError> {
        wait_for_request(self.raw, timeout_ms, "PHImageDataResult")
    }

    pub fn cancel(&self) {
        unsafe { ffi::ph_image_request_cancel(self.raw.as_ptr()) };
    }
}

impl Drop for PHImageDataRequestHandle {
    fn drop(&mut self) {
        unsafe { ffi::ph_image_request_release(self.raw.as_ptr()) };
    }
}

#[derive(Debug)]
pub struct PHLivePhotoRequestHandle {
    raw: NonNull<c_void>,
}

impl PHLivePhotoRequestHandle {
    pub fn wait(&self, timeout_ms: u64) -> Result<PHLivePhotoResult, PhotoKitError> {
        wait_for_request(self.raw, timeout_ms, "PHLivePhotoResult")
    }

    pub fn cancel(&self) {
        unsafe { ffi::ph_image_request_cancel(self.raw.as_ptr()) };
    }
}

impl Drop for PHLivePhotoRequestHandle {
    fn drop(&mut self) {
        unsafe { ffi::ph_image_request_release(self.raw.as_ptr()) };
    }
}

fn wait_for_request<T: serde::de::DeserializeOwned>(
    raw: NonNull<c_void>,
    timeout_ms: u64,
    context: &str,
) -> Result<T, PhotoKitError> {
    let mut error = ptr::null_mut();
    let payload = unsafe { ffi::ph_image_request_wait_json(raw.as_ptr(), timeout_ms, &mut error) };
    if payload.is_null() {
        Err(unsafe { PhotoKitError::from_error_ptr(error, "request wait failed") })
    } else {
        unsafe { parse_json_ptr(payload, context) }
    }
}

impl PHAsset {
    pub fn resources(&self) -> Result<Vec<PHAssetResource>, PhotoKitError> {
        let identifier = cstring_from_str(&self.local_identifier, "asset local identifier")?;
        let mut error = ptr::null_mut();
        let payload = unsafe { ffi::ph_asset_resources_json(identifier.as_ptr(), &mut error) };
        if payload.is_null() {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "asset resources failed") })
        } else {
            unsafe { parse_json_ptr(payload, "PHAssetResource list") }
        }
    }
}
