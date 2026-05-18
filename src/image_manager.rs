use core::ffi::c_void;
use std::ptr::{self, NonNull};

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::asset::PHAsset;
use crate::error::{NSErrorInfo, PhotoKitError};
use crate::ffi;
use crate::live_photo::PHLivePhotoResult;
use crate::private::{cstring_from_str, json_cstring, parse_json_ptr};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps the target size passed to `PHImageManager` image requests.
pub struct PHImageSize {
    /// Corresponds to `PHImageSize.width`.
    pub width: f64,
    /// Corresponds to `PHImageSize.height`.
    pub height: f64,
}

#[allow(non_upper_case_globals)]
/// Matches `PHImageManagerMaximumSize`.
pub const PHImageManagerMaximumSize: PHImageSize = PHImageSize {
    width: -1.0,
    height: -1.0,
};
#[allow(non_upper_case_globals)]
/// Matches `PHImageResultRequestIDKey`.
pub const PHImageResultRequestIDKey: &str = "PHImageResultRequestIDKey";
#[allow(non_upper_case_globals)]
/// Matches `PHImageErrorKey`.
pub const PHImageErrorKey: &str = "PHImageErrorKey";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHImageContentMode`.
pub enum PHImageContentMode {
    #[default]
    /// Case of `PHImageContentMode`.
    Default,
    /// Case of `PHImageContentMode`.
    AspectFit,
    /// Case of `PHImageContentMode`.
    AspectFill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHImageRequestOptionsVersion`.
