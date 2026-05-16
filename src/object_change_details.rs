use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHObjectChangeDetails<T> {
    pub object_before_changes: T,
    pub object_after_changes: Option<T>,
    pub asset_content_changed: bool,
    pub object_was_deleted: bool,
}
