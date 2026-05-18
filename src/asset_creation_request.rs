use std::ptr;

use base64::Engine;
use serde::Serialize;

use crate::asset::PHAssetResourceType;
use crate::error::PhotoKitError;
use crate::ffi;
use crate::object::PHObjectPlaceholder;
use crate::private::{json_cstring, take_string};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHAssetResourceCreationOptions`.
pub struct PHAssetResourceCreationOptions {
    /// Corresponds to `PHAssetResourceCreationOptions.originalFilename`.
    pub original_filename: Option<String>,
    /// Corresponds to `PHAssetResourceCreationOptions.contentTypeIdentifier`.
    pub content_type_identifier: Option<String>,
    /// Corresponds to `PHAssetResourceCreationOptions.uniformTypeIdentifier`.
    pub uniform_type_identifier: Option<String>,
    #[serde(default)]
    /// Corresponds to `PHAssetResourceCreationOptions.shouldMoveFile`.
    pub should_move_file: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// Helper input for `PHAssetCreationRequest` resource additions.
pub struct PHAssetCreationResource {
    /// Corresponds to `PHAssetCreationResource.resourceType`.
    pub resource_type: PHAssetResourceType,
    /// Corresponds to `PHAssetCreationResource.fileUrl`.
    pub file_url: Option<String>,
    /// Corresponds to `PHAssetCreationResource.dataBase64`.
    pub data_base64: Option<String>,
    /// Corresponds to `PHAssetCreationResource.options`.
    pub options: Option<PHAssetResourceCreationOptions>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
/// Wraps `PHAssetCreationRequest`.
pub struct PHAssetCreationRequest {
    /// Serialized field carried by `PHAssetCreationRequest`.
    pub resources: Vec<PHAssetCreationResource>,
}

impl PHAssetCreationRequest {
    /// Creates a helper value for the related Photos framework API.
    pub fn new() -> Self {
        Self::default()
    }

    /// Wraps a Photos framework operation on `PHAssetCreationRequest`.
    pub fn add_file_resource(
        mut self,
        resource_type: impl Into<PHAssetResourceType>,
        file_url: impl Into<String>,
        options: Option<PHAssetResourceCreationOptions>,
    ) -> Self {
        self.resources.push(PHAssetCreationResource {
            resource_type: resource_type.into(),
            file_url: Some(file_url.into()),
            data_base64: None,
            options,
        });
        self
    }

    /// Wraps a Photos framework operation on `PHAssetCreationRequest`.
    pub fn add_data_resource(
        mut self,
        resource_type: impl Into<PHAssetResourceType>,
        data_base64: impl Into<String>,
        options: Option<PHAssetResourceCreationOptions>,
    ) -> Self {
        self.resources.push(PHAssetCreationResource {
            resource_type: resource_type.into(),
            file_url: None,
            data_base64: Some(data_base64.into()),
            options,
        });
        self
    }

    /// Wraps a Photos framework operation on `PHAssetCreationRequest`.
    pub fn add_data_resource_bytes(
        self,
        resource_type: impl Into<PHAssetResourceType>,
        data: &[u8],
        options: Option<PHAssetResourceCreationOptions>,
    ) -> Self {
        self.add_data_resource(
            resource_type,
            base64::engine::general_purpose::STANDARD.encode(data),
            options,
        )
    }

    /// Queries Photos framework state exposed by `PHAssetCreationRequest`.
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// Wraps a Photos framework operation on `PHAssetCreationRequest`.
    pub fn supports_asset_resource_types(
        resource_types: &[PHAssetResourceType],
    ) -> Result<bool, PhotoKitError> {
        let resource_types_json = json_cstring(resource_types, "resource types")?;
        let mut error = ptr::null_mut();
        let supported = unsafe {
            ffi::ph_asset_creation_request_supports_resource_types(
                resource_types_json.as_ptr(),
                &mut error,
            )
        };
        if error.is_null() {
            Ok(supported != 0)
        } else {
            Err(unsafe {
                PhotoKitError::from_error_ptr(
                    error,
                    "asset creation resource type support lookup failed",
                )
            })
        }
    }

    /// Executes the Photos framework operation represented by `PHAssetCreationRequest`.
    pub fn perform(self) -> Result<String, PhotoKitError> {
        let resources_json = json_cstring(&self.resources, "asset creation resources")?;
        let mut error = ptr::null_mut();
        let payload =
            unsafe { ffi::ph_asset_creation_request_perform(resources_json.as_ptr(), &mut error) };
        if payload.is_null() {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "asset creation failed") })
        } else {
            unsafe { take_string(payload) }.ok_or_else(|| {
                PhotoKitError::OperationFailed("missing asset creation payload".to_owned())
            })
        }
    }

    /// Wraps a Photos framework operation on `PHAssetCreationRequest`.
    pub fn perform_placeholder(self) -> Result<PHObjectPlaceholder, PhotoKitError> {
        self.perform().map(PHObjectPlaceholder::new)
    }
}
