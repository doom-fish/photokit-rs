use core::ffi::c_void;
use std::ops::Deref;
use std::ptr::{self, NonNull};

use serde::{Deserialize, Serialize};

use crate::content_editing_input::{PHAdjustmentData, PHContentEditingInput};
use crate::error::PhotoKitError;
use crate::ffi;
use crate::private::{cstring_from_str, json_cstring, parse_json_ptr, take_string};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Serialized snapshot of `PHContentEditingOutput` properties.
pub struct PHContentEditingOutputInfo {
    /// Corresponds to `PHContentEditingOutputInfo.adjustmentData`.
    pub adjustment_data: Option<PHAdjustmentData>,
    /// Corresponds to `PHContentEditingOutputInfo.renderedContentUrl`.
    pub rendered_content_url: String,
    /// Corresponds to `PHContentEditingOutputInfo.defaultRenderedContentTypeIdentifier`.
    pub default_rendered_content_type_identifier: Option<String>,
    #[serde(default)]
    /// Corresponds to `PHContentEditingOutputInfo.supportedRenderedContentTypeIdentifiers`.
    pub supported_rendered_content_type_identifiers: Vec<String>,
}

/// Wraps `PHContentEditingOutput`.
pub struct PHContentEditingOutput {
    raw: NonNull<c_void>,
    info: PHContentEditingOutputInfo,
}

impl PHContentEditingOutput {
    pub(crate) unsafe fn from_raw(raw: *mut c_void) -> Result<Self, PhotoKitError> {
        let raw = NonNull::new(raw).ok_or_else(|| {
            PhotoKitError::OperationFailed(
                "failed to create PHContentEditingOutput handle".to_owned(),
            )
        })?;
        let mut error = ptr::null_mut();
        let payload = ffi::ph_content_editing_output_json(raw.as_ptr(), &mut error);
        if payload.is_null() {
            Err(PhotoKitError::from_error_ptr(
                error,
                "content editing output snapshot failed",
            ))
        } else {
            let info = parse_json_ptr(payload, "PHContentEditingOutput")?;
            Ok(Self { raw, info })
        }
    }

    /// Returns the cached Photos framework snapshot for `PHContentEditingOutput`.
    pub fn snapshot(&self) -> &PHContentEditingOutputInfo {
        &self.info
    }

    /// Updates the wrapped Photos framework value on `PHContentEditingOutput`.
    pub fn set_adjustment_data(
        &mut self,
        adjustment_data: Option<PHAdjustmentData>,
    ) -> Result<(), PhotoKitError> {
        let adjustment_json = match adjustment_data {
            Some(adjustment_data) => Some(json_cstring(&adjustment_data, "PHAdjustmentData")?),
            None => None,
        };
        let mut error = ptr::null_mut();
        let status = unsafe {
            ffi::ph_content_editing_output_set_adjustment_data_json(
                self.raw.as_ptr(),
                adjustment_json
                    .as_ref()
                    .map_or(ptr::null(), |value| value.as_c_str().as_ptr()),
                &mut error,
            )
        };
        if status == ffi::status::OK && error.is_null() {
            let mut refresh_error = ptr::null_mut();
            let payload = unsafe {
                ffi::ph_content_editing_output_json(self.raw.as_ptr(), &mut refresh_error)
            };
            if payload.is_null() {
                Err(unsafe {
                    PhotoKitError::from_error_ptr(
                        refresh_error,
                        "content editing output refresh failed",
                    )
                })
            } else {
                self.info = unsafe { parse_json_ptr(payload, "PHContentEditingOutput") }?;
                Ok(())
            }
        } else {
            Err(unsafe {
                PhotoKitError::from_error_ptr(
                    error,
                    "set content editing output adjustment data failed",
                )
            })
        }
    }

    pub(crate) fn as_raw(&self) -> *mut c_void {
        self.raw.as_ptr()
    }

    /// Wraps a Photos framework operation on `PHContentEditingOutput`.
    pub fn rendered_content_url_for_type_identifier(
        &self,
        type_identifier: &str,
    ) -> Result<String, PhotoKitError> {
        let type_identifier =
            cstring_from_str(type_identifier, "rendered content type identifier")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_content_editing_output_rendered_content_url_for_type(
                self.raw.as_ptr(),
                type_identifier.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "rendered content url lookup failed")
            })
        } else {
            unsafe { take_string(payload) }.ok_or_else(|| {
                PhotoKitError::OperationFailed("missing rendered content url payload".to_owned())
            })
        }
    }
}

impl Deref for PHContentEditingOutput {
    type Target = PHContentEditingOutputInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl core::fmt::Debug for PHContentEditingOutput {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PHContentEditingOutput")
            .field("info", &self.info)
            .finish_non_exhaustive()
    }
}

impl Drop for PHContentEditingOutput {
    fn drop(&mut self) {
        unsafe { ffi::ph_content_editing_output_release(self.raw.as_ptr()) };
    }
}

impl PHContentEditingInput {
    /// Wraps a Photos framework operation on `PHContentEditingInput`.
    pub fn create_content_editing_output(&self) -> Result<PHContentEditingOutput, PhotoKitError> {
        let mut error = ptr::null_mut();
        let raw =
            unsafe { ffi::ph_content_editing_output_new_for_input(self.raw.as_ptr(), &mut error) };
        if raw.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "create content editing output failed")
            })
        } else {
            unsafe { PHContentEditingOutput::from_raw(raw) }
        }
    }
}
