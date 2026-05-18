use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHObjectChangeDetails`.
pub struct PHObjectChangeDetails<T> {
    /// Corresponds to `PHObjectChangeDetails.objectBeforeChanges`.
    pub object_before_changes: T,
    /// Corresponds to `PHObjectChangeDetails.objectAfterChanges`.
    pub object_after_changes: Option<T>,
    /// Corresponds to `PHObjectChangeDetails.assetContentChanged`.
    pub asset_content_changed: bool,
    /// Corresponds to `PHObjectChangeDetails.objectWasDeleted`.
    pub object_was_deleted: bool,
}
