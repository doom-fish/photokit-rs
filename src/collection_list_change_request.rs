use serde::{Deserialize, Serialize};

use crate::change_request::PHChangeRequest;
use crate::collection::PHCollection;
use crate::collection_list::PHCollectionList;
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
/// Helper mutation batch for `PHCollectionListChangeRequest` child-collection edits.
pub struct PHCollectionListChildMutation {
    /// Corresponds to `PHCollectionListChildMutation.kind`.
    pub kind: String,
    #[serde(default)]
    /// Corresponds to `PHCollectionListChildMutation.childLocalIdentifiers`.
    pub child_local_identifiers: Vec<String>,
    #[serde(default)]
    /// Corresponds to `PHCollectionListChildMutation.indexes`.
    pub indexes: Vec<usize>,
    /// Corresponds to `PHCollectionListChildMutation.toIndex`.
    pub to_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHCollectionListChangeRequest`.
pub struct PHCollectionListChangeRequest {
    /// Serialized field carried by `PHCollectionListChangeRequest`.
    pub collection_list_local_identifier: Option<String>,
    #[serde(default)]
    /// Serialized field carried by `PHCollectionListChangeRequest`.
    pub top_level_user_collections: bool,
    /// Serialized field carried by `PHCollectionListChangeRequest`.
    pub creation_title: Option<String>,
    /// Serialized field carried by `PHCollectionListChangeRequest`.
    pub title: Option<String>,
    #[serde(default)]
    /// Serialized field carried by `PHCollectionListChangeRequest`.
    pub child_mutations: Vec<PHCollectionListChildMutation>,
}

impl PHCollectionListChangeRequest {
    /// Wraps a Photos framework operation on `PHCollectionListChangeRequest`.
    pub fn creation_request_for_collection_list(title: impl Into<String>) -> Self {
        Self {
            creation_title: Some(title.into()),
            ..Self::default()
        }
    }

    /// Wraps a Photos framework operation on `PHCollectionListChangeRequest`.
    pub fn change_request_for_collection_list(collection_list: &PHCollectionList) -> Self {
        Self {
            collection_list_local_identifier: Some(collection_list.local_identifier.clone()),
            ..Self::default()
        }
    }

    /// Wraps a Photos framework operation on `PHCollectionListChangeRequest`.
    pub fn change_request_for_top_level_user_collections() -> Self {
        Self {
            top_level_user_collections: true,
            ..Self::default()
        }
    }

    /// Updates the wrapped Photos framework value on `PHCollectionListChangeRequest`.
    pub fn set_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Wraps a Photos framework operation on `PHCollectionListChangeRequest`.
    pub fn add_child_collections(mut self, collections: &[PHCollection]) -> Self {
        self.child_mutations.push(PHCollectionListChildMutation {
            kind: "add".to_owned(),
            child_local_identifiers: collections
                .iter()
                .map(|collection| collection.local_identifier.clone())
                .collect(),
            indexes: Vec::new(),
            to_index: None,
        });
        self
    }

    /// Wraps a Photos framework operation on `PHCollectionListChangeRequest`.
    pub fn insert_child_collections(
        mut self,
        collections: &[PHCollection],
        indexes: &[usize],
    ) -> Self {
        self.child_mutations.push(PHCollectionListChildMutation {
            kind: "insert".to_owned(),
            child_local_identifiers: collections
                .iter()
                .map(|collection| collection.local_identifier.clone())
                .collect(),
            indexes: indexes.to_vec(),
            to_index: None,
        });
        self
    }

    /// Wraps a Photos framework operation on `PHCollectionListChangeRequest`.
    pub fn remove_child_collections(mut self, collections: &[PHCollection]) -> Self {
        self.child_mutations.push(PHCollectionListChildMutation {
            kind: "remove".to_owned(),
            child_local_identifiers: collections
                .iter()
                .map(|collection| collection.local_identifier.clone())
                .collect(),
            indexes: Vec::new(),
            to_index: None,
        });
        self
    }

    /// Wraps a Photos framework operation on `PHCollectionListChangeRequest`.
    pub fn remove_child_collections_at_indexes(mut self, indexes: &[usize]) -> Self {
        self.child_mutations.push(PHCollectionListChildMutation {
            kind: "removeAtIndexes".to_owned(),
            child_local_identifiers: Vec::new(),
            indexes: indexes.to_vec(),
            to_index: None,
        });
        self
    }

    /// Wraps a Photos framework operation on `PHCollectionListChangeRequest`.
    pub fn replace_child_collections_at_indexes(
        mut self,
        indexes: &[usize],
        collections: &[PHCollection],
    ) -> Self {
        self.child_mutations.push(PHCollectionListChildMutation {
            kind: "replace".to_owned(),
            child_local_identifiers: collections
                .iter()
                .map(|collection| collection.local_identifier.clone())
                .collect(),
            indexes: indexes.to_vec(),
            to_index: None,
        });
        self
    }

    /// Wraps a Photos framework operation on `PHCollectionListChangeRequest`.
    pub fn move_child_collections_at_indexes(mut self, indexes: &[usize], to_index: usize) -> Self {
        self.child_mutations.push(PHCollectionListChildMutation {
            kind: "move".to_owned(),
            child_local_identifiers: Vec::new(),
            indexes: indexes.to_vec(),
            to_index: Some(to_index),
        });
        self
    }

    /// Wraps a Photos framework operation on `PHCollectionListChangeRequest`.
    pub fn delete_collection_lists(
        collection_lists: &[PHCollectionList],
    ) -> Result<(), PhotoKitError> {
        let identifiers: Vec<&str> = collection_lists
            .iter()
            .map(|collection_list| collection_list.local_identifier.as_str())
            .collect();
        let identifiers_json = json_cstring(&identifiers, "collection list identifiers")?;
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::ph_collection_list_change_request_delete_json(
                identifiers_json.as_ptr(),
                &mut error,
            )
        };
        if status == ffi::status::OK && error.is_null() {
            Ok(())
        } else {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "delete collection lists failed") })
        }
    }
}

impl PHChangeRequest for PHCollectionListChangeRequest {
    type Output = Option<PHObjectPlaceholder>;

    fn perform(self) -> Result<Self::Output, PhotoKitError> {
        let payload_json = json_cstring(&self, "PHCollectionListChangeRequest")?;
        let mut error = core::ptr::null_mut();
        let payload = unsafe {
            ffi::ph_collection_list_change_request_perform_json(payload_json.as_ptr(), &mut error)
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "collection list change request failed")
            })
        } else {
            let result: PHChangeRequestPerformResult = unsafe {
                crate::private::parse_json_ptr(payload, "PHCollectionListChangeRequest result")
            }?;
            Ok(result
                .placeholder_local_identifier
                .map(PHObjectPlaceholder::new))
        }
    }
}
