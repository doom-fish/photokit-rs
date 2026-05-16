#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(
    clippy::cargo_common_metadata,
    clippy::doc_markdown,
    clippy::manual_map,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::option_if_let_else,
    clippy::return_self_not_must_use,
    clippy::single_option_map,
    clippy::struct_excessive_bools,
    clippy::type_complexity
)]

pub mod asset;
pub mod asset_collection;
pub mod asset_creation_request;
pub mod change;
pub mod cloud_identifier;
pub mod collection_list;
pub mod content_editing_input;
pub mod content_editing_output;
pub mod error;
pub mod fetch_options;
pub mod fetch_result;
mod ffi;
pub mod image_manager;
pub mod library;
pub mod live_photo;
pub mod object_change_details;
mod private;
pub mod photo_library;
pub mod types;

pub use asset::{
    PHAsset, PHAssetEditOperation, PHAssetPlaybackStyle, PHAssetResource, PHCoordinate,
    PHMediaType,
};
pub use asset_collection::{PHAssetCollection, PHAssetCollectionType, PHCollectionEditOperation};
pub use asset_creation_request::{
    PHAssetCreationRequest, PHAssetCreationResource, PHAssetResourceCreationOptions,
};
pub use change::PHChange;
pub use cloud_identifier::{PHCloudIdentifier, PHCloudIdentifierMapping, PHLocalIdentifierMapping};
pub use collection_list::{PHCollectionList, PHCollectionListType};
pub use content_editing_input::{
    PHAdjustmentData, PHContentEditingInput, PHContentEditingInputInfo,
    PHContentEditingInputRequestOptions,
};
pub use content_editing_output::{PHContentEditingOutput, PHContentEditingOutputInfo};
pub use error::{NSErrorInfo, PHAuthorizationStatus, PhotoKitError};
pub use fetch_options::{PHFetchOptions, PHSortDescriptor};
pub use fetch_result::PHFetchResult;
pub use image_manager::{
    PHCachingImageManager, PHImageContentMode, PHImageDataRequestHandle, PHImageDataResult,
    PHImageManager, PHImageRequest, PHImageRequestHandle, PHImageRequestOptionsDeliveryMode,
    PHImageRequestOptionsResizeMode, PHImageRequestOptionsVersion, PHImageResult,
    PHLivePhotoRequestHandle,
};
pub use live_photo::{PHLivePhoto, PHLivePhotoResult};
pub use object_change_details::PHObjectChangeDetails;
pub use photo_library::{PHAccessLevel, PHChangeObserver, PHPhotoLibrary, PHPhotoLibraryChange};

/// Common imports.
pub mod prelude {
    pub use crate::asset::{
        PHAsset, PHAssetEditOperation, PHAssetPlaybackStyle, PHAssetResource, PHCoordinate,
        PHMediaType,
    };
    pub use crate::asset_collection::{
        PHAssetCollection, PHAssetCollectionType, PHCollectionEditOperation,
    };
    pub use crate::asset_creation_request::{
        PHAssetCreationRequest, PHAssetCreationResource, PHAssetResourceCreationOptions,
    };
    pub use crate::change::PHChange;
    pub use crate::cloud_identifier::{
        PHCloudIdentifier, PHCloudIdentifierMapping, PHLocalIdentifierMapping,
    };
    pub use crate::collection_list::{PHCollectionList, PHCollectionListType};
    pub use crate::content_editing_input::{
        PHAdjustmentData, PHContentEditingInput, PHContentEditingInputInfo,
        PHContentEditingInputRequestOptions,
    };
    pub use crate::content_editing_output::{PHContentEditingOutput, PHContentEditingOutputInfo};
    pub use crate::error::{NSErrorInfo, PHAuthorizationStatus, PhotoKitError};
    pub use crate::fetch_options::{PHFetchOptions, PHSortDescriptor};
    pub use crate::fetch_result::PHFetchResult;
    pub use crate::image_manager::{
        PHCachingImageManager, PHImageContentMode, PHImageDataRequestHandle, PHImageDataResult,
        PHImageManager, PHImageRequest, PHImageRequestHandle,
        PHImageRequestOptionsDeliveryMode, PHImageRequestOptionsResizeMode,
        PHImageRequestOptionsVersion, PHImageResult, PHLivePhotoRequestHandle,
    };
    pub use crate::live_photo::{PHLivePhoto, PHLivePhotoResult};
    pub use crate::object_change_details::PHObjectChangeDetails;
    pub use crate::photo_library::{PHAccessLevel, PHChangeObserver, PHPhotoLibrary, PHPhotoLibraryChange};
}
