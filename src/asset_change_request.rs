use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::asset::{PHAsset, PHCoordinate};
use crate::change_request::PHChangeRequest;
use crate::error::PhotoKitError;
use crate::ffi;
use crate::object::PHObjectPlaceholder;
use crate::private::json_cstring;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PHChangeRequestPerformResult {
    pub placeholder_local_identifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHAssetChangeRequest`.
pub struct PHAssetChangeRequest {
    /// Serialized field carried by `PHAssetChangeRequest`.
    pub asset_local_identifier: Option<String>,
    /// Serialized field carried by `PHAssetChangeRequest`.
    pub create_image_file_url: Option<String>,
    /// Serialized field carried by `PHAssetChangeRequest`.
    pub create_image_data_base64: Option<String>,
    /// Serialized field carried by `PHAssetChangeRequest`.
    pub create_video_file_url: Option<String>,
    /// Serialized field carried by `PHAssetChangeRequest`.
    pub set_creation_date: Option<String>,
    #[serde(default)]
    /// Serialized field carried by `PHAssetChangeRequest`.
    pub clear_creation_date: bool,
    /// Serialized field carried by `PHAssetChangeRequest`.
    pub set_location: Option<PHCoordinate>,
    #[serde(default)]
    /// Serialized field carried by `PHAssetChangeRequest`.
    pub clear_location: bool,
    /// Serialized field carried by `PHAssetChangeRequest`.
    pub favorite: Option<bool>,
    /// Serialized field carried by `PHAssetChangeRequest`.
    pub hidden: Option<bool>,
    #[serde(default)]
    /// Serialized field carried by `PHAssetChangeRequest`.
    pub revert_asset_content_to_original: bool,
}

impl PHAssetChangeRequest {
    /// Wraps a Photos framework operation on `PHAssetChangeRequest`.
    pub fn change_request_for_asset(asset: &PHAsset) -> Self {
        Self {
            asset_local_identifier: Some(asset.local_identifier.clone()),
            ..Self::default()
        }
    }

    /// Wraps a Photos framework operation on `PHAssetChangeRequest`.
    pub fn creation_request_for_asset_from_image_file_url(file_url: impl Into<String>) -> Self {
        Self {
            create_image_file_url: Some(file_url.into()),
            ..Self::default()
        }
    }

    /// Wraps a Photos framework operation on `PHAssetChangeRequest`.
    pub fn creation_request_for_asset_from_image_data(data: &[u8]) -> Self {
        Self {
            create_image_data_base64: Some(base64::engine::general_purpose::STANDARD.encode(data)),
            ..Self::default()
        }
    }

    /// Wraps a Photos framework operation on `PHAssetChangeRequest`.
    pub fn creation_request_for_asset_from_video_file_url(file_url: impl Into<String>) -> Self {
        Self {
            create_video_file_url: Some(file_url.into()),
            ..Self::default()
        }
    }

    /// Updates the wrapped Photos framework value on `PHAssetChangeRequest`.
    pub fn set_creation_date(mut self, creation_date: impl Into<String>) -> Self {
        self.set_creation_date = Some(creation_date.into());
        self.clear_creation_date = false;
        self
    }

    /// Clears Photos framework state on `PHAssetChangeRequest`.
    pub fn clear_creation_date(mut self) -> Self {
        self.set_creation_date = None;
        self.clear_creation_date = true;
        self
    }

    /// Updates the wrapped Photos framework value on `PHAssetChangeRequest`.
    pub fn set_location(mut self, location: PHCoordinate) -> Self {
        self.set_location = Some(location);
        self.clear_location = false;
        self
    }

    /// Clears Photos framework state on `PHAssetChangeRequest`.
    pub fn clear_location(mut self) -> Self {
        self.set_location = None;
        self.clear_location = true;
        self
    }

    /// Updates the wrapped Photos framework value on `PHAssetChangeRequest`.
    pub fn set_favorite(mut self, favorite: bool) -> Self {
        self.favorite = Some(favorite);
        self
    }

    /// Updates the wrapped Photos framework value on `PHAssetChangeRequest`.
    pub fn set_hidden(mut self, hidden: bool) -> Self {
        self.hidden = Some(hidden);
        self
    }

    /// Wraps a Photos framework operation on `PHAssetChangeRequest`.
    pub fn revert_asset_content_to_original(mut self) -> Self {
        self.revert_asset_content_to_original = true;
        self
    }

    /// Wraps a Photos framework operation on `PHAssetChangeRequest`.
    pub fn delete_assets(assets: &[PHAsset]) -> Result<(), PhotoKitError> {
        let identifiers: Vec<&str> = assets
            .iter()
            .map(|asset| asset.local_identifier.as_str())
            .collect();
        let identifiers_json = json_cstring(&identifiers, "asset identifiers")?;
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::ph_asset_change_request_delete_assets_json(identifiers_json.as_ptr(), &mut error)
        };
        if status == ffi::status::OK && error.is_null() {
            Ok(())
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "delete assets failed") })
        }
    }
}

impl PHChangeRequest for PHAssetChangeRequest {
    type Output = Option<PHObjectPlaceholder>;

    fn perform(self) -> Result<Self::Output, PhotoKitError> {
        let payload_json = json_cstring(&self, "PHAssetChangeRequest")?;
        let mut error = core::ptr::null_mut();
        let payload =
            unsafe { ffi::ph_asset_change_request_perform_json(payload_json.as_ptr(), &mut error) };
        if payload.is_null() {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "asset change request failed") })
        } else {
            let result: PHChangeRequestPerformResult =
                unsafe { crate::private::parse_json_ptr(payload, "PHAssetChangeRequest result") }?;
            Ok(result
                .placeholder_local_identifier
                .map(PHObjectPlaceholder::new))
        }
    }
}
