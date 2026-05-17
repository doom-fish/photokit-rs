use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::asset::PHAssetResource;
use crate::error::{NSErrorInfo, PhotoKitError};
use crate::ffi;
use crate::private::json_cstring;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PHAssetResourceRequestOptions {
    #[serde(default)]
    pub network_access_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHAssetResourceDataResult {
    pub request_id: i32,
    pub data_base64: String,
    #[serde(default)]
    pub error: Option<NSErrorInfo>,
}

impl PHAssetResourceDataResult {
    pub fn data(&self) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(self.data_base64.as_bytes())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHAssetResourceWriteResult {
    pub file_url: String,
    pub success: bool,
    #[serde(default)]
    pub error: Option<NSErrorInfo>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PHAssetResourceManager;

impl PHAssetResourceManager {
    pub fn request_data_for_asset_resource(
        &self,
        resource: &PHAssetResource,
        options: &PHAssetResourceRequestOptions,
        timeout_ms: u64,
    ) -> Result<PHAssetResourceDataResult, PhotoKitError> {
        let resource_json = json_cstring(resource, "PHAssetResource")?;
        let options_json = json_cstring(options, "PHAssetResourceRequestOptions")?;
        let mut error = core::ptr::null_mut();
        let payload = unsafe {
            ffi::ph_asset_resource_manager_request_data_json(
                resource_json.as_ptr(),
                options_json.as_ptr(),
                timeout_ms,
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "asset resource data request failed")
            })
        } else {
            unsafe { crate::private::parse_json_ptr(payload, "PHAssetResourceDataResult") }
        }
    }

    pub fn write_data_for_asset_resource(
        &self,
        resource: &PHAssetResource,
        file_url: &str,
        options: &PHAssetResourceRequestOptions,
        timeout_ms: u64,
    ) -> Result<PHAssetResourceWriteResult, PhotoKitError> {
        let resource_json = json_cstring(resource, "PHAssetResource")?;
        let options_json = json_cstring(options, "PHAssetResourceRequestOptions")?;
        let file_url = crate::private::cstring_from_str(file_url, "asset resource file url")?;
        let mut error = core::ptr::null_mut();
        let payload = unsafe {
            ffi::ph_asset_resource_manager_write_data_json(
                resource_json.as_ptr(),
                file_url.as_ptr(),
                options_json.as_ptr(),
                timeout_ms,
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "asset resource write failed") })
        } else {
            unsafe { crate::private::parse_json_ptr(payload, "PHAssetResourceWriteResult") }
        }
    }
}
