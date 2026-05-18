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

/// Wraps `PHAsset`, `PHAssetResource`, and related Photos framework asset types.
pub mod asset;
/// Wraps `PHAssetChangeRequest` mutation APIs.
pub mod asset_change_request;
/// Wraps `PHAssetCollection` and related Photos framework collection types.
pub mod asset_collection;
/// Wraps `PHAssetCollectionChangeRequest` mutation APIs.
pub mod asset_collection_change_request;
/// Wraps `PHAssetCreationRequest` creation APIs.
pub mod asset_creation_request;
/// Wraps `PHAssetResourceManager`.
pub mod asset_resource_manager;
#[cfg(feature = "async")]
/// Async wrappers for Photos framework completion-handler APIs.
pub mod async_api;
/// Wraps `PHChange`.
pub mod change;
/// Traits for Photos framework change-request wrappers.
pub mod change_request;
/// Wraps `PHCloudIdentifier` and related Photos framework identifier mappings.
pub mod cloud_identifier;
/// Wraps `PHCollection`.
pub mod collection;
/// Wraps `PHCollectionList` and related Photos framework list types.
pub mod collection_list;
/// Wraps `PHCollectionListChangeRequest` mutation APIs.
pub mod collection_list_change_request;
/// Wraps `PHContentEditingInput` and related Photos framework editing-input types.
pub mod content_editing_input;
/// Wraps `PHContentEditingOutput` and related Photos framework editing-output types.
pub mod content_editing_output;
/// Wraps Photos framework authorization and error values.
pub mod error;
/// Wraps `PHFetchOptions`.
pub mod fetch_options;
/// Wraps `PHFetchResult`.
pub mod fetch_result;
/// Wraps `PHFetchResultChangeDetails`.
pub mod fetch_result_change_details;
mod ffi;
/// Wraps `PHImageManager` and `PHCachingImageManager`.
pub mod image_manager;
/// Re-exports Photos framework wrapper types.
pub mod library;
/// Wraps `PHLivePhoto` and related request results.
pub mod live_photo;
/// Wraps `PHLivePhotoEditingContext` and related frame-processing types.
pub mod live_photo_editing_context;
/// Wraps `PHObject` and `PHObjectPlaceholder`.
pub mod object;
/// Wraps `PHObjectChangeDetails`.
pub mod object_change_details;
/// Wraps persistent change history Photos framework types.
pub mod persistent_change;
/// Wraps `PHPhotoLibrary` and observer registration APIs.
pub mod photo_library;
mod private;
/// Wraps `PHProject` and `PHProjectChangeRequest`.
pub mod project;
/// Re-exports shared Photos framework wrapper types.
pub mod types;

pub use asset::{
    PHAsset, PHAssetBurstSelectionType, PHAssetEditOperation, PHAssetMediaSubtype,
    PHAssetPlaybackStyle, PHAssetResource, PHAssetResourceType, PHAssetSourceType, PHCoordinate,
    PHMediaType,
};
pub use asset_change_request::PHAssetChangeRequest;
pub use asset_collection::{
    PHAssetCollection, PHAssetCollectionSubtype, PHAssetCollectionType, PHCollectionEditOperation,
};
pub use asset_collection_change_request::{
    PHAssetCollectionAssetMutation, PHAssetCollectionChangeRequest,
};
pub use asset_creation_request::{
    PHAssetCreationRequest, PHAssetCreationResource, PHAssetResourceCreationOptions,
};
pub use asset_resource_manager::{
    PHAssetResourceDataResult, PHAssetResourceManager, PHAssetResourceRequestOptions,
    PHAssetResourceWriteResult,
};
pub use change::PHChange;
pub use change_request::PHChangeRequest;
pub use cloud_identifier::{PHCloudIdentifier, PHCloudIdentifierMapping, PHLocalIdentifierMapping};
pub use collection::PHCollection;
pub use collection_list::{PHCollectionList, PHCollectionListSubtype, PHCollectionListType};
pub use collection_list_change_request::{
    PHCollectionListChangeRequest, PHCollectionListChildMutation,
};
pub use content_editing_input::{
    PHAdjustmentData, PHContentEditingInput, PHContentEditingInputInfo,
    PHContentEditingInputRequestOptions,
};
pub use content_editing_output::{PHContentEditingOutput, PHContentEditingOutputInfo};
pub use error::{
    NSErrorInfo, PHAuthorizationStatus, PHLocalIdentifiersErrorKey, PHPhotosError,
    PHPhotosErrorDomain, PhotoKitError,
};
pub use fetch_options::{PHFetchOptions, PHSortDescriptor};
pub use fetch_result::PHFetchResult;
pub use fetch_result_change_details::{PHFetchResultChangeDetails, PHFetchResultMove};
pub use image_manager::{
    PHCachingImageManager, PHImageContentMode, PHImageDataRequestHandle, PHImageDataResult,
    PHImageErrorKey, PHImageManager, PHImageManagerMaximumSize, PHImageRequest,
    PHImageRequestHandle, PHImageRequestOptionsDeliveryMode, PHImageRequestOptionsResizeMode,
    PHImageRequestOptionsVersion, PHImageResult, PHImageResultRequestIDKey, PHImageSize,
    PHLivePhotoRequestHandle, PHVideoRequestOptions, PHVideoRequestOptionsDeliveryMode,
    PHVideoRequestOptionsVersion, PHVideoResult,
};
pub use live_photo::{PHLivePhoto, PHLivePhotoInfoErrorKey, PHLivePhotoResult};
pub use live_photo_editing_context::{
    PHLivePhotoEditingContext, PHLivePhotoEditingContextInfo, PHLivePhotoEditingSaveResult,
    PHLivePhotoFrame, PHLivePhotoFrameProcessingDecision, PHLivePhotoFrameType,
};
pub use object::{PHObject, PHObjectPlaceholder};
pub use object_change_details::PHObjectChangeDetails;
pub use persistent_change::{
    PHObjectType, PHPersistentChange, PHPersistentChangeFetchResult, PHPersistentChangeToken,
    PHPersistentObjectChangeDetails,
};
pub use photo_library::{
    PHAccessLevel, PHAvailabilityObserver, PHChangeObserver, PHPhotoLibrary,
    PHPhotoLibraryAvailabilityChange, PHPhotoLibraryChange,
};
pub use project::{PHProject, PHProjectChangeRequest};

