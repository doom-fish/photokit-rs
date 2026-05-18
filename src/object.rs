use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHObject`.
pub struct PHObject {
    /// Corresponds to `PHObject.localIdentifier`.
    pub local_identifier: String,
}

impl PHObject {
    /// Creates a helper value for the related Photos framework API.
    pub fn new(local_identifier: impl Into<String>) -> Self {
        Self {
            local_identifier: local_identifier.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `PHObjectPlaceholder`.
pub struct PHObjectPlaceholder {
    /// Corresponds to `PHObjectPlaceholder.localIdentifier`.
    pub local_identifier: String,
}

impl PHObjectPlaceholder {
    /// Creates a helper value for the related Photos framework API.
    pub fn new(local_identifier: impl Into<String>) -> Self {
        Self {
            local_identifier: local_identifier.into(),
        }
    }

    /// Wraps a Photos framework operation on `PHObjectPlaceholder`.
    pub fn object(&self) -> PHObject {
        PHObject::new(self.local_identifier.clone())
    }
}

impl From<PHObjectPlaceholder> for PHObject {
    fn from(value: PHObjectPlaceholder) -> Self {
        Self::new(value.local_identifier)
    }
}

impl From<&PHObjectPlaceholder> for PHObject {
    fn from(value: &PHObjectPlaceholder) -> Self {
        value.object()
    }
}
