use std::ops::{BitOr, BitOrAssign};
use std::ptr;

use serde::{Deserialize, Serialize};

use crate::asset_collection::PHAssetCollection;
use crate::error::PhotoKitError;
use crate::fetch_options::PHFetchOptions;
use crate::fetch_result::PHFetchResult;
use crate::ffi;
use crate::private::{cstring_from_str, json_cstring, parse_json_ptr};

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
pub enum PHAssetPlaybackStyle {
    Unsupported,
    Image,
    ImageAnimated,
    LivePhoto,
    Video,
    VideoLooping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PHAssetEditOperation {
    Delete,
    Content,
    Properties,
}

impl PHAssetEditOperation {
    pub(crate) const fn as_raw(self) -> i32 {
        match self {
            Self::Delete => 1,
            Self::Content => 2,
            Self::Properties => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PHCoordinate {
    pub latitude: f64,
    pub longitude: f64,
}

macro_rules! option_set_type {
    ($name:ident, $raw:ty, { $($constant:ident = $value:expr,)* }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
        #[serde(transparent)]
        pub struct $name(pub $raw);

        impl $name {
            $(pub const $constant: Self = Self($value);)*

            pub const fn bits(self) -> $raw {
                self.0
            }

            pub const fn contains(self, other: Self) -> bool {
                self.0 & other.0 == other.0
            }
        }

        impl From<$raw> for $name {
            fn from(value: $raw) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $raw {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl BitOr for $name {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }

        impl BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }
    };
}

option_set_type!(PHAssetMediaSubtype, u64, {
    NONE = 0,
    PHOTO_PANORAMA = 1 << 0,
    PHOTO_HDR = 1 << 1,
    PHOTO_SCREENSHOT = 1 << 2,
    PHOTO_LIVE = 1 << 3,
    PHOTO_DEPTH_EFFECT = 1 << 4,
    SPATIAL_MEDIA = 1 << 10,
    VIDEO_STREAMED = 1 << 16,
    VIDEO_HIGH_FRAME_RATE = 1 << 17,
    VIDEO_TIMELAPSE = 1 << 18,
    VIDEO_SCREEN_RECORDING = 1 << 19,
    VIDEO_CINEMATIC = 1 << 21,
});

option_set_type!(PHAssetBurstSelectionType, u64, {
    NONE = 0,
    AUTO_PICK = 1 << 0,
    USER_PICK = 1 << 1,
});

option_set_type!(PHAssetSourceType, u64, {
    NONE = 0,
    USER_LIBRARY = 1 << 0,
    CLOUD_SHARED = 1 << 1,
    ITUNES_SYNCED = 1 << 2,
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PHAssetResourceType(pub i64);

impl PHAssetResourceType {
    pub const PHOTO: Self = Self(1);
    pub const VIDEO: Self = Self(2);
    pub const AUDIO: Self = Self(3);
    pub const ALTERNATE_PHOTO: Self = Self(4);
    pub const FULL_SIZE_PHOTO: Self = Self(5);
    pub const FULL_SIZE_VIDEO: Self = Self(6);
    pub const ADJUSTMENT_DATA: Self = Self(7);
    pub const ADJUSTMENT_BASE_PHOTO: Self = Self(8);
    pub const PAIRED_VIDEO: Self = Self(9);
    pub const FULL_SIZE_PAIRED_VIDEO: Self = Self(10);
    pub const ADJUSTMENT_BASE_PAIRED_VIDEO: Self = Self(11);
    pub const ADJUSTMENT_BASE_VIDEO: Self = Self(12);
    pub const PHOTO_PROXY: Self = Self(19);

    pub const fn raw_value(self) -> i64 {
        self.0
    }
}

impl From<i64> for PHAssetResourceType {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<PHAssetResourceType> for i64 {
    fn from(value: PHAssetResourceType) -> Self {
        value.0
    }
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHAsset {
    pub local_identifier: String,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
    #[serde(default)]
    pub added_date: Option<String>,
    pub pixel_width: u64,
    pub pixel_height: u64,
    pub location: Option<PHCoordinate>,
    pub media_type: PHMediaType,
    pub media_subtypes: PHAssetMediaSubtype,
    pub duration: f64,
    #[serde(default)]
    pub is_hidden: bool,
    pub is_favorite: bool,
    #[serde(default)]
    pub playback_style: Option<PHAssetPlaybackStyle>,
    #[serde(default)]
    pub content_type_identifier: Option<String>,
    #[serde(default)]
    pub burst_identifier: Option<String>,
    #[serde(default)]
    pub burst_selection_types: PHAssetBurstSelectionType,
    #[serde(default)]
    pub represents_burst: bool,
    #[serde(default)]
    pub source_type: PHAssetSourceType,
    #[serde(default)]
    pub has_adjustments: bool,
    #[serde(default)]
    pub adjustment_format_identifier: Option<String>,
}

impl PHAsset {
    pub fn is_live_photo(&self) -> bool {
        self.media_subtypes
            .contains(PHAssetMediaSubtype::PHOTO_LIVE)
    }

    pub fn fetch(fetch_options: &PHFetchOptions) -> Result<PHFetchResult<Self>, PhotoKitError> {
        let options_json = json_cstring(fetch_options, "PHFetchOptions")?;
        let mut error = ptr::null_mut();
        let payload = unsafe { ffi::ph_asset_fetch_all_json(options_json.as_ptr(), &mut error) };
        if payload.is_null() {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "fetch assets failed") })
        } else {
            let assets: Vec<Self> = unsafe { parse_json_ptr(payload, "PHAsset list") }?;
            Ok(assets.into())
        }
    }

    pub fn fetch_with_media_type(
        media_type: PHMediaType,
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<Self>, PhotoKitError> {
        let options_json = json_cstring(fetch_options, "PHFetchOptions")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_asset_fetch_with_media_type_json(
                match media_type {
                    PHMediaType::Unknown => 0,
                    PHMediaType::Image => 1,
                    PHMediaType::Video => 2,
                    PHMediaType::Audio => 3,
                },
                options_json.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "fetch assets by media type failed")
            })
        } else {
            let assets: Vec<Self> = unsafe { parse_json_ptr(payload, "PHAsset list") }?;
            Ok(assets.into())
        }
    }

    pub fn fetch_with_local_identifiers(
        identifiers: &[String],
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<Self>, PhotoKitError> {
        let identifiers_json = json_cstring(identifiers, "asset identifiers")?;
        let options_json = json_cstring(fetch_options, "PHFetchOptions")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_asset_fetch_with_local_identifiers_json(
                identifiers_json.as_ptr(),
                options_json.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "fetch assets by local identifier failed")
            })
        } else {
            let assets: Vec<Self> = unsafe { parse_json_ptr(payload, "PHAsset list") }?;
            Ok(assets.into())
        }
    }

    pub fn fetch_in_asset_collection(
        collection: &PHAssetCollection,
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<Self>, PhotoKitError> {
        let collection_identifier =
            cstring_from_str(&collection.local_identifier, "collection local identifier")?;
        let options_json = json_cstring(fetch_options, "PHFetchOptions")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_asset_fetch_in_collection_json(
                collection_identifier.as_ptr(),
                options_json.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "fetch assets in asset collection failed")
            })
        } else {
            let assets: Vec<Self> = unsafe { parse_json_ptr(payload, "PHAsset list") }?;
            Ok(assets.into())
        }
    }

