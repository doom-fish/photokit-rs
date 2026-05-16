use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PHSortDescriptor {
    pub key: String,
    pub ascending: bool,
}

impl PHSortDescriptor {
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
pub struct PHFetchOptions {
    pub predicate: Option<String>,
    #[serde(default)]
    pub sort_descriptors: Vec<PHSortDescriptor>,
    #[serde(default)]
    pub include_hidden_assets: bool,
    #[serde(default)]
    pub include_all_burst_assets: bool,
    pub include_asset_source_types: Option<u64>,
    pub fetch_limit: Option<usize>,
    #[serde(default = "default_true")]
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
    pub fn with_predicate(mut self, predicate: impl Into<String>) -> Self {
        self.predicate = Some(predicate.into());
        self
    }

    pub fn with_fetch_limit(mut self, fetch_limit: usize) -> Self {
        self.fetch_limit = Some(fetch_limit);
        self
    }

    pub fn with_sort_descriptor(mut self, descriptor: PHSortDescriptor) -> Self {
        self.sort_descriptors.push(descriptor);
        self
    }

    pub fn with_include_hidden_assets(mut self, include_hidden_assets: bool) -> Self {
        self.include_hidden_assets = include_hidden_assets;
        self
    }

    pub fn with_include_all_burst_assets(mut self, include_all_burst_assets: bool) -> Self {
        self.include_all_burst_assets = include_all_burst_assets;
        self
    }

    pub fn with_include_asset_source_types(mut self, include_asset_source_types: u64) -> Self {
        self.include_asset_source_types = Some(include_asset_source_types);
        self
    }

    pub fn with_wants_incremental_change_details(
        mut self,
        wants_incremental_change_details: bool,
    ) -> Self {
        self.wants_incremental_change_details = wants_incremental_change_details;
        self
    }
}
