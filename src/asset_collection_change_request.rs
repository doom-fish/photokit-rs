use serde::{Deserialize, Serialize};

use crate::asset::PHAsset;
use crate::asset_collection::PHAssetCollection;
use crate::change_request::PHChangeRequest;
use crate::error::PhotoKitError;
use crate::ffi;
use crate::object::PHObjectPlaceholder;
use crate::private::json_cstring;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PHChangeRequestPerformResult {
    pub placeholder_local_identifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PHAssetCollectionAssetMutation {
    pub kind: String,
    #[serde(default)]
    pub asset_local_identifiers: Vec<String>,
    #[serde(default)]
    pub indexes: Vec<usize>,
    pub to_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PHAssetCollectionChangeRequest {
    pub asset_collection_local_identifier: Option<String>,
    pub creation_title: Option<String>,
    pub title: Option<String>,
    #[serde(default)]
    pub asset_mutations: Vec<PHAssetCollectionAssetMutation>,
}

impl PHAssetCollectionChangeRequest {
    pub fn creation_request_for_asset_collection(title: impl Into<String>) -> Self {
        Self {
            creation_title: Some(title.into()),
            ..Self::default()
        }
    }

    pub fn change_request_for_asset_collection(collection: &PHAssetCollection) -> Self {
        Self {
            asset_collection_local_identifier: Some(collection.local_identifier.clone()),
            ..Self::default()
        }
    }

    pub fn set_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn add_assets(mut self, assets: &[PHAsset]) -> Self {
        self.asset_mutations.push(PHAssetCollectionAssetMutation {
            kind: "add".to_owned(),
            asset_local_identifiers: assets
                .iter()
                .map(|asset| asset.local_identifier.clone())
                .collect(),
            indexes: Vec::new(),
            to_index: None,
        });
        self
    }

    pub fn insert_assets(mut self, assets: &[PHAsset], indexes: &[usize]) -> Self {
        self.asset_mutations.push(PHAssetCollectionAssetMutation {
            kind: "insert".to_owned(),
            asset_local_identifiers: assets
                .iter()
                .map(|asset| asset.local_identifier.clone())
                .collect(),
            indexes: indexes.to_vec(),
            to_index: None,
        });
        self
    }

    pub fn remove_assets(mut self, assets: &[PHAsset]) -> Self {
        self.asset_mutations.push(PHAssetCollectionAssetMutation {
            kind: "remove".to_owned(),
            asset_local_identifiers: assets
                .iter()
                .map(|asset| asset.local_identifier.clone())
                .collect(),
            indexes: Vec::new(),
            to_index: None,
        });
        self
    }

    pub fn remove_assets_at_indexes(mut self, indexes: &[usize]) -> Self {
        self.asset_mutations.push(PHAssetCollectionAssetMutation {
            kind: "removeAtIndexes".to_owned(),
            asset_local_identifiers: Vec::new(),
            indexes: indexes.to_vec(),
            to_index: None,
        });
        self
    }

    pub fn replace_assets_at_indexes(mut self, indexes: &[usize], assets: &[PHAsset]) -> Self {
        self.asset_mutations.push(PHAssetCollectionAssetMutation {
            kind: "replace".to_owned(),
            asset_local_identifiers: assets
                .iter()
                .map(|asset| asset.local_identifier.clone())
                .collect(),
            indexes: indexes.to_vec(),
            to_index: None,
        });
        self
    }

    pub fn move_assets_at_indexes(mut self, indexes: &[usize], to_index: usize) -> Self {
        self.asset_mutations.push(PHAssetCollectionAssetMutation {
            kind: "move".to_owned(),
            asset_local_identifiers: Vec::new(),
            indexes: indexes.to_vec(),
            to_index: Some(to_index),
        });
        self
    }

    pub fn delete_asset_collections(
        collections: &[PHAssetCollection],
    ) -> Result<(), PhotoKitError> {
        let identifiers: Vec<&str> = collections
            .iter()
            .map(|collection| collection.local_identifier.as_str())
            .collect();
        let identifiers_json = json_cstring(&identifiers, "asset collection identifiers")?;
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::ph_asset_collection_change_request_delete_json(
                identifiers_json.as_ptr(),
                &mut error,
            )
        };
        if status == ffi::status::OK && error.is_null() {
            Ok(())
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "delete asset collections failed") })
        }
    }
}

impl PHChangeRequest for PHAssetCollectionChangeRequest {
    type Output = Option<PHObjectPlaceholder>;

    fn perform(self) -> Result<Self::Output, PhotoKitError> {
        let payload_json = json_cstring(&self, "PHAssetCollectionChangeRequest")?;
        let mut error = core::ptr::null_mut();
        let payload = unsafe {
            ffi::ph_asset_collection_change_request_perform_json(payload_json.as_ptr(), &mut error)
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "asset collection change request failed")
            })
        } else {
            let result: PHChangeRequestPerformResult = unsafe {
                crate::private::parse_json_ptr(payload, "PHAssetCollectionChangeRequest result")
            }?;
            Ok(result
                .placeholder_local_identifier
                .map(PHObjectPlaceholder::new))
        }
    }
}
