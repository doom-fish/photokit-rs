use base64::Engine;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PHObjectType(pub i64);

impl PHObjectType {
    pub const ASSET: Self = Self(1);
    pub const ASSET_COLLECTION: Self = Self(2);
    pub const COLLECTION_LIST: Self = Self(3);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHPersistentChangeToken {
    pub data_base64: String,
}

impl PHPersistentChangeToken {
    pub fn data(&self) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(self.data_base64.as_bytes())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHPersistentObjectChangeDetails {
    pub object_type: PHObjectType,
    #[serde(default)]
    pub inserted_local_identifiers: Vec<String>,
    #[serde(default)]
    pub updated_local_identifiers: Vec<String>,
    #[serde(default)]
    pub deleted_local_identifiers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHPersistentChange {
    pub change_token: PHPersistentChangeToken,
    #[serde(default)]
    pub change_details: Vec<PHPersistentObjectChangeDetails>,
}

impl PHPersistentChange {
    pub fn change_details_for_object_type(
        &self,
        object_type: PHObjectType,
    ) -> Option<&PHPersistentObjectChangeDetails> {
        self.change_details
            .iter()
            .find(|details| details.object_type == object_type)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PHPersistentChangeFetchResult {
    #[serde(default)]
    pub changes: Vec<PHPersistentChange>,
}

impl PHPersistentChangeFetchResult {
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, PHPersistentChange> {
        self.changes.iter()
    }

    pub fn into_vec(self) -> Vec<PHPersistentChange> {
        self.changes
    }
}

impl IntoIterator for PHPersistentChangeFetchResult {
    type Item = PHPersistentChange;
    type IntoIter = std::vec::IntoIter<PHPersistentChange>;

    fn into_iter(self) -> Self::IntoIter {
        self.changes.into_iter()
    }
}

impl<'a> IntoIterator for &'a PHPersistentChangeFetchResult {
    type Item = &'a PHPersistentChange;
    type IntoIter = std::slice::Iter<'a, PHPersistentChange>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
