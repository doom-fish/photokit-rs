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
pub struct PHCollectionListChildMutation {
    pub kind: String,
    #[serde(default)]
    pub child_local_identifiers: Vec<String>,
    #[serde(default)]
    pub indexes: Vec<usize>,
    pub to_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PHCollectionListChangeRequest {
    pub collection_list_local_identifier: Option<String>,
    #[serde(default)]
    pub top_level_user_collections: bool,
    pub creation_title: Option<String>,
    pub title: Option<String>,
    #[serde(default)]
    pub child_mutations: Vec<PHCollectionListChildMutation>,
}

impl PHCollectionListChangeRequest {
    pub fn creation_request_for_collection_list(title: impl Into<String>) -> Self {
        Self {
            creation_title: Some(title.into()),
            ..Self::default()
        }
    }

    pub fn change_request_for_collection_list(collection_list: &PHCollectionList) -> Self {
        Self {
            collection_list_local_identifier: Some(collection_list.local_identifier.clone()),
            ..Self::default()
        }
    }

    pub fn change_request_for_top_level_user_collections() -> Self {
        Self {
            top_level_user_collections: true,
            ..Self::default()
        }
    }

    pub fn set_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

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

    pub fn remove_child_collections_at_indexes(mut self, indexes: &[usize]) -> Self {
        self.child_mutations.push(PHCollectionListChildMutation {
            kind: "removeAtIndexes".to_owned(),
            child_local_identifiers: Vec::new(),
            indexes: indexes.to_vec(),
            to_index: None,
        });
        self
    }

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

    pub fn move_child_collections_at_indexes(mut self, indexes: &[usize], to_index: usize) -> Self {
        self.child_mutations.push(PHCollectionListChildMutation {
            kind: "move".to_owned(),
            child_local_identifiers: Vec::new(),
            indexes: indexes.to_vec(),
            to_index: Some(to_index),
        });
        self
    }

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
