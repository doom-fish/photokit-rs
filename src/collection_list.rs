use std::ptr;

use serde::{Deserialize, Serialize};

use crate::asset_collection::PHCollectionEditOperation;
use crate::error::PhotoKitError;
use crate::fetch_options::PHFetchOptions;
use crate::fetch_result::PHFetchResult;
use crate::ffi;
use crate::private::{cstring_from_str, json_cstring, parse_json_ptr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHCollectionListType`.
pub enum PHCollectionListType {
    /// Case of `PHCollectionListType`.
    Folder,
    /// Case of `PHCollectionListType`.
    SmartFolder,
}

impl PHCollectionListType {
    pub(crate) const fn as_raw(self) -> i32 {
        match self {
            Self::Folder => 2,
            Self::SmartFolder => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(transparent)]
/// Wraps `PHCollectionListSubtype`.
pub struct PHCollectionListSubtype(
    /// Raw value for `PHCollectionListSubtype`.
    pub i64,
);

impl PHCollectionListSubtype {
    /// Constant on `PHCollectionListSubtype`.
    pub const REGULAR_FOLDER: Self = Self(100);
    /// Constant on `PHCollectionListSubtype`.
    pub const SMART_FOLDER_EVENTS: Self = Self(200);
    /// Constant on `PHCollectionListSubtype`.
    pub const SMART_FOLDER_FACES: Self = Self(201);
    /// Constant on `PHCollectionListSubtype`.
    pub const ANY: Self = Self(i64::MAX);

    /// Returns the raw Photos framework value for `PHCollectionListSubtype`.
    pub const fn raw_value(self) -> i64 {
        self.0
    }
}

impl From<i64> for PHCollectionListSubtype {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<PHCollectionListSubtype> for i64 {
    fn from(value: PHCollectionListSubtype) -> Self {
        value.0
    }
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHCollectionList`.
pub struct PHCollectionList {
    /// Corresponds to `PHCollectionList.localIdentifier`.
    pub local_identifier: String,
    /// Corresponds to `PHCollectionList.localizedTitle`.
    pub localized_title: Option<String>,
    /// Corresponds to `PHCollectionList.collectionListType`.
    pub collection_list_type: PHCollectionListType,
    /// Corresponds to `PHCollectionList.collectionListSubtype`.
    pub collection_list_subtype: PHCollectionListSubtype,
    /// Corresponds to `PHCollectionList.startDate`.
    pub start_date: Option<String>,
    /// Corresponds to `PHCollectionList.endDate`.
    pub end_date: Option<String>,
    #[serde(default)]
    /// Corresponds to `PHCollectionList.localizedLocationNames`.
    pub localized_location_names: Vec<String>,
    #[serde(default)]
    /// Corresponds to `PHCollectionList.canContainAssets`.
    pub can_contain_assets: bool,
    #[serde(default)]
    /// Corresponds to `PHCollectionList.canContainCollections`.
    pub can_contain_collections: bool,
}

impl PHCollectionList {
    /// Wraps a Photos framework fetch operation on `PHCollectionList`.
    pub fn fetch_containing_collection_local_identifier(
        collection_local_identifier: &str,
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<Self>, PhotoKitError> {
        let collection_identifier =
            cstring_from_str(collection_local_identifier, "collection local identifier")?;
        let options_json = json_cstring(fetch_options, "PHFetchOptions")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_collection_list_fetch_containing_collection_json(
                collection_identifier.as_ptr(),
                options_json.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(
                    error,
                    "fetch collection lists containing collection failed",
                )
            })
        } else {
            let lists: Vec<Self> = unsafe { parse_json_ptr(payload, "PHCollectionList list") }?;
            Ok(lists.into())
        }
    }

    /// Wraps a Photos framework fetch operation on `PHCollectionList`.
    pub fn fetch_with_local_identifiers(
        identifiers: &[String],
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<Self>, PhotoKitError> {
        let identifiers_json = json_cstring(identifiers, "collection list identifiers")?;
        let options_json = json_cstring(fetch_options, "PHFetchOptions")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_collection_list_fetch_with_local_identifiers_json(
                identifiers_json.as_ptr(),
                options_json.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(
                    error,
                    "fetch collection lists by local identifier failed",
                )
            })
        } else {
            let lists: Vec<Self> = unsafe { parse_json_ptr(payload, "PHCollectionList list") }?;
            Ok(lists.into())
        }
    }

    /// Wraps a Photos framework fetch operation on `PHCollectionList`.
    pub fn fetch_with_type(
        collection_list_type: PHCollectionListType,
        collection_list_subtype: impl Into<PHCollectionListSubtype>,
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<Self>, PhotoKitError> {
        let options_json = json_cstring(fetch_options, "PHFetchOptions")?;
        let collection_list_subtype = collection_list_subtype.into();
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_collection_list_fetch_with_type_json(
                collection_list_type.as_raw(),
                collection_list_subtype.raw_value(),
                options_json.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "fetch collection lists by type failed")
            })
        } else {
            let lists: Vec<Self> = unsafe { parse_json_ptr(payload, "PHCollectionList list") }?;
            Ok(lists.into())
        }
    }

    /// Looks up `PHCollectionList` from Photos framework identifiers.
    pub fn from_local_identifier(
        local_identifier: impl Into<String>,
    ) -> Result<Option<Self>, PhotoKitError> {
        let result = Self::fetch_with_local_identifiers(
            &[local_identifier.into()],
            &PHFetchOptions::default(),
        )?;
        Ok(result.into_vec().into_iter().next())
    }

    /// Queries Photos framework state exposed by `PHCollectionList`.
    pub fn can_perform_edit_operation(
        &self,
        edit_operation: PHCollectionEditOperation,
    ) -> Result<bool, PhotoKitError> {
        let collection_identifier =
            cstring_from_str(&self.local_identifier, "collection list local identifier")?;
        let mut error = ptr::null_mut();
        let allowed = unsafe {
            ffi::ph_collection_list_can_perform_edit_operation(
                collection_identifier.as_ptr(),
                edit_operation.as_raw(),
                &mut error,
            )
        };
        if error.is_null() {
            Ok(allowed != 0)
        } else {
            Err(unsafe {
                PhotoKitError::from_error_ptr(
                    error,
                    "collection list edit capability lookup failed",
                )
            })
        }
    }

    /// Wraps a Photos framework operation on `PHCollectionList`.
    pub fn containing_collection_lists(
        &self,
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<Self>, PhotoKitError> {
        Self::fetch_containing_collection_local_identifier(&self.local_identifier, fetch_options)
    }
}
