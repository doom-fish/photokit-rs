use serde::{Deserialize, Serialize};

use crate::asset::PHAssetSourceType;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Sort descriptor wrapper used with `PHFetchOptions`.
pub struct PHSortDescriptor {
    /// Serialized field carried by `PHSortDescriptor`.
    pub key: String,
    /// Serialized field carried by `PHSortDescriptor`.
    pub ascending: bool,
}

impl PHSortDescriptor {
    /// Creates a helper value for the related Photos framework API.
    pub fn new(key: impl Into<String>, ascending: bool) -> Self {
        Self {
            key: key.into(),
            ascending,
        }
    }
}

const fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHFetchOptions`.
pub struct PHFetchOptions {
    /// Serialized field carried by `PHFetchOptions`.
    pub predicate: Option<String>,
    #[serde(default)]
    /// Serialized field carried by `PHFetchOptions`.
    pub sort_descriptors: Vec<PHSortDescriptor>,
    #[serde(default)]
    /// Serialized field carried by `PHFetchOptions`.
    pub include_hidden_assets: bool,
    #[serde(default)]
    /// Serialized field carried by `PHFetchOptions`.
    pub include_all_burst_assets: bool,
    /// Serialized field carried by `PHFetchOptions`.
    pub include_asset_source_types: Option<PHAssetSourceType>,
    /// Serialized field carried by `PHFetchOptions`.
    pub fetch_limit: Option<usize>,
    #[serde(default = "default_true")]
    /// Serialized field carried by `PHFetchOptions`.
    pub wants_incremental_change_details: bool,
}

impl Default for PHFetchOptions {
    fn default() -> Self {
        Self {
            predicate: None,
            sort_descriptors: Vec::new(),
            include_hidden_assets: false,
            include_all_burst_assets: false,
            include_asset_source_types: None,
            fetch_limit: None,
            wants_incremental_change_details: true,
        }
    }
}

impl PHFetchOptions {
    /// Sets a Photos framework option on `PHFetchOptions`.
    pub fn with_predicate(mut self, predicate: impl Into<String>) -> Self {
        self.predicate = Some(predicate.into());
        self
    }

    /// Sets a Photos framework option on `PHFetchOptions`.
    pub fn with_fetch_limit(mut self, fetch_limit: usize) -> Self {
        self.fetch_limit = Some(fetch_limit);
        self
    }

    /// Sets a Photos framework option on `PHFetchOptions`.
    pub fn with_sort_descriptor(mut self, descriptor: PHSortDescriptor) -> Self {
        self.sort_descriptors.push(descriptor);
        self
    }

    /// Sets a Photos framework option on `PHFetchOptions`.
    pub fn with_include_hidden_assets(mut self, include_hidden_assets: bool) -> Self {
        self.include_hidden_assets = include_hidden_assets;
        self
    }

    /// Sets a Photos framework option on `PHFetchOptions`.
    pub fn with_include_all_burst_assets(mut self, include_all_burst_assets: bool) -> Self {
        self.include_all_burst_assets = include_all_burst_assets;
        self
    }

    /// Sets a Photos framework option on `PHFetchOptions`.
    pub fn with_include_asset_source_types(
        mut self,
        include_asset_source_types: impl Into<PHAssetSourceType>,
    ) -> Self {
        self.include_asset_source_types = Some(include_asset_source_types.into());
        self
    }

    /// Sets a Photos framework option on `PHFetchOptions`.
    pub fn with_wants_incremental_change_details(
        mut self,
        wants_incremental_change_details: bool,
    ) -> Self {
        self.wants_incremental_change_details = wants_incremental_change_details;
        self
    }
}
