use std::ptr;

use serde::{Deserialize, Serialize};

use crate::asset_collection::{PHAssetCollection, PHCollectionEditOperation};
use crate::collection_list::PHCollectionList;
use crate::error::PhotoKitError;
use crate::fetch_options::PHFetchOptions;
use crate::fetch_result::PHFetchResult;
use crate::ffi;
use crate::object::PHObject;
use crate::private::{cstring_from_str, json_cstring, parse_json_ptr};

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHCollection`.
pub struct PHCollection {
    /// Corresponds to `PHCollection.localIdentifier`.
    pub local_identifier: String,
    /// Corresponds to `PHCollection.localizedTitle`.
    pub localized_title: Option<String>,
    /// Corresponds to `PHCollection.canContainAssets`.
    pub can_contain_assets: bool,
    /// Corresponds to `PHCollection.canContainCollections`.
    pub can_contain_collections: bool,
    /// Corresponds to `PHCollection.kind`.
    pub kind: String,
}

impl PHCollection {
    /// Wraps a Photos framework operation on `PHCollection`.
    pub fn object(&self) -> PHObject {
        PHObject::new(self.local_identifier.clone())
    }

    /// Queries Photos framework state exposed by `PHCollection`.
    pub fn is_asset_collection(&self) -> bool {
        self.kind == "assetCollection"
    }

    /// Queries Photos framework state exposed by `PHCollection`.
    pub fn is_collection_list(&self) -> bool {
        self.kind == "collectionList"
    }

    /// Queries Photos framework state exposed by `PHCollection`.
    pub fn is_project(&self) -> bool {
        self.kind == "project"
    }

    /// Wraps a Photos framework fetch operation on `PHCollection`.
    pub fn fetch_collections_in_collection_list(
        collection_list: &PHCollectionList,
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<Self>, PhotoKitError> {
        let identifier = cstring_from_str(
            &collection_list.local_identifier,
            "collection list local identifier",
        )?;
        let options_json = json_cstring(fetch_options, "PHFetchOptions")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_collection_fetch_in_collection_list_json(
                identifier.as_ptr(),
                options_json.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "fetch collections in list failed") })
        } else {
            let collections: Vec<Self> = unsafe { parse_json_ptr(payload, "PHCollection list") }?;
            Ok(collections.into())
        }
    }

    /// Wraps a Photos framework fetch operation on `PHCollection`.
    pub fn fetch_top_level_user_collections(
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<Self>, PhotoKitError> {
        let options_json = json_cstring(fetch_options, "PHFetchOptions")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_collection_fetch_top_level_user_collections_json(
                options_json.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "fetch top level user collections failed")
            })
        } else {
            let collections: Vec<Self> = unsafe { parse_json_ptr(payload, "PHCollection list") }?;
            Ok(collections.into())
        }
    }

    /// Queries Photos framework state exposed by `PHCollection`.
    pub fn can_perform_edit_operation(
        &self,
        edit_operation: PHCollectionEditOperation,
    ) -> Result<bool, PhotoKitError> {
        let identifier = cstring_from_str(&self.local_identifier, "collection local identifier")?;
        let mut error = ptr::null_mut();
        let allowed = unsafe {
            ffi::ph_collection_can_perform_edit_operation(
                identifier.as_ptr(),
                edit_operation.as_raw(),
                &mut error,
            )
        };
        if error.is_null() {
            Ok(allowed != 0)
        } else {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "collection edit capability lookup failed")
            })
        }
    }

    /// Wraps a Photos framework operation on `PHCollection`.
    pub fn asset_collection(&self) -> Result<Option<PHAssetCollection>, PhotoKitError> {
        PHAssetCollection::from_local_identifier(self.local_identifier.clone())
    }

    /// Wraps a Photos framework operation on `PHCollection`.
    pub fn collection_list(&self) -> Result<Option<PHCollectionList>, PhotoKitError> {
        PHCollectionList::from_local_identifier(self.local_identifier.clone())
    }
}

impl From<&PHAssetCollection> for PHCollection {
    fn from(value: &PHAssetCollection) -> Self {
        Self {
            local_identifier: value.local_identifier.clone(),
            localized_title: value.localized_title.clone(),
            can_contain_assets: value.can_contain_assets,
            can_contain_collections: value.can_contain_collections,
            kind: "assetCollection".to_owned(),
        }
    }
}

impl From<&PHCollectionList> for PHCollection {
    fn from(value: &PHCollectionList) -> Self {
        Self {
            local_identifier: value.local_identifier.clone(),
            localized_title: value.localized_title.clone(),
            can_contain_assets: value.can_contain_assets,
            can_contain_collections: value.can_contain_collections,
            kind: "collectionList".to_owned(),
        }
    }
}
