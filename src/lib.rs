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
pub mod asset_change_request;
pub mod asset_collection;
pub mod asset_collection_change_request;
pub mod asset_creation_request;
pub mod asset_resource_manager;
pub mod change;
pub mod change_request;
pub mod cloud_identifier;
pub mod collection;
pub mod collection_list;
pub mod collection_list_change_request;
pub mod content_editing_input;
pub mod content_editing_output;
pub mod error;
pub mod fetch_options;
pub mod fetch_result;
pub mod fetch_result_change_details;
mod ffi;
pub mod image_manager;
pub mod library;
pub mod live_photo;
pub mod live_photo_editing_context;
pub mod object;
pub mod object_change_details;
pub mod persistent_change;
pub mod photo_library;
mod private;
pub mod project;
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
