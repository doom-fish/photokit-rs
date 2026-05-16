use std::ptr;

use base64::Engine;
use serde::Serialize;

use crate::error::PhotoKitError;
use crate::ffi;
use crate::private::{json_cstring, take_string};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PHAssetResourceCreationOptions {
    pub original_filename: Option<String>,
    pub content_type_identifier: Option<String>,
    pub uniform_type_identifier: Option<String>,
    #[serde(default)]
    pub should_move_file: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PHAssetCreationResource {
    pub resource_type: i64,
    pub file_url: Option<String>,
    pub data_base64: Option<String>,
    pub options: Option<PHAssetResourceCreationOptions>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct PHAssetCreationRequest {
    pub resources: Vec<PHAssetCreationResource>,
}

impl PHAssetCreationRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file_resource(
        mut self,
        resource_type: i64,
        file_url: impl Into<String>,
        options: Option<PHAssetResourceCreationOptions>,
    ) -> Self {
        self.resources.push(PHAssetCreationResource {
            resource_type,
            file_url: Some(file_url.into()),
            data_base64: None,
            options,
        });
        self
    }

    pub fn add_data_resource(
        mut self,
        resource_type: i64,
        data_base64: impl Into<String>,
        options: Option<PHAssetResourceCreationOptions>,
    ) -> Self {
        self.resources.push(PHAssetCreationResource {
            resource_type,
            file_url: None,
            data_base64: Some(data_base64.into()),
            options,
        });
        self
    }

    pub fn add_data_resource_bytes(
        self,
        resource_type: i64,
        data: &[u8],
        options: Option<PHAssetResourceCreationOptions>,
    ) -> Self {
        self.add_data_resource(
            resource_type,
            base64::engine::general_purpose::STANDARD.encode(data),
            options,
        )
    }

    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    pub fn supports_asset_resource_types(resource_types: &[i64]) -> Result<bool, PhotoKitError> {
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

    pub fn perform(self) -> Result<String, PhotoKitError> {
        let resources_json = json_cstring(&self.resources, "asset creation resources")?;
        let mut error = ptr::null_mut();
        let payload = unsafe { ffi::ph_asset_creation_request_perform(resources_json.as_ptr(), &mut error) };
        if payload.is_null() {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "asset creation failed") })
        } else {
            unsafe { take_string(payload) }.ok_or_else(|| {
                PhotoKitError::OperationFailed("missing asset creation payload".to_owned())
            })
        }
    }
}
