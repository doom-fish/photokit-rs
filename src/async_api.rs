//! Async API for `photokit` (Photos.framework)
//!
//! This module wraps Photos.framework completion-handler APIs as executor-agnostic
//! [`std::future::Future`] types. Enable with the `async` Cargo feature.
//!
//! ## Available Types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`AsyncPHPhotoLibrary`](crate::async_api::AsyncPHPhotoLibrary) | Async authorization + `performChanges` entrypoints |
//! | [`AsyncPHAssetChangeRequest`](crate::async_api::AsyncPHAssetChangeRequest) | Async asset change requests |
//! | [`AsyncPHAssetCollectionChangeRequest`](crate::async_api::AsyncPHAssetCollectionChangeRequest) | Async asset-collection change requests |
//! | [`AsyncPHCollectionListChangeRequest`](crate::async_api::AsyncPHCollectionListChangeRequest) | Async collection-list change requests |
//! | [`AsyncPHImageManager`](crate::async_api::AsyncPHImageManager) | Async image + image-data requests |
//! | [`AsyncPHLivePhotoEditingContext`](crate::async_api::AsyncPHLivePhotoEditingContext) | Async live photo save + prepare |
//!
//! ## Notes
//! - `PHCachingImageManager::start/stop_caching_images` are synchronous; use the
//!   sync API from [`crate::image_manager`] directly.
//! - Multi-fire delegate callbacks (change observers, availability observers) are
//!   stream patterns and deferred to Tier 2.
//!
//! ## Example
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use photokit::async_api::AsyncPHPhotoLibrary;
//! use photokit::PHAccessLevel;
//!
//! pollster::block_on(async {
//!     let status = AsyncPHPhotoLibrary::request_authorization(PHAccessLevel::ReadWrite).await?;
//!     println!("Status: {status:?}");
//!     Ok::<_, Box<dyn std::error::Error>>(())
//! })
//! # }
//! ```

use std::ffi::{c_void, CStr};
use std::future::Future;
use std::pin::Pin;
use std::ptr::NonNull;
use std::task::{Context, Poll};

use doom_fish_utils::completion::{error_from_cstr, AsyncCompletion, AsyncCompletionFuture};
use serde::Deserialize;

use crate::asset::PHAsset;
use crate::asset_change_request::PHAssetChangeRequest;
use crate::asset_collection_change_request::PHAssetCollectionChangeRequest;
use crate::collection_list_change_request::PHCollectionListChangeRequest;
use crate::content_editing_output::PHContentEditingOutput;
use crate::error::{PHAuthorizationStatus, PhotoKitError};
use crate::image_manager::{PHImageDataResult, PHImageRequest, PHImageResult};
use crate::live_photo::PHLivePhotoResult;
use crate::live_photo_editing_context::{PHLivePhotoEditingContext, PHLivePhotoEditingSaveResult};
use crate::photo_library::PHAccessLevel;
use crate::private::{cstring_from_str, json_cstring};

macro_rules! define_json_callback {
    ($name:ident, $t:ty) => {
        unsafe extern "C" fn $name(
            result: *const core::ffi::c_char,
            error: *const core::ffi::c_char,
            ctx: *mut c_void,
        ) {
            if !error.is_null() {
                let message = error_from_cstr(error);
                AsyncCompletion::<$t>::complete_err(ctx, message);
            } else if !result.is_null() {
                let json = CStr::from_ptr(result).to_string_lossy();
                match serde_json::from_str::<$t>(json.as_ref()) {
                    Ok(value) => AsyncCompletion::complete_ok(ctx, value),
                    Err(err) => AsyncCompletion::<$t>::complete_err(ctx, err.to_string()),
                }
            } else {
                AsyncCompletion::<$t>::complete_err(
                    ctx,
                    "no result or error from async callback".to_owned(),
                );
            }
        }
    };
}

#[derive(Debug, Deserialize)]
struct AuthStatusPayload {
    status: i32,
}

define_json_callback!(auth_status_callback, AuthStatusPayload);

/// Future for [`AsyncPHPhotoLibrary::request_authorization`].
pub struct RequestAuthorizationFuture {
    inner: AsyncCompletionFuture<AuthStatusPayload>,
}

impl core::fmt::Debug for RequestAuthorizationFuture {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RequestAuthorizationFuture")
            .finish_non_exhaustive()
    }
}

impl Future for RequestAuthorizationFuture {
    type Output = Result<PHAuthorizationStatus, PhotoKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|result| {
            result
                .map(|payload| PHAuthorizationStatus::from_raw(payload.status))
                .map_err(PhotoKitError::OperationFailed)
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChangeResultPayload {
    placeholder_local_identifier: Option<String>,
}

define_json_callback!(change_result_callback, ChangeResultPayload);

/// Future for `performChanges`-backed async mutation requests.
pub struct PerformChangesFuture {
    inner: AsyncCompletionFuture<ChangeResultPayload>,
}

impl core::fmt::Debug for PerformChangesFuture {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PerformChangesFuture")
            .finish_non_exhaustive()
    }
}

impl Future for PerformChangesFuture {
    type Output = Result<Option<String>, PhotoKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|result| {
            result
                .map(|payload| payload.placeholder_local_identifier)
                .map_err(PhotoKitError::OperationFailed)
        })
    }
}

