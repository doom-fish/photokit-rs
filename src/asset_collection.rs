use std::ptr;

use serde::{Deserialize, Serialize};

use crate::asset::{PHAsset, PHCoordinate};
use crate::collection_list::PHCollectionList;
use crate::error::PhotoKitError;
use crate::fetch_options::PHFetchOptions;
use crate::fetch_result::PHFetchResult;
use crate::ffi;
use crate::private::{cstring_from_str, json_cstring, parse_json_ptr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PHAssetCollectionType {
    Album,
    SmartAlbum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PHCollectionEditOperation {
    DeleteContent,
    RemoveContent,
    AddContent,
    CreateContent,
    RearrangeContent,
    Delete,
    Rename,
}

impl PHCollectionEditOperation {
    pub(crate) const fn as_raw(self) -> i32 {
        match self {
            Self::DeleteContent => 1,
            Self::RemoveContent => 2,
            Self::AddContent => 3,
            Self::CreateContent => 4,
            Self::RearrangeContent => 5,
            Self::Delete => 6,
            Self::Rename => 7,
        }
    }
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHAssetCollection {
    pub local_identifier: String,
    pub localized_title: Option<String>,
    pub collection_type: PHAssetCollectionType,
    pub collection_subtype: i64,
    pub estimated_asset_count: Option<u64>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub approximate_location: Option<PHCoordinate>,
    #[serde(default)]
    pub localized_location_names: Vec<String>,
    #[serde(default)]
    pub can_contain_assets: bool,
    #[serde(default)]
    pub can_contain_collections: bool,
}

impl PHAssetCollection {
    pub fn fetch(fetch_options: &PHFetchOptions) -> Result<PHFetchResult<Self>, PhotoKitError> {
        let options_json = json_cstring(fetch_options, "PHFetchOptions")?;
        let mut error = ptr::null_mut();
        let payload =
            unsafe { ffi::ph_asset_collection_fetch_all_json(options_json.as_ptr(), &mut error) };
        if payload.is_null() {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "fetch asset collections failed") })
        } else {
            let collections: Vec<Self> = unsafe { parse_json_ptr(payload, "PHAssetCollection list") }?;
            Ok(collections.into())
        }
    }

    pub fn fetch_with_type(
        collection_type: PHAssetCollectionType,
        collection_subtype: i64,
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<Self>, PhotoKitError> {
        let options_json = json_cstring(fetch_options, "PHFetchOptions")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_asset_collection_fetch_with_type_json(
                match collection_type {
                    PHAssetCollectionType::Album => 1,
                    PHAssetCollectionType::SmartAlbum => 2,
                },
                collection_subtype,
                options_json.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "fetch asset collections by type failed")
            })
        } else {
            let collections: Vec<Self> = unsafe { parse_json_ptr(payload, "PHAssetCollection list") }?;
            Ok(collections.into())
        }
    }

    pub fn fetch_with_local_identifiers(
        identifiers: &[String],
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<Self>, PhotoKitError> {
        let identifiers_json = json_cstring(identifiers, "collection identifiers")?;
        let options_json = json_cstring(fetch_options, "PHFetchOptions")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_asset_collection_fetch_with_local_identifiers_json(
                identifiers_json.as_ptr(),
                options_json.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(
                    error,
                    "fetch asset collections by local identifier failed",
                )
            })
        } else {
            let collections: Vec<Self> = unsafe { parse_json_ptr(payload, "PHAssetCollection list") }?;
            Ok(collections.into())
        }
    }

    pub fn fetch_containing_asset(
        asset: &PHAsset,
        collection_type: PHAssetCollectionType,
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<Self>, PhotoKitError> {
        let asset_identifier = cstring_from_str(&asset.local_identifier, "asset local identifier")?;
        let options_json = json_cstring(fetch_options, "PHFetchOptions")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_asset_collection_fetch_containing_asset_json(
                asset_identifier.as_ptr(),
                match collection_type {
                    PHAssetCollectionType::Album => 1,
                    PHAssetCollectionType::SmartAlbum => 2,
                },
                options_json.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(
                    error,
                    "fetch asset collections containing asset failed",
                )
            })
        } else {
            let collections: Vec<Self> = unsafe { parse_json_ptr(payload, "PHAssetCollection list") }?;
            Ok(collections.into())
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
            cstring_from_str(&self.local_identifier, "collection local identifier")?;
        let mut error = ptr::null_mut();
        let allowed = unsafe {
            ffi::ph_asset_collection_can_perform_edit_operation(
                collection_identifier.as_ptr(),
                edit_operation.as_raw(),
                &mut error,
            )
        };
        if error.is_null() {
            Ok(allowed != 0)
        } else {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "asset collection edit capability lookup failed")
            })
        }
    }

    pub fn assets(
        &self,
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<PHAsset>, PhotoKitError> {
        PHAsset::fetch_in_asset_collection(self, fetch_options)
    }

    pub fn containing_collection_lists(
        &self,
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<PHCollectionList>, PhotoKitError> {
        PHCollectionList::fetch_containing_collection_local_identifier(
            &self.local_identifier,
            fetch_options,
        )
    }
}
