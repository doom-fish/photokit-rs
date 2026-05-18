use std::collections::BTreeMap;
use std::ptr;

use serde::{Deserialize, Serialize};

use crate::error::{NSErrorInfo, PhotoKitError};
use crate::ffi;
use crate::photo_library::PHPhotoLibrary;
use crate::private::{json_cstring, parse_json_ptr};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHCloudIdentifier`.
pub struct PHCloudIdentifier {
    /// Corresponds to `PHCloudIdentifier.stringValue`.
    pub string_value: String,
}

impl PHCloudIdentifier {
    /// Creates a helper value for the related Photos framework API.
    pub fn new(string_value: impl Into<String>) -> Self {
        Self {
            string_value: string_value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Identifier mapping returned by Photos framework cloud identifier lookups.
pub struct PHCloudIdentifierMapping {
    /// Corresponds to `PHCloudIdentifierMapping.cloudIdentifier`.
    pub cloud_identifier: Option<PHCloudIdentifier>,
    /// Corresponds to `PHCloudIdentifierMapping.error`.
    pub error: Option<NSErrorInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Identifier mapping returned by Photos framework local identifier lookups.
pub struct PHLocalIdentifierMapping {
    /// Corresponds to `PHLocalIdentifierMapping.localIdentifier`.
    pub local_identifier: Option<String>,
    /// Corresponds to `PHLocalIdentifierMapping.error`.
    pub error: Option<NSErrorInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PHCloudIdentifierMappingEntry {
    local_identifier: String,
    mapping: PHCloudIdentifierMapping,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PHLocalIdentifierMappingEntry {
    cloud_identifier: PHCloudIdentifier,
    mapping: PHLocalIdentifierMapping,
}

impl PHPhotoLibrary {
    /// Wraps a Photos framework operation on `PHPhotoLibrary`.
    pub fn cloud_identifier_mappings_for_local_identifiers(
        &self,
        local_identifiers: &[String],
    ) -> Result<BTreeMap<String, PHCloudIdentifierMapping>, PhotoKitError> {
        let identifiers_json = json_cstring(local_identifiers, "local identifiers")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_photo_library_cloud_identifier_mappings_json(
                self.raw.as_ptr(),
                identifiers_json.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "cloud identifier mapping lookup failed")
            })
        } else {
            let entries: Vec<PHCloudIdentifierMappingEntry> =
                unsafe { parse_json_ptr(payload, "PHCloudIdentifierMapping list") }?;
            Ok(entries
                .into_iter()
                .map(|entry| (entry.local_identifier, entry.mapping))
                .collect())
        }
    }

    /// Wraps a Photos framework operation on `PHPhotoLibrary`.
    pub fn local_identifier_mappings_for_cloud_identifiers(
        &self,
        cloud_identifiers: &[PHCloudIdentifier],
    ) -> Result<BTreeMap<PHCloudIdentifier, PHLocalIdentifierMapping>, PhotoKitError> {
        let identifiers_json = json_cstring(cloud_identifiers, "cloud identifiers")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_photo_library_local_identifier_mappings_json(
                self.raw.as_ptr(),
                identifiers_json.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "local identifier mapping lookup failed")
            })
        } else {
            let entries: Vec<PHLocalIdentifierMappingEntry> =
                unsafe { parse_json_ptr(payload, "PHLocalIdentifierMapping list") }?;
            Ok(entries
                .into_iter()
                .map(|entry| (entry.cloud_identifier, entry.mapping))
                .collect())
        }
    }
}
