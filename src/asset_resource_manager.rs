use core::ffi::c_void;
use std::sync::{Mutex, PoisonError};

use doom_fish_utils::callback_context::CallbackContext;
use serde::{Deserialize, Serialize};

use crate::asset::PHAssetResource;
use crate::error::{NSErrorInfo, PhotoKitError};
use crate::ffi;
use crate::private::json_cstring;

type ResourceDataSink = CallbackContext<Mutex<Vec<u8>>>;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResourceDataRequestOutcome {
    #[serde(alias = "requestID")]
    request_id: i32,
    #[serde(default)]
    error: Option<NSErrorInfo>,
}

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
    #[serde(alias = "requestID")]
    /// Corresponds to `PHAssetResourceDataResult.requestId`.
    pub request_id: i32,
    #[serde(default)]
    /// Corresponds to the bytes delivered by `PHAssetResourceManager.requestData`.
    pub data: Vec<u8>,
    #[serde(default)]
    /// Corresponds to `PHAssetResourceDataResult.error`.
    pub error: Option<NSErrorInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Serialized result from `PHAssetResourceManager.writeData`.
pub struct PHAssetResourceWriteResult {
    #[serde(rename = "fileURL", alias = "fileUrl")]
    /// Corresponds to `PHAssetResourceWriteResult.fileURL`.
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
        let sink = ResourceDataSink::new(Mutex::new(Vec::new()));
        let mut error = core::ptr::null_mut();
        let payload = unsafe {
            ffi::ph_asset_resource_manager_request_data(
                resource_json.as_ptr(),
                options_json.as_ptr(),
                timeout_ms,
                append_resource_data,
                sink.as_ptr(),
                ResourceDataSink::RETAIN,
                ResourceDataSink::RELEASE,
                &mut error,
            )
        };
        if payload.is_null() {
            return Err(unsafe {
                PhotoKitError::from_error_ptr(error, "asset resource data request failed")
            });
        }
        let outcome: ResourceDataRequestOutcome =
            unsafe { crate::private::parse_json_ptr(payload, "PHAssetResourceDataResult") }?;
        let data = std::mem::take(&mut *sink.get().lock().unwrap_or_else(PoisonError::into_inner));
        Ok(PHAssetResourceDataResult {
            request_id: outcome.request_id,
            data,
            error: outcome.error,
        })
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

unsafe extern "C" fn append_resource_data(bytes: *const u8, len: usize, context: *mut c_void) {
    if bytes.is_null() || len == 0 {
        return;
    }
    let _ = ResourceDataSink::with(context, "append_resource_data", |sink| {
        let chunk = std::slice::from_raw_parts(bytes, len);
        sink.lock()
            .unwrap_or_else(PoisonError::into_inner)
            .extend_from_slice(chunk);
    });
}

#[cfg(test)]
mod tests {
    use std::ptr;
    use std::sync::Mutex;

    use super::{append_resource_data, ResourceDataSink};

    #[test]
    fn data_sink_collects_chunks_until_the_request_owner_drops_it() {
        let sink = ResourceDataSink::new(Mutex::new(Vec::new()));
        let swift_reference = sink.retained_ptr();

        unsafe {
            append_resource_data(b"hello ".as_ptr(), 6, swift_reference);
            append_resource_data(b"world".as_ptr(), 5, swift_reference);
            append_resource_data(ptr::null(), 3, swift_reference);
            append_resource_data(b"ignored".as_ptr(), 0, swift_reference);
        }
        assert_eq!(sink.get().lock().unwrap().as_slice(), b"hello world");

        sink.deactivate();
        unsafe { append_resource_data(b"late chunk after timeout".as_ptr(), 24, swift_reference) };
        assert_eq!(sink.get().lock().unwrap().as_slice(), b"hello world");

        drop(sink);
        unsafe { (ResourceDataSink::RELEASE)(swift_reference) };
    }

    #[test]
    fn data_sink_ignores_a_null_context() {
        unsafe { append_resource_data(b"chunk".as_ptr(), 5, ptr::null_mut()) };
    }
}
