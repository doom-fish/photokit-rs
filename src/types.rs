use base64::Engine;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PHMediaType {
    Unknown,
    Image,
    Video,
    Audio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PHAssetCollectionType {
    Album,
    SmartAlbum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum PHImageContentMode {
    #[default]
    Default,
    AspectFit,
    AspectFill,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PHFetchOptions {
    pub predicate: Option<String>,
    #[serde(default)]
    pub sort_descriptors: Vec<PHSortDescriptor>,
    pub fetch_limit: Option<usize>,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PHCoordinate {
    pub latitude: f64,
    pub longitude: f64,
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHAsset {
    pub local_identifier: String,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
    pub pixel_width: u64,
    pub pixel_height: u64,
    pub location: Option<PHCoordinate>,
    pub media_type: PHMediaType,
    pub media_subtypes: u64,
    pub duration: f64,
    pub is_favorite: bool,
}

impl PHAsset {
    pub fn is_live_photo(&self) -> bool {
        self.media_subtypes & (1 << 3) != 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHAssetCollection {
    pub local_identifier: String,
    pub localized_title: Option<String>,
    pub collection_type: PHAssetCollectionType,
    pub collection_subtype: i64,
    pub estimated_asset_count: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHAssetResource {
    pub asset_local_identifier: String,
    pub resource_type: i64,
    pub original_filename: String,
    pub uniform_type_identifier: Option<String>,
    pub pixel_width: Option<i64>,
    pub pixel_height: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PHPhotoLibraryChange {
    pub change_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.objects.iter()
    }

    pub fn into_vec(self) -> Vec<T> {
        self.objects
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHImageRequest {
    pub target_width: f64,
    pub target_height: f64,
    pub content_mode: PHImageContentMode,
}

impl PHImageRequest {
    pub fn new(target_width: f64, target_height: f64, content_mode: PHImageContentMode) -> Self {
        Self {
            target_width,
            target_height,
            content_mode,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHImageResult {
    pub tiff_data_base64: String,
    pub width: f64,
    pub height: f64,
    pub cancelled: bool,
    pub degraded: bool,
}

impl PHImageResult {
    pub fn tiff_data(&self) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(self.tiff_data_base64.as_bytes())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHImageDataResult {
    pub data_base64: String,
    pub uniform_type_identifier: Option<String>,
    pub orientation: i32,
    pub cancelled: bool,
}

impl PHImageDataResult {
    pub fn data(&self) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(self.data_base64.as_bytes())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHLivePhotoResult {
    pub has_live_photo: bool,
    pub cancelled: bool,
    pub degraded: bool,
}
