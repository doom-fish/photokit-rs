use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::asset::PHAssetResource;
use crate::error::{NSErrorInfo, PhotoKitError};
use crate::ffi;
use crate::private::json_cstring;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHAssetResourceRequestOptions`.
pub struct PHAssetResourceRequestOptions {
    #[serde(default)]
    /// Corresponds to `PHAssetResourceRequestOptions.networkAccessAllowed`.
    pub network_access_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Serialized result from `PHAssetResourceManager.requestData`.
pub struct PHAssetResourceDataResult {
    /// Corresponds to `PHAssetResourceDataResult.requestId`.
    pub request_id: i32,
    /// Corresponds to `PHAssetResourceDataResult.dataBase64`.
    pub data_base64: String,
    #[serde(default)]
    /// Corresponds to `PHAssetResourceDataResult.error`.
    pub error: Option<NSErrorInfo>,
}

impl PHAssetResourceDataResult {
    /// Decodes the binary data carried by `PHAssetResourceDataResult`.
    pub fn data(&self) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(self.data_base64.as_bytes())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Serialized result from `PHAssetResourceManager.writeData`.
pub struct PHAssetResourceWriteResult {
    /// Corresponds to `PHAssetResourceWriteResult.fileUrl`.
    pub file_url: String,
    /// Corresponds to `PHAssetResourceWriteResult.success`.
    pub success: bool,
    #[serde(default)]
    /// Corresponds to `PHAssetResourceWriteResult.error`.
    pub error: Option<NSErrorInfo>,
}

#[derive(Debug, Clone, Copy, Default)]
/// Wraps `PHAssetResourceManager`.
pub struct PHAssetResourceManager;

impl PHAssetResourceManager {
    /// Wraps a Photos framework request operation on `PHAssetResourceManager`.
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

    /// Wraps a Photos framework operation on `PHAssetResourceManager`.
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
