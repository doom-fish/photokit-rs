use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHObject {
    pub local_identifier: String,
}

impl PHObject {
    pub fn new(local_identifier: impl Into<String>) -> Self {
        Self {
            local_identifier: local_identifier.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHObjectPlaceholder {
    pub local_identifier: String,
}

impl PHObjectPlaceholder {
    pub fn new(local_identifier: impl Into<String>) -> Self {
        Self {
            local_identifier: local_identifier.into(),
        }
    }

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
