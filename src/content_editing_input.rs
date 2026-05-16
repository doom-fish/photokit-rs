use core::ffi::c_void;
use std::ops::Deref;
use std::ptr::{self, NonNull};

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::asset::{PHAsset, PHAssetPlaybackStyle, PHCoordinate, PHMediaType};
use crate::error::PhotoKitError;
use crate::ffi;
use crate::private::{json_cstring, parse_json_ptr};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHAdjustmentData {
    pub format_identifier: String,
    pub format_version: String,
    pub data_base64: String,
}

impl PHAdjustmentData {
    pub fn data(&self) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(self.data_base64.as_bytes())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHContentEditingInputRequestOptions {
    #[serde(default)]
    pub network_access_allowed: bool,
    #[serde(default = "default_true")]
    pub accepts_any_adjustment_data: bool,
}

const fn default_true() -> bool {
    true
}

impl Default for PHContentEditingInputRequestOptions {
    fn default() -> Self {
        Self {
            network_access_allowed: false,
            accepts_any_adjustment_data: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHContentEditingInputInfo {
    pub media_type: PHMediaType,
    pub media_subtypes: u64,
    pub creation_date: Option<String>,
    pub location: Option<PHCoordinate>,
    pub content_type_identifier: Option<String>,
    pub uniform_type_identifier: Option<String>,
    pub playback_style: Option<PHAssetPlaybackStyle>,
    pub adjustment_data: Option<PHAdjustmentData>,
    #[serde(default)]
    pub has_display_size_image: bool,
    pub display_size_image_width: Option<f64>,
    pub display_size_image_height: Option<f64>,
    pub full_size_image_url: Option<String>,
    pub full_size_image_orientation: i32,
    pub audiovisual_asset_class: Option<String>,
    #[serde(default)]
    pub has_live_photo: bool,
    pub live_photo_size_width: Option<f64>,
    pub live_photo_size_height: Option<f64>,
}

pub struct PHContentEditingInput {
    pub(crate) raw: NonNull<c_void>,
    info: PHContentEditingInputInfo,
}

impl PHContentEditingInput {
    pub(crate) unsafe fn from_raw(raw: *mut c_void) -> Result<Self, PhotoKitError> {
        let raw = NonNull::new(raw).ok_or_else(|| {
            PhotoKitError::OperationFailed("failed to create PHContentEditingInput handle".to_owned())
        })?;
        let mut error = ptr::null_mut();
        let payload = ffi::ph_content_editing_input_json(raw.as_ptr(), &mut error);
        if payload.is_null() {
            Err(PhotoKitError::from_error_ptr(
                error,
                "content editing input snapshot failed",
            ))
        } else {
            let info = parse_json_ptr(payload, "PHContentEditingInput")?;
            Ok(Self { raw, info })
        }
    }

    pub fn snapshot(&self) -> &PHContentEditingInputInfo {
        &self.info
    }
}

impl Deref for PHContentEditingInput {
    type Target = PHContentEditingInputInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl core::fmt::Debug for PHContentEditingInput {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PHContentEditingInput")
            .field("info", &self.info)
            .finish_non_exhaustive()
    }
}

impl Drop for PHContentEditingInput {
    fn drop(&mut self) {
        unsafe { ffi::ph_content_editing_input_release(self.raw.as_ptr()) };
    }
}

impl PHAsset {
    pub fn request_content_editing_input(
        &self,
        options: &PHContentEditingInputRequestOptions,
        timeout_ms: u64,
    ) -> Result<PHContentEditingInput, PhotoKitError> {
        let asset_identifier =
            crate::private::cstring_from_str(&self.local_identifier, "asset local identifier")?;
        let options_json = json_cstring(options, "PHContentEditingInputRequestOptions")?;
        let mut error = ptr::null_mut();
        let raw = unsafe {
            ffi::ph_asset_request_content_editing_input(
                asset_identifier.as_ptr(),
                options_json.as_ptr(),
                timeout_ms,
                &mut error,
            )
        };
        if raw.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "request content editing input failed")
            })
        } else {
            unsafe { PHContentEditingInput::from_raw(raw) }
        }
    }
}