pub enum PHImageRequestOptionsVersion {
    /// Case of `PHImageRequestOptionsVersion`.
    Current,
    /// Case of `PHImageRequestOptionsVersion`.
    Unadjusted,
    /// Case of `PHImageRequestOptionsVersion`.
    Original,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHImageRequestOptionsDeliveryMode`.
pub enum PHImageRequestOptionsDeliveryMode {
    /// Case of `PHImageRequestOptionsDeliveryMode`.
    Opportunistic,
    /// Case of `PHImageRequestOptionsDeliveryMode`.
    HighQualityFormat,
    /// Case of `PHImageRequestOptionsDeliveryMode`.
    FastFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHImageRequestOptionsResizeMode`.
pub enum PHImageRequestOptionsResizeMode {
    /// Case of `PHImageRequestOptionsResizeMode`.
    None,
    /// Case of `PHImageRequestOptionsResizeMode`.
    Fast,
    /// Case of `PHImageRequestOptionsResizeMode`.
    Exact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
/// Wraps `PHVideoRequestOptionsVersion`.
pub struct PHVideoRequestOptionsVersion(
    /// Raw value for `PHVideoRequestOptionsVersion`.
    pub i64,
);

impl PHVideoRequestOptionsVersion {
    /// Constant on `PHVideoRequestOptionsVersion`.
    pub const CURRENT: Self = Self(0);
    /// Constant on `PHVideoRequestOptionsVersion`.
    pub const ORIGINAL: Self = Self(1);
}

impl Default for PHVideoRequestOptionsVersion {
    fn default() -> Self {
        Self::CURRENT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
/// Wraps `PHVideoRequestOptionsDeliveryMode`.
pub struct PHVideoRequestOptionsDeliveryMode(
    /// Raw value for `PHVideoRequestOptionsDeliveryMode`.
    pub i64,
);

impl PHVideoRequestOptionsDeliveryMode {
    /// Constant on `PHVideoRequestOptionsDeliveryMode`.
    pub const AUTOMATIC: Self = Self(0);
    /// Constant on `PHVideoRequestOptionsDeliveryMode`.
    pub const HIGH_QUALITY_FORMAT: Self = Self(1);
    /// Constant on `PHVideoRequestOptionsDeliveryMode`.
    pub const MEDIUM_QUALITY_FORMAT: Self = Self(2);
    /// Constant on `PHVideoRequestOptionsDeliveryMode`.
    pub const FAST_FORMAT: Self = Self(3);
}

impl Default for PHVideoRequestOptionsDeliveryMode {
    fn default() -> Self {
        Self::AUTOMATIC
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHVideoRequestOptions`.
pub struct PHVideoRequestOptions {
    #[serde(default)]
    /// Corresponds to `PHVideoRequestOptions.networkAccessAllowed`.
    pub network_access_allowed: bool,
    #[serde(default)]
    /// Corresponds to `PHVideoRequestOptions.version`.
    pub version: PHVideoRequestOptionsVersion,
    #[serde(default)]
    /// Corresponds to `PHVideoRequestOptions.deliveryMode`.
    pub delivery_mode: PHVideoRequestOptionsDeliveryMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Request builder for `PHImageManager.requestImage`.
pub struct PHImageRequest {
    /// Corresponds to `PHImageRequest.targetWidth`.
    pub target_width: f64,
    /// Corresponds to `PHImageRequest.targetHeight`.
    pub target_height: f64,
    /// Corresponds to `PHImageRequest.contentMode`.
    pub content_mode: PHImageContentMode,
    /// Corresponds to `PHImageRequest.version`.
    pub version: Option<PHImageRequestOptionsVersion>,
    /// Corresponds to `PHImageRequest.deliveryMode`.
    pub delivery_mode: Option<PHImageRequestOptionsDeliveryMode>,
    /// Corresponds to `PHImageRequest.resizeMode`.
    pub resize_mode: Option<PHImageRequestOptionsResizeMode>,
    #[serde(default)]
    /// Corresponds to `PHImageRequest.networkAccessAllowed`.
    pub network_access_allowed: bool,
    #[serde(default)]
    /// Corresponds to `PHImageRequest.synchronous`.
    pub synchronous: bool,
    #[serde(default)]
    /// Corresponds to `PHImageRequest.allowSecondaryDegradedImage`.
    pub allow_secondary_degraded_image: bool,
}

impl PHImageRequest {
    /// Creates a helper value for the related Photos framework API.
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

    /// Wraps a Photos framework operation on `PHImageRequest`.
    pub fn maximum(content_mode: PHImageContentMode) -> Self {
        Self::new(
            PHImageManagerMaximumSize.width,
            PHImageManagerMaximumSize.height,
            content_mode,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Serialized result delivered by `PHImageManager.requestImage`.
pub struct PHImageResult {
    /// Corresponds to `PHImageResult.tiffDataBase64`.
    pub tiff_data_base64: String,
    /// Corresponds to `PHImageResult.width`.
    pub width: f64,
    /// Corresponds to `PHImageResult.height`.
    pub height: f64,
    /// Corresponds to `PHImageResult.cancelled`.
    pub cancelled: bool,
    /// Corresponds to `PHImageResult.degraded`.
    pub degraded: bool,
    #[serde(default)]
    /// Corresponds to `PHImageResult.requestId`.
    pub request_id: Option<i32>,
    #[serde(default)]
    /// Corresponds to `PHImageResult.error`.
    pub error: Option<NSErrorInfo>,
}

impl PHImageResult {
    /// Wraps a Photos framework operation on `PHImageResult`.
    pub fn tiff_data(&self) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(self.tiff_data_base64.as_bytes())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Serialized result delivered by `PHImageManager.requestImageDataAndOrientation`.
pub struct PHImageDataResult {
    /// Corresponds to `PHImageDataResult.dataBase64`.
    pub data_base64: String,
    /// Corresponds to `PHImageDataResult.uniformTypeIdentifier`.
    pub uniform_type_identifier: Option<String>,
    #[serde(default)]
    /// Corresponds to `PHImageDataResult.contentTypeIdentifier`.
    pub content_type_identifier: Option<String>,
    /// Corresponds to `PHImageDataResult.orientation`.
    pub orientation: i32,
    /// Corresponds to `PHImageDataResult.cancelled`.
    pub cancelled: bool,
    #[serde(default)]
    /// Corresponds to `PHImageDataResult.degraded`.
    pub degraded: bool,
    #[serde(default)]
    /// Corresponds to `PHImageDataResult.isInCloud`.
    pub is_in_cloud: bool,
    #[serde(default)]
    /// Corresponds to `PHImageDataResult.requestId`.
    pub request_id: Option<i32>,
    #[serde(default)]
    /// Corresponds to `PHImageDataResult.error`.
    pub error: Option<NSErrorInfo>,
}

impl PHImageDataResult {
    /// Decodes the binary data carried by `PHImageDataResult`.
    pub fn data(&self) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(self.data_base64.as_bytes())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Serialized result delivered by `PHImageManager` video request APIs.
pub struct PHVideoResult {
    /// Corresponds to `PHVideoResult.resultType`.
    pub result_type: String,
    #[serde(default)]
    /// Corresponds to `PHVideoResult.requestId`.
    pub request_id: Option<i32>,
    #[serde(default)]
    /// Corresponds to `PHVideoResult.cancelled`.
    pub cancelled: bool,
    #[serde(default)]
    /// Corresponds to `PHVideoResult.isInCloud`.
    pub is_in_cloud: bool,
    #[serde(default)]
    /// Corresponds to `PHVideoResult.error`.
    pub error: Option<NSErrorInfo>,
    #[serde(default)]
    /// Corresponds to `PHVideoResult.assetUrl`.
    pub asset_url: Option<String>,
    #[serde(default)]
    /// Corresponds to `PHVideoResult.durationSeconds`.
    pub duration_seconds: Option<f64>,
    #[serde(default)]
    /// Corresponds to `PHVideoResult.hasPlayerItem`.
    pub has_player_item: bool,
    #[serde(default)]
    /// Corresponds to `PHVideoResult.hasExportSession`.
    pub has_export_session: bool,
    #[serde(default)]
    /// Corresponds to `PHVideoResult.hasAvAsset`.
    pub has_av_asset: bool,
    #[serde(default)]
    /// Corresponds to `PHVideoResult.hasAudioMix`.
    pub has_audio_mix: bool,
    #[serde(default)]
    /// Corresponds to `PHVideoResult.exportPreset`.
    pub export_preset: Option<String>,
    #[serde(default)]
    /// Corresponds to `PHVideoResult.supportedFileTypes`.
    pub supported_file_types: Vec<String>,
}

#[derive(Debug)]
/// Wraps `PHImageManager`.
pub struct PHImageManager {
    raw: NonNull<c_void>,
}

impl PHImageManager {
    /// Returns the shared Photos framework `PHImageManager` instance.
    pub fn shared() -> Result<Self, PhotoKitError> {
        let raw = NonNull::new(unsafe { ffi::ph_image_manager_default() }).ok_or_else(|| {
            PhotoKitError::OperationFailed("failed to create PHImageManager".to_owned())
        })?;
        Ok(Self { raw })
    }

    /// Wraps a Photos framework request operation on `PHImageManager`.
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

    /// Wraps a Photos framework request operation on `PHImageManager`.
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

    /// Wraps a Photos framework request operation on `PHImageManager`.
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

    /// Wraps a Photos framework request operation on `PHImageManager`.
    pub fn request_player_item_for_video(
        &self,
        asset: &PHAsset,
        options: &PHVideoRequestOptions,
        timeout_ms: u64,
    ) -> Result<PHVideoResult, PhotoKitError> {
        self.request_video_json(
            asset,
            options,
            timeout_ms,
            None,
            ffi::ph_image_manager_request_player_item_for_video_json,
        )
    }

    /// Wraps a Photos framework request operation on `PHImageManager`.
    pub fn request_export_session_for_video(
        &self,
        asset: &PHAsset,
        options: &PHVideoRequestOptions,
        export_preset: &str,
        timeout_ms: u64,
    ) -> Result<PHVideoResult, PhotoKitError> {
        let export_preset = cstring_from_str(export_preset, "video export preset")?;
        let asset_identifier = cstring_from_str(&asset.local_identifier, "asset local identifier")?;
        let options_json = json_cstring(options, "PHVideoRequestOptions")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_image_manager_request_export_session_for_video_json(
                self.raw.as_ptr(),
                asset_identifier.as_ptr(),
                options_json.as_ptr(),
                export_preset.as_ptr(),
                timeout_ms,
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "requestExportSessionForVideo failed")
            })
        } else {
            unsafe { parse_json_ptr(payload, "PHVideoResult") }
        }
    }

    /// Wraps a Photos framework request operation on `PHImageManager`.
    pub fn request_av_asset_for_video(
        &self,
        asset: &PHAsset,
        options: &PHVideoRequestOptions,
        timeout_ms: u64,
    ) -> Result<PHVideoResult, PhotoKitError> {
        self.request_video_json(
            asset,
            options,
            timeout_ms,
            None,
            ffi::ph_image_manager_request_av_asset_for_video_json,
        )
    }

    fn request_video_json(
        &self,
        asset: &PHAsset,
        options: &PHVideoRequestOptions,
        timeout_ms: u64,
        export_preset: Option<&str>,
        request_fn: unsafe extern "C" fn(
            *mut c_void,
            *const core::ffi::c_char,
            *const core::ffi::c_char,
            u64,
            *mut *mut core::ffi::c_char,
        ) -> *mut core::ffi::c_char,
    ) -> Result<PHVideoResult, PhotoKitError> {
        debug_assert!(export_preset.is_none());
        let asset_identifier = cstring_from_str(&asset.local_identifier, "asset local identifier")?;
        let options_json = json_cstring(options, "PHVideoRequestOptions")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            request_fn(
                self.raw.as_ptr(),
                asset_identifier.as_ptr(),
                options_json.as_ptr(),
                timeout_ms,
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "video request failed") })
        } else {
            unsafe { parse_json_ptr(payload, "PHVideoResult") }
        }
    }
}

impl Drop for PHImageManager {
    fn drop(&mut self) {
        unsafe { ffi::ph_image_manager_release(self.raw.as_ptr()) };
    }
}

#[derive(Debug)]
/// Wraps `PHCachingImageManager`.
pub struct PHCachingImageManager {
    raw: NonNull<c_void>,
}

impl PHCachingImageManager {
    /// Creates a helper value for the related Photos framework API.
    pub fn new() -> Result<Self, PhotoKitError> {
        let raw =
            NonNull::new(unsafe { ffi::ph_caching_image_manager_new() }).ok_or_else(|| {
                PhotoKitError::OperationFailed("failed to create PHCachingImageManager".to_owned())
            })?;
        Ok(Self { raw })
    }

    /// Wraps a Photos framework operation on `PHCachingImageManager`.
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

    /// Wraps a Photos framework operation on `PHCachingImageManager`.
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

    /// Wraps a Photos framework operation on `PHCachingImageManager`.
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
/// Cancellable request handle returned by `PHImageManager.requestImage`.
pub struct PHImageRequestHandle {
    pub(crate) raw: NonNull<c_void>,
}

impl PHImageRequestHandle {
    /// Waits for the Photos framework operation represented by `PHImageRequestHandle` to finish.
    pub fn wait(&self, timeout_ms: u64) -> Result<PHImageResult, PhotoKitError> {
        wait_for_request(self.raw, timeout_ms, "PHImageResult")
    }

    /// Cancels the Photos framework operation represented by `PHImageRequestHandle`.
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
/// Cancellable request handle returned by `PHImageManager.requestImageData`.
pub struct PHImageDataRequestHandle {
    raw: NonNull<c_void>,
}

impl PHImageDataRequestHandle {
    /// Waits for the Photos framework operation represented by `PHImageDataRequestHandle` to finish.
    pub fn wait(&self, timeout_ms: u64) -> Result<PHImageDataResult, PhotoKitError> {
        wait_for_request(self.raw, timeout_ms, "PHImageDataResult")
    }

    /// Cancels the Photos framework operation represented by `PHImageDataRequestHandle`.
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
/// Cancellable request handle returned by `PHImageManager.requestLivePhoto`.
pub struct PHLivePhotoRequestHandle {
    pub(crate) raw: NonNull<c_void>,
}

impl PHLivePhotoRequestHandle {
    /// Waits for the Photos framework operation represented by `PHLivePhotoRequestHandle` to finish.
    pub fn wait(&self, timeout_ms: u64) -> Result<PHLivePhotoResult, PhotoKitError> {
        wait_for_request(self.raw, timeout_ms, "PHLivePhotoResult")
    }

    /// Cancels the Photos framework operation represented by `PHLivePhotoRequestHandle`.
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
