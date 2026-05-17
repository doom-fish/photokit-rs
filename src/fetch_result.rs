use serde::{Deserialize, Serialize};

use crate::asset::{PHAsset, PHMediaType};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PHFetchResult<T> {
    pub objects: Vec<T>,
}

impl<T> From<Vec<T>> for PHFetchResult<T> {
    fn from(objects: Vec<T>) -> Self {
        Self { objects }
    }
}

impl<T> PHFetchResult<T> {
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn first(&self) -> Option<&T> {
        self.objects.first()
    }

    pub fn last(&self) -> Option<&T> {
        self.objects.last()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.objects.get(index)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.objects.iter()
    }

    pub fn into_vec(self) -> Vec<T> {
        self.objects
    }
}

impl<T: Clone> PHFetchResult<T> {
    pub fn objects_at_indexes(&self, indexes: &[usize]) -> Vec<T> {
        indexes
            .iter()
            .filter_map(|index| self.objects.get(*index).cloned())
            .collect()
    }
}

impl<T: PartialEq> PHFetchResult<T> {
    pub fn contains(&self, object: &T) -> bool {
        self.objects.contains(object)
    }

    pub fn index_of(&self, object: &T) -> Option<usize> {
        self.objects
            .iter()
            .position(|candidate| candidate == object)
    }
}

impl PHFetchResult<PHAsset> {
    pub fn count_of_assets_with_media_type(&self, media_type: PHMediaType) -> usize {
        self.objects
            .iter()
            .filter(|asset| asset.media_type == media_type)
            .count()
    }
}

impl<T> IntoIterator for PHFetchResult<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.objects.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a PHFetchResult<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