/// Common imports.
pub mod prelude {
    pub use crate::asset::{
        PHAsset, PHAssetBurstSelectionType, PHAssetEditOperation, PHAssetMediaSubtype,
        PHAssetPlaybackStyle, PHAssetResource, PHAssetResourceType, PHAssetSourceType,
        PHCoordinate, PHMediaType,
    };
    pub use crate::asset_change_request::PHAssetChangeRequest;
    pub use crate::asset_collection::{
        PHAssetCollection, PHAssetCollectionSubtype, PHAssetCollectionType,
        PHCollectionEditOperation,
    };
    pub use crate::asset_collection_change_request::{
        PHAssetCollectionAssetMutation, PHAssetCollectionChangeRequest,
    };
    pub use crate::asset_creation_request::{
        PHAssetCreationRequest, PHAssetCreationResource, PHAssetResourceCreationOptions,
    };
    pub use crate::asset_resource_manager::{
        PHAssetResourceDataResult, PHAssetResourceManager, PHAssetResourceRequestOptions,
        PHAssetResourceWriteResult,
    };
    pub use crate::change::PHChange;
    pub use crate::change_request::PHChangeRequest;
    pub use crate::cloud_identifier::{
        PHCloudIdentifier, PHCloudIdentifierMapping, PHLocalIdentifierMapping,
    };
    pub use crate::collection::PHCollection;
    pub use crate::collection_list::{
        PHCollectionList, PHCollectionListSubtype, PHCollectionListType,
    };
    pub use crate::collection_list_change_request::{
        PHCollectionListChangeRequest, PHCollectionListChildMutation,
    };
    pub use crate::content_editing_input::{
        PHAdjustmentData, PHContentEditingInput, PHContentEditingInputInfo,
        PHContentEditingInputRequestOptions,
    };
    pub use crate::content_editing_output::{PHContentEditingOutput, PHContentEditingOutputInfo};
    pub use crate::error::{
        NSErrorInfo, PHAuthorizationStatus, PHLocalIdentifiersErrorKey, PHPhotosError,
        PHPhotosErrorDomain, PhotoKitError,
    };
    pub use crate::fetch_options::{PHFetchOptions, PHSortDescriptor};
    pub use crate::fetch_result::PHFetchResult;
    pub use crate::fetch_result_change_details::{PHFetchResultChangeDetails, PHFetchResultMove};
    pub use crate::image_manager::{
        PHCachingImageManager, PHImageContentMode, PHImageDataRequestHandle, PHImageDataResult,
        PHImageErrorKey, PHImageManager, PHImageManagerMaximumSize, PHImageRequest,
        PHImageRequestHandle, PHImageRequestOptionsDeliveryMode, PHImageRequestOptionsResizeMode,
        PHImageRequestOptionsVersion, PHImageResult, PHImageResultRequestIDKey, PHImageSize,
        PHLivePhotoRequestHandle, PHVideoRequestOptions, PHVideoRequestOptionsDeliveryMode,
        PHVideoRequestOptionsVersion, PHVideoResult,
    };
    pub use crate::live_photo::{PHLivePhoto, PHLivePhotoInfoErrorKey, PHLivePhotoResult};
    pub use crate::live_photo_editing_context::{
        PHLivePhotoEditingContext, PHLivePhotoEditingContextInfo, PHLivePhotoEditingSaveResult,
        PHLivePhotoFrame, PHLivePhotoFrameProcessingDecision, PHLivePhotoFrameType,
    };
    pub use crate::object::{PHObject, PHObjectPlaceholder};
    pub use crate::object_change_details::PHObjectChangeDetails;
    pub use crate::persistent_change::{
        PHObjectType, PHPersistentChange, PHPersistentChangeFetchResult, PHPersistentChangeToken,
        PHPersistentObjectChangeDetails,
    };
    pub use crate::photo_library::{
        PHAccessLevel, PHAvailabilityObserver, PHChangeObserver, PHPhotoLibrary,
        PHPhotoLibraryAvailabilityChange, PHPhotoLibraryChange,
    };
    pub use crate::project::{PHProject, PHProjectChangeRequest};
}
