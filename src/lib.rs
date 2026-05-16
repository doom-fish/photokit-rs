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

pub mod error;
mod ffi;
pub mod library;
mod private;
pub mod types;

pub use error::{NSErrorInfo, PHAuthorizationStatus, PhotoKitError};
pub use library::{
    PHCachingImageManager, PHChangeObserver, PHImageDataRequestHandle, PHImageManager,
    PHImageRequestHandle, PHLivePhotoRequestHandle, PHPhotoLibrary,
};
pub use types::{
    PHAsset, PHAssetCollection, PHAssetCollectionType, PHAssetResource, PHCoordinate,
    PHFetchOptions, PHFetchResult, PHImageContentMode, PHImageDataResult, PHImageRequest,
    PHImageResult, PHLivePhotoResult, PHMediaType, PHPhotoLibraryChange, PHSortDescriptor,
};

/// Common imports.
pub mod prelude {
    pub use crate::error::{NSErrorInfo, PHAuthorizationStatus, PhotoKitError};
    pub use crate::library::{
        PHCachingImageManager, PHChangeObserver, PHImageDataRequestHandle, PHImageManager,
        PHImageRequestHandle, PHLivePhotoRequestHandle, PHPhotoLibrary,
    };
    pub use crate::types::{
        PHAsset, PHAssetCollection, PHAssetCollectionType, PHAssetResource, PHCoordinate,
        PHFetchOptions, PHFetchResult, PHImageContentMode, PHImageDataResult, PHImageRequest,
        PHImageResult, PHLivePhotoResult, PHMediaType, PHPhotoLibraryChange, PHSortDescriptor,
    };
}
