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
pub enum PHCollectionListType {
    Folder,
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

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHCollectionList {
    pub local_identifier: String,
    pub localized_title: Option<String>,
    pub collection_list_type: PHCollectionListType,
    pub collection_list_subtype: i64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    #[serde(default)]
    pub localized_location_names: Vec<String>,
    #[serde(default)]
    pub can_contain_assets: bool,
    #[serde(default)]
    pub can_contain_collections: bool,
}

impl PHCollectionList {
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

    pub fn fetch_with_type(
        collection_list_type: PHCollectionListType,
        collection_list_subtype: i64,
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<Self>, PhotoKitError> {
        let options_json = json_cstring(fetch_options, "PHFetchOptions")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_collection_list_fetch_with_type_json(
                collection_list_type.as_raw(),
                collection_list_subtype,
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

    pub fn from_local_identifier(
        local_identifier: impl Into<String>,
    ) -> Result<Option<Self>, PhotoKitError> {
        let result = Self::fetch_with_local_identifiers(
            &[local_identifier.into()],
            &PHFetchOptions::default(),
        )?;
        Ok(result.into_vec().into_iter().next())
    }

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
                PhotoKitError::from_error_ptr(error, "collection list edit capability lookup failed")
            })
        }
    }

    pub fn containing_collection_lists(
        &self,
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<Self>, PhotoKitError> {
        Self::fetch_containing_collection_local_identifier(&self.local_identifier, fetch_options)
    }
}
