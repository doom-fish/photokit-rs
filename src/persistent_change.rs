use base64::Engine;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
/// Wraps `PHObjectType`.
pub struct PHObjectType(
    /// Raw value for `PHObjectType`.
    pub i64,
);

impl PHObjectType {
    /// Constant on `PHObjectType`.
    pub const ASSET: Self = Self(1);
    /// Constant on `PHObjectType`.
    pub const ASSET_COLLECTION: Self = Self(2);
    /// Constant on `PHObjectType`.
    pub const COLLECTION_LIST: Self = Self(3);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHPersistentChangeToken`.
pub struct PHPersistentChangeToken {
    /// Corresponds to `PHPersistentChangeToken.dataBase64`.
    pub data_base64: String,
}

impl PHPersistentChangeToken {
    /// Decodes the binary data carried by `PHPersistentChangeToken`.
    pub fn data(&self) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(self.data_base64.as_bytes())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHPersistentObjectChangeDetails`.
pub struct PHPersistentObjectChangeDetails {
    /// Corresponds to `PHPersistentObjectChangeDetails.objectType`.
    pub object_type: PHObjectType,
    #[serde(default)]
    /// Corresponds to `PHPersistentObjectChangeDetails.insertedLocalIdentifiers`.
    pub inserted_local_identifiers: Vec<String>,
    #[serde(default)]
    /// Corresponds to `PHPersistentObjectChangeDetails.updatedLocalIdentifiers`.
    pub updated_local_identifiers: Vec<String>,
    #[serde(default)]
    /// Corresponds to `PHPersistentObjectChangeDetails.deletedLocalIdentifiers`.
    pub deleted_local_identifiers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHPersistentChange`.
pub struct PHPersistentChange {
    /// Corresponds to `PHPersistentChange.changeToken`.
    pub change_token: PHPersistentChangeToken,
    #[serde(default)]
    /// Corresponds to `PHPersistentChange.changeDetails`.
    pub change_details: Vec<PHPersistentObjectChangeDetails>,
}

impl PHPersistentChange {
    /// Wraps a Photos framework operation on `PHPersistentChange`.
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
/// Fetch result wrapper for `PHPersistentChange` values.
pub struct PHPersistentChangeFetchResult {
    #[serde(default)]
    /// Serialized field carried by `PHPersistentChangeFetchResult`.
    pub changes: Vec<PHPersistentChange>,
}

impl PHPersistentChangeFetchResult {
    /// Wraps a Photos framework operation on `PHPersistentChangeFetchResult`.
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    /// Queries Photos framework state exposed by `PHPersistentChangeFetchResult`.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Wraps a Photos framework operation on `PHPersistentChangeFetchResult`.
    pub fn iter(&self) -> std::slice::Iter<'_, PHPersistentChange> {
        self.changes.iter()
    }

    /// Wraps a Photos framework operation on `PHPersistentChangeFetchResult`.
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