define_json_callback!(image_result_callback, PHImageResult);
define_json_callback!(image_data_result_callback, PHImageDataResult);

/// Future for [`AsyncPHImageManager::request_image`].
pub struct RequestImageFuture {
    inner: AsyncCompletionFuture<PHImageResult>,
}

impl core::fmt::Debug for RequestImageFuture {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RequestImageFuture").finish_non_exhaustive()
    }
}

impl Future for RequestImageFuture {
    type Output = Result<PHImageResult, PhotoKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|result| result.map_err(PhotoKitError::OperationFailed))
    }
}

/// Future for [`AsyncPHImageManager::request_image_data`].
pub struct RequestImageDataFuture {
    inner: AsyncCompletionFuture<PHImageDataResult>,
}

impl core::fmt::Debug for RequestImageDataFuture {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RequestImageDataFuture")
            .finish_non_exhaustive()
    }
}

impl Future for RequestImageDataFuture {
    type Output = Result<PHImageDataResult, PhotoKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|result| result.map_err(PhotoKitError::OperationFailed))
    }
}

define_json_callback!(live_photo_save_callback, PHLivePhotoEditingSaveResult);
define_json_callback!(live_photo_prepare_callback, PHLivePhotoResult);

/// Future for [`AsyncPHLivePhotoEditingContext::save_live_photo`].
pub struct SaveLivePhotoFuture {
    inner: AsyncCompletionFuture<PHLivePhotoEditingSaveResult>,
}

impl core::fmt::Debug for SaveLivePhotoFuture {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SaveLivePhotoFuture")
            .finish_non_exhaustive()
    }
}

impl Future for SaveLivePhotoFuture {
    type Output = Result<PHLivePhotoEditingSaveResult, PhotoKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|result| result.map_err(PhotoKitError::OperationFailed))
    }
}

/// Future for [`AsyncPHLivePhotoEditingContext::prepare_live_photo`].
pub struct PrepareLivePhotoFuture {
    inner: AsyncCompletionFuture<PHLivePhotoResult>,
}

impl core::fmt::Debug for PrepareLivePhotoFuture {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PrepareLivePhotoFuture")
            .finish_non_exhaustive()
    }
}

impl Future for PrepareLivePhotoFuture {
    type Output = Result<PHLivePhotoResult, PhotoKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|result| result.map_err(PhotoKitError::OperationFailed))
    }
}

/// Async wrapper for `PHPhotoLibrary` authorization and `performChanges` operations.
#[derive(Debug, Clone, Copy)]
pub struct AsyncPHPhotoLibrary;

impl AsyncPHPhotoLibrary {
    /// Asynchronously request photo library authorization for the given access level.
    pub fn request_authorization(access_level: PHAccessLevel) -> RequestAuthorizationFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            crate::ffi::ph_photo_library_request_authorization_async(
                access_level.as_raw(),
                auth_status_callback,
                ctx,
            );
        }
        RequestAuthorizationFuture { inner: future }
    }

    /// Asynchronously perform asset changes via `PHPhotoLibrary.performChanges`.
    pub fn perform_asset_change(
        request: &PHAssetChangeRequest,
    ) -> Result<PerformChangesFuture, PhotoKitError> {
        let payload_json = json_cstring(request, "PHAssetChangeRequest")?;
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            crate::ffi::ph_asset_change_request_perform_async(
                payload_json.as_ptr(),
                change_result_callback,
                ctx,
            );
        }
        Ok(PerformChangesFuture { inner: future })
    }

    /// Asynchronously perform asset-collection changes via `PHPhotoLibrary.performChanges`.
    pub fn perform_collection_change(
        request: &PHAssetCollectionChangeRequest,
    ) -> Result<PerformChangesFuture, PhotoKitError> {
        let payload_json = json_cstring(request, "PHAssetCollectionChangeRequest")?;
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            crate::ffi::ph_asset_collection_change_request_perform_async(
                payload_json.as_ptr(),
                change_result_callback,
                ctx,
            );
        }
        Ok(PerformChangesFuture { inner: future })
    }

    /// Asynchronously perform collection-list changes via `PHPhotoLibrary.performChanges`.
    pub fn perform_collection_list_change(
        request: &PHCollectionListChangeRequest,
    ) -> Result<PerformChangesFuture, PhotoKitError> {
        let payload_json = json_cstring(request, "PHCollectionListChangeRequest")?;
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            crate::ffi::ph_collection_list_change_request_perform_async(
                payload_json.as_ptr(),
                change_result_callback,
                ctx,
            );
        }
        Ok(PerformChangesFuture { inner: future })
    }
}