    pub fn fetch_key_assets_in_asset_collection(
        collection: &PHAssetCollection,
        fetch_options: &PHFetchOptions,
    ) -> Result<PHFetchResult<Self>, PhotoKitError> {
        let collection_identifier =
            cstring_from_str(&collection.local_identifier, "collection local identifier")?;
        let options_json = json_cstring(fetch_options, "PHFetchOptions")?;
        let mut error = ptr::null_mut();
        let payload = unsafe {
            ffi::ph_asset_fetch_key_assets_in_collection_json(
                collection_identifier.as_ptr(),
                options_json.as_ptr(),
                &mut error,
            )
        };
        if payload.is_null() {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "fetch key assets in asset collection failed")
            })
        } else {
            let assets: Vec<Self> = unsafe { parse_json_ptr(payload, "PHAsset list") }?;
            Ok(assets.into())
        }
    }

    pub fn from_local_identifier(
        local_identifier: impl Into<String>,
    ) -> Result<Option<Self>, PhotoKitError> {
        let result = Self::fetch_with_local_identifiers(
            &[local_identifier.into()],
            &PHFetchOptions::default(),
        )?;
        Ok(result.into_vec().into_iter().next())
    }

    pub fn can_perform_edit_operation(
        &self,
        edit_operation: PHAssetEditOperation,
    ) -> Result<bool, PhotoKitError> {
        let asset_identifier = cstring_from_str(&self.local_identifier, "asset local identifier")?;
        let mut error = ptr::null_mut();
        let allowed = unsafe {
            ffi::ph_asset_can_perform_edit_operation(
                asset_identifier.as_ptr(),
                edit_operation.as_raw(),
                &mut error,
            )
        };
        if error.is_null() {
            Ok(allowed != 0)
        } else {
            Err(unsafe {
                PhotoKitError::from_error_ptr(error, "asset edit capability lookup failed")
            })
        }
    }

    pub fn resources(&self) -> Result<Vec<PHAssetResource>, PhotoKitError> {
        let identifier = cstring_from_str(&self.local_identifier, "asset local identifier")?;
        let mut error = ptr::null_mut();
        let payload = unsafe { ffi::ph_asset_resources_json(identifier.as_ptr(), &mut error) };
        if payload.is_null() {
            Err(unsafe { PhotoKitError::from_error_ptr(error, "asset resources failed") })
        } else {
            unsafe { parse_json_ptr(payload, "PHAssetResource list") }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHAssetResource {
    pub asset_local_identifier: String,
    pub resource_type: PHAssetResourceType,
    pub original_filename: String,
    pub uniform_type_identifier: Option<String>,
    #[serde(default)]
    pub content_type_identifier: Option<String>,
    pub pixel_width: Option<i64>,
    pub pixel_height: Option<i64>,
}

impl PHAssetResource {
    pub fn for_asset(asset: &PHAsset) -> Result<Vec<Self>, PhotoKitError> {
        asset.resources()
    }
}
