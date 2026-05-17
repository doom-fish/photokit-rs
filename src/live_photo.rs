use std::ptr::{self, NonNull};

use serde::{Deserialize, Serialize};

use crate::error::{NSErrorInfo, PhotoKitError};
use crate::ffi;
use crate::image_manager::{PHImageRequest, PHLivePhotoRequestHandle};
use crate::private::json_cstring;

#[allow(non_upper_case_globals)]
pub const PHLivePhotoInfoErrorKey: &str = "PHLivePhotoInfoErrorKey";

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PHLivePhoto {
    pub available: bool,
    pub size_width: f64,
    pub size_height: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHLivePhotoResult {
    pub has_live_photo: bool,
    pub cancelled: bool,
    pub degraded: bool,
    #[serde(default)]
    pub size_width: f64,
    #[serde(default)]
    pub size_height: f64,
    #[serde(default)]
    pub request_id: Option<i32>,
    #[serde(default)]
    pub error: Option<NSErrorInfo>,
}

impl PHLivePhotoResult {
    pub fn live_photo(&self) -> Option<PHLivePhoto> {
        self.has_live_photo.then_some(PHLivePhoto {
            available: true,
            size_width: self.size_width,
            size_height: self.size_height,
        })
    }
}

impl PHLivePhoto {
    pub fn request_with_resource_file_urls(
        file_urls: &[String],
        request: &PHImageRequest,
    ) -> Result<PHLivePhotoRequestHandle, PhotoKitError> {
        let file_urls_json = json_cstring(file_urls, "live photo resource urls")?;
        let request_json = json_cstring(request, "PHImageRequest")?;
        let mut error = ptr::null_mut();
        let raw = unsafe {
            ffi::ph_live_photo_request_with_resource_file_urls(
                file_urls_json.as_ptr(),
                request_json.as_ptr(),
                &mut error,
            )
        };
        NonNull::new(raw)
            .map(|raw| PHLivePhotoRequestHandle { raw })
            .ok_or_else(|| unsafe {
                PhotoKitError::from_error_ptr(error, "request live photo with resource urls failed")
            })
    }
}