/// Async wrapper for [`PHAssetChangeRequest`].
#[derive(Debug, Clone, Copy)]
pub struct AsyncPHAssetChangeRequest;

impl AsyncPHAssetChangeRequest {
    /// Asynchronously perform an asset change request.
    pub fn perform(request: &PHAssetChangeRequest) -> Result<PerformChangesFuture, PhotoKitError> {
        AsyncPHPhotoLibrary::perform_asset_change(request)
    }
}

/// Async wrapper for [`PHAssetCollectionChangeRequest`].
#[derive(Debug, Clone, Copy)]
pub struct AsyncPHAssetCollectionChangeRequest;

impl AsyncPHAssetCollectionChangeRequest {
    /// Asynchronously perform an asset-collection change request.
    pub fn perform(
        request: &PHAssetCollectionChangeRequest,
    ) -> Result<PerformChangesFuture, PhotoKitError> {
        AsyncPHPhotoLibrary::perform_collection_change(request)
    }
}

/// Async wrapper for [`PHCollectionListChangeRequest`].
#[derive(Debug, Clone, Copy)]
pub struct AsyncPHCollectionListChangeRequest;

impl AsyncPHCollectionListChangeRequest {
    /// Asynchronously perform a collection-list change request.
    pub fn perform(
        request: &PHCollectionListChangeRequest,
    ) -> Result<PerformChangesFuture, PhotoKitError> {
        AsyncPHPhotoLibrary::perform_collection_list_change(request)
    }
}

/// Async wrapper for `PHImageManager` image + image-data requests.
#[derive(Debug)]
pub struct AsyncPHImageManager {
    raw: NonNull<c_void>,
}

impl AsyncPHImageManager {
    /// Create an async wrapper around the shared `PHImageManager`.
    pub fn shared() -> Result<Self, PhotoKitError> {
        let raw =
            NonNull::new(unsafe { crate::ffi::ph_image_manager_default() }).ok_or_else(|| {
                PhotoKitError::OperationFailed("failed to get PHImageManager".to_owned())
            })?;
        Ok(Self { raw })
    }

    /// Asynchronously request an image for `asset`.
    pub fn request_image(
        &self,
        asset: &PHAsset,
        request: PHImageRequest,
    ) -> Result<RequestImageFuture, PhotoKitError> {
        let asset_id = cstring_from_str(&asset.local_identifier, "asset local identifier")?;
        let request_json = json_cstring(&request, "PHImageRequest")?;
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            crate::ffi::ph_image_manager_request_image_async(
                self.raw.as_ptr(),
                asset_id.as_ptr(),
                request_json.as_ptr(),
                image_result_callback,
                ctx,
            );
        }
        Ok(RequestImageFuture { inner: future })
    }

    /// Asynchronously request raw image data for `asset`.
    pub fn request_image_data(
        &self,
        asset: &PHAsset,
        request: &PHImageRequest,
    ) -> Result<RequestImageDataFuture, PhotoKitError> {
        let asset_id = cstring_from_str(&asset.local_identifier, "asset local identifier")?;
        let request_json = json_cstring(request, "PHImageRequest")?;
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            crate::ffi::ph_image_manager_request_image_data_async(
                self.raw.as_ptr(),
                asset_id.as_ptr(),
                request_json.as_ptr(),
                image_data_result_callback,
                ctx,
            );
        }
        Ok(RequestImageDataFuture { inner: future })
    }
}

impl Drop for AsyncPHImageManager {
    fn drop(&mut self) {
        unsafe { crate::ffi::ph_image_manager_release(self.raw.as_ptr()) };
    }
}

/// Async wrapper for `PHLivePhotoEditingContext` completion-handler APIs.
#[derive(Debug, Clone, Copy)]
pub struct AsyncPHLivePhotoEditingContext;

impl AsyncPHLivePhotoEditingContext {
    /// Asynchronously save a live photo to `output`.
    pub fn save_live_photo(
        context: &PHLivePhotoEditingContext,
        output: &PHContentEditingOutput,
    ) -> SaveLivePhotoFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            crate::ffi::ph_live_photo_editing_context_save_async(
                context.as_raw(),
                output.as_raw(),
                live_photo_save_callback,
                ctx,
            );
        }
        SaveLivePhotoFuture { inner: future }
    }

    /// Asynchronously prepare a live photo for playback at the given target size.
    pub fn prepare_live_photo(
        context: &PHLivePhotoEditingContext,
        target_width: f64,
        target_height: f64,
    ) -> PrepareLivePhotoFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            crate::ffi::ph_live_photo_editing_context_prepare_async(
                context.as_raw(),
                target_width,
                target_height,
                live_photo_prepare_callback,
                ctx,
            );
        }
        PrepareLivePhotoFuture { inner: future }
    }
}
