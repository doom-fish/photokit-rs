use core::ffi::c_void;
use std::ptr::{self, NonNull};

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::asset::PHAsset;
use crate::error::PhotoKitError;
use crate::ffi;
use crate::live_photo::PHLivePhotoResult;
use crate::private::{cstring_from_str, json_cstring, parse_json_ptr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum PHImageContentMode {
    #[default]
    Default,
    AspectFit,
    AspectFill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PHImageRequestOptionsVersion {
    Current,
    Unadjusted,
    Original,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PHImageRequestOptionsDeliveryMode {
    Opportunistic,
    HighQualityFormat,
    FastFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PHImageRequestOptionsResizeMode {
    None,
    Fast,
    Exact,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHImageRequest {
    pub target_width: f64,
    pub target_height: f64,
    pub content_mode: PHImageContentMode,
    pub version: Option<PHImageRequestOptionsVersion>,
    pub delivery_mode: Option<PHImageRequestOptionsDeliveryMode>,
    pub resize_mode: Option<PHImageRequestOptionsResizeMode>,
    #[serde(default)]
    pub network_access_allowed: bool,
    #[serde(default)]
    pub synchronous: bool,
    #[serde(default)]
    pub allow_secondary_degraded_image: bool,
}

impl PHImageRequest {
    pub fn new(target_width: f64, target_height: f64, content_mode: PHImageContentMode) -> Self {
        Self {
            target_width,
            target_height,
            content_mode,
            version: None,
            delivery_mode: None,
            resize_mode: None,
            network_access_allowed: false,
            synchronous: false,
            allow_secondary_degraded_image: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHImageResult {
    pub tiff_data_base64: String,
    pub width: f64,
    pub height: f64,
    pub cancelled: bool,
    pub degraded: bool,
}

impl PHImageResult {
    pub fn tiff_data(&self) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(self.tiff_data_base64.as_bytes())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHImageDataResult {
    pub data_base64: String,
    pub uniform_type_identifier: Option<String>,
    #[serde(default)]
    pub content_type_identifier: Option<String>,
    pub orientation: i32,
    pub cancelled: bool,
    #[serde(default)]
    pub degraded: bool,
    #[serde(default)]
    pub is_in_cloud: bool,
}

impl PHImageDataResult {
    pub fn data(&self) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(self.data_base64.as_bytes())
            .unwrap_or_default()
    }
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
        request: &PHImageRequest,
    ) -> Result<PHImageDataRequestHandle, PhotoKitError> {
        let asset_identifier = cstring_from_str(&asset.local_identifier, "asset local identifier")?;
        let request_json = json_cstring(request, "PHImageRequest")?;
        let mut error = ptr::null_mut();
        let raw = unsafe {
            ffi::ph_image_manager_request_image_data(
                self.raw.as_ptr(),
                asset_identifier.as_ptr(),
                request_json.as_ptr(),
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
        let raw = NonNull::new(unsafe { ffi::ph_caching_image_manager_new() }).ok_or_else(|| {
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
        if status == ffi::status::OK && error.is_null() {
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
        if status == ffi::status::OK && error.is_null() {
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
    pub(crate) raw: NonNull<c_void>,
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
    pub(crate) raw: NonNull<c_void>,
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
