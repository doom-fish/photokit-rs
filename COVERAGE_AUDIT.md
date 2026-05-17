# photokit-rs coverage audit (vs MacOSX26.2.sdk)

SDK_PUBLIC_SYMBOLS: 84
VERIFIED: 81
GAPS: 0
EXEMPT: 3
COVERAGE_PCT: 100.0%

This audit is **symbol-level**, not per-method: a type counts as verified when the crate exposes that Photos.framework symbol (or an idiomatic Rust wrapper for it) through its public API. Per the audit rubric, iOS-only/macOS-unavailable symbols were filtered out, and plain request-id/block typedefs (for example `PHImageRequestID`) were not counted because the rubric only scores interfaces, protocols, enums/structs, exported constants, and top-level functions.

Filtered-out unavailable families include `PHAssetResourceUploadJob*`, `PHLivePhotoShouldRenderAtPlaybackTime`, the `PHLivePhoto (NSItemProvider)` category, and other non-macOS-only declarations.

## 🟢 VERIFIED
| Symbol | Kind | Header | Wrapped by |
| --- | --- | --- | --- |
| PHAccessLevel | enum | PHPhotoLibrary.h | `PHAccessLevel` |
| PHAdjustmentData | interface | PHAdjustmentData.h | `PHAdjustmentData` |
| PHAsset | interface | PHAsset.h | `PHAsset` |
| PHAssetBurstSelectionType | enum | PhotosTypes.h | `PHAssetBurstSelectionType` |
| PHAssetChangeRequest | interface | PHAssetChangeRequest.h | `PHAssetChangeRequest` |
| PHAssetCollection | interface | PHCollection.h | `PHAssetCollection` |
| PHAssetCollectionChangeRequest | interface | PHAssetCollectionChangeRequest.h | `PHAssetCollectionChangeRequest` |
| PHAssetCollectionSubtype | enum | PhotosTypes.h | `PHAssetCollectionSubtype` |
| PHAssetCollectionType | enum | PhotosTypes.h | `PHAssetCollectionType` |
| PHAssetCreationRequest | interface | PHAssetCreationRequest.h | `PHAssetCreationRequest` |
| PHAssetEditOperation | enum | PhotosTypes.h | `PHAssetEditOperation` |
| PHAssetMediaSubtype | enum | PhotosTypes.h | `PHAssetMediaSubtype` |
| PHAssetMediaType | enum | PhotosTypes.h | `PHMediaType` |
| PHAssetPlaybackStyle | enum | PhotosTypes.h | `PHAssetPlaybackStyle` |
| PHAssetResource | interface | PHAssetResource.h | `PHAssetResource` |
| PHAssetResourceCreationOptions | interface | PHAssetCreationRequest.h | `PHAssetResourceCreationOptions` |
| PHAssetResourceManager | interface | PHAssetResourceManager.h | `PHAssetResourceManager` |
| PHAssetResourceRequestOptions | interface | PHAssetResourceManager.h | `PHAssetResourceRequestOptions` |
| PHAssetResourceType | enum | PhotosTypes.h | `PHAssetResourceType` |
| PHAssetSourceType | enum | PhotosTypes.h | `PHAssetSourceType` |
| PHAuthorizationStatus | enum | PHPhotoLibrary.h | `PHAuthorizationStatus` |
| PHCachingImageManager | interface | PHImageManager.h | `PHCachingImageManager` |
| PHChange | interface | PHChange.h | `PHChange` |
| PHChangeRequest | interface | PHChangeRequest.h | `PHChangeRequest` |
| PHCloudIdentifier | interface | PHCloudIdentifier.h | `PHCloudIdentifier` |
| PHCloudIdentifierMapping | interface | PHCloudIdentifier.h | `PHCloudIdentifierMapping` |
| PHCollection | interface | PHCollection.h | `PHCollection` |
| PHCollectionEditOperation | enum | PhotosTypes.h | `PHCollectionEditOperation` |
| PHCollectionList | interface | PHCollection.h | `PHCollectionList` |
| PHCollectionListChangeRequest | interface | PHCollectionListChangeRequest.h | `PHCollectionListChangeRequest` |
| PHCollectionListSubtype | enum | PhotosTypes.h | `PHCollectionListSubtype` |
| PHCollectionListType | enum | PhotosTypes.h | `PHCollectionListType` |
| PHContentEditingInput | interface | PHContentEditingInput.h | `PHContentEditingInput` |
| PHContentEditingInputCancelledKey | const | PHAssetChangeRequest.h | `PHAsset::request_content_editing_input` cancelled-error mapping |
| PHContentEditingInputErrorKey | const | PHAssetChangeRequest.h | `PHAsset::request_content_editing_input` framework-error mapping |
| PHContentEditingInputRequestOptions | interface | PHAssetChangeRequest.h | `PHContentEditingInputRequestOptions` |
| PHContentEditingInputResultIsInCloudKey | const | PHAssetChangeRequest.h | `PHAsset::request_content_editing_input` iCloud-error mapping |
| PHContentEditingOutput | interface | PHContentEditingOutput.h | `PHContentEditingOutput`, `PHContentEditingOutputInfo` |
| PHFetchOptions | interface | PHFetchOptions.h | `PHFetchOptions` |
| PHFetchResult | interface | PHFetchResult.h | `PHFetchResult<T>` |
| PHFetchResultChangeDetails | interface | PHChange.h | `PHFetchResultChangeDetails` |
| PHImageCancelledKey | const | PHImageManager.h | `PHImageResult.cancelled`, `PHImageDataResult.cancelled`, `PHLivePhotoResult.cancelled` |
| PHImageContentMode | enum | PhotosTypes.h | `PHImageContentMode` |
| PHImageErrorKey | const | PHImageManager.h | `PHImageErrorKey`, request-result `error` fields |
| PHImageManager | interface | PHImageManager.h | `PHImageManager` |
| PHImageManagerMaximumSize | const | PHImageManager.h | `PHImageManagerMaximumSize`, `PHImageRequest::maximum` |
| PHImageRequestOptions | interface | PHImageManager.h | `PHImageRequest` |
| PHImageRequestOptionsDeliveryMode | enum | PHImageManager.h | `PHImageRequestOptionsDeliveryMode` |
| PHImageRequestOptionsResizeMode | enum | PHImageManager.h | `PHImageRequestOptionsResizeMode` |
| PHImageRequestOptionsVersion | enum | PHImageManager.h | `PHImageRequestOptionsVersion` |
| PHImageResultIsDegradedKey | const | PHImageManager.h | `PHImageResult.degraded`, `PHImageDataResult.degraded`, `PHLivePhotoResult.degraded` |
| PHImageResultIsInCloudKey | const | PHImageManager.h | `PHImageDataResult.is_in_cloud` |
| PHImageResultRequestIDKey | const | PHImageManager.h | `PHImageResultRequestIDKey`, request-result `request_id` fields |
| PHLivePhoto | interface | PHLivePhoto.h | `PHLivePhoto` |
| PHLivePhotoEditingContext | interface | PHLivePhotoEditingContext.h | `PHLivePhotoEditingContext` |
| PHLivePhotoFrame | protocol | PHLivePhotoEditingContext.h | `PHLivePhotoFrame` |
| PHLivePhotoFrameType | enum | PHLivePhotoEditingContext.h | `PHLivePhotoFrameType` |
| PHLivePhotoInfoCancelledKey | const | PHLivePhoto.h | `PHLivePhotoResult.cancelled` |
| PHLivePhotoInfoErrorKey | const | PHLivePhoto.h | `PHLivePhotoInfoErrorKey`, `PHLivePhotoResult.error` |
| PHLivePhotoInfoIsDegradedKey | const | PHLivePhoto.h | `PHLivePhotoResult.degraded` |
| PHLivePhotoRequestOptions | interface | PHImageManager.h | `PHImageRequest` + `PHImageManager::request_live_photo` |
| PHLocalIdentifierMapping | interface | PHCloudIdentifier.h | `PHLocalIdentifierMapping` |
| PHLocalIdentifiersErrorKey | const | PHError.h | `PHLocalIdentifiersErrorKey`, `NSErrorInfo.local_identifiers` |
| PHObject | interface | PHObject.h | `PHObject` |
| PHObjectChangeDetails | interface | PHChange.h | `PHObjectChangeDetails<T>` |
| PHObjectPlaceholder | interface | PHObject.h | `PHObjectPlaceholder` |
| PHObjectType | enum | PhotosTypes.h | `PHObjectType` |
| PHPersistentChange | interface | PHPersistentChange.h | `PHPersistentChange` |
| PHPersistentChangeFetchResult | interface | PHPersistentChangeFetchResult.h | `PHPersistentChangeFetchResult` |
| PHPersistentChangeToken | interface | PHPersistentChangeToken.h | `PHPersistentChangeToken` |
| PHPersistentObjectChangeDetails | interface | PHPersistentObjectChangeDetails.h | `PHPersistentObjectChangeDetails` |
| PHPhotoLibrary | interface | PHPhotoLibrary.h | `PHPhotoLibrary` |
| PHPhotoLibraryAvailabilityObserver | protocol | PHPhotoLibrary.h | `PHAvailabilityObserver` + `PHPhotoLibrary::register_availability_observer` |
| PHPhotoLibraryChangeObserver | protocol | PHPhotoLibrary.h | `PHChangeObserver` + `PHPhotoLibrary::{register_change_observer, register_detailed_change_observer}` |
| PHPhotosError | enum | PHError.h | `PHPhotosError` |
| PHPhotosErrorDomain | const | PHError.h | `PHPhotosErrorDomain` |
| PHProject | interface | PHProject.h | `PHProject` |
| PHProjectChangeRequest | interface | PHProjectChangeRequest.h | `PHProjectChangeRequest` |
| PHVideoRequestOptions | interface | PHImageManager.h | `PHVideoRequestOptions` |
| PHVideoRequestOptionsDeliveryMode | enum | PHImageManager.h | `PHVideoRequestOptionsDeliveryMode` |
| PHVideoRequestOptionsVersion | enum | PHImageManager.h | `PHVideoRequestOptionsVersion` |

## 🔴 GAPS

None.

## ⏭️ EXEMPT
| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
| PHLivePhotoEditingErrorCode | enum | PHLivePhotoEditingContext.h | Deprecated live-photo editing error codes; modern callers should use `PHPhotosError` / `NSError`. | Per-case `API_DEPRECATED_WITH_REPLACEMENT(...)` on macOS 10.12–10.15 |
| PHLivePhotoEditingErrorDomain | const | PHLivePhotoEditingContext.h | Deprecated live-photo editing error domain replaced by `PHPhotosErrorDomain`. | `API_DEPRECATED_WITH_REPLACEMENT("PHPhotosErrorDomain", macos(10.12, 10.15))` |
| PHLocalIdentifierNotFound | const | PHCloudIdentifier.h | Deprecated cloud/local-id sentinel replaced by `PHLocalIdentifierMapping.error`. | `API_DEPRECATED_WITH_REPLACEMENT("Check for PHPhotosErrorIdentifierNotFound in PHLocalIdentifierMapping.error", macos(10.13, 12))` |
