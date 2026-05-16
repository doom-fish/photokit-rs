# photokit-rs coverage audit (vs MacOSX26.2.sdk)

SDK_PUBLIC_SYMBOLS: 84
VERIFIED: 44
GAPS: 37
EXEMPT: 3
COVERAGE_PCT: 54.3%

This audit is **symbol-level**, not per-method: a type counts as verified when the crate exposes that Photos.framework symbol (or an idiomatic Rust wrapper for it) through its public API. Per the audit rubric, iOS-only/macOS-unavailable symbols were filtered out, and plain request-id/block typedefs (for example `PHImageRequestID`) were not counted because the rubric only scores interfaces, protocols, enums/structs, exported constants, and top-level functions.

Filtered-out unavailable families include `PHAssetResourceUploadJob*`, `PHLivePhotoShouldRenderAtPlaybackTime`, the `PHLivePhoto (NSItemProvider)` category, and other non-macOS-only declarations.

## 🟢 VERIFIED
| Symbol | Kind | Header | Wrapped by |
| --- | --- | --- | --- |
| PHAccessLevel | enum | PHPhotoLibrary.h | `PHAccessLevel` |
| PHAdjustmentData | interface | PHAdjustmentData.h | `PHAdjustmentData` |
| PHAsset | interface | PHAsset.h | `PHAsset` |
| PHAssetCollection | interface | PHCollection.h | `PHAssetCollection` |
| PHAssetCollectionType | enum | PhotosTypes.h | `PHAssetCollectionType` |
| PHAssetCreationRequest | interface | PHAssetCreationRequest.h | `PHAssetCreationRequest` |
| PHAssetEditOperation | enum | PhotosTypes.h | `PHAssetEditOperation` |
| PHAssetMediaType | enum | PhotosTypes.h | `PHMediaType` |
| PHAssetPlaybackStyle | enum | PhotosTypes.h | `PHAssetPlaybackStyle` |
| PHAssetResource | interface | PHAssetResource.h | `PHAssetResource` |
| PHAssetResourceCreationOptions | interface | PHAssetCreationRequest.h | `PHAssetResourceCreationOptions` |
| PHAuthorizationStatus | enum | PHPhotoLibrary.h | `PHAuthorizationStatus` |
| PHCachingImageManager | interface | PHImageManager.h | `PHCachingImageManager` |
| PHChange | interface | PHChange.h | `PHChange` |
| PHCloudIdentifier | interface | PHCloudIdentifier.h | `PHCloudIdentifier` |
| PHCloudIdentifierMapping | interface | PHCloudIdentifier.h | `PHCloudIdentifierMapping` |
| PHCollectionEditOperation | enum | PhotosTypes.h | `PHCollectionEditOperation` |
| PHCollectionList | interface | PHCollection.h | `PHCollectionList` |
| PHCollectionListType | enum | PhotosTypes.h | `PHCollectionListType` |
| PHContentEditingInput | interface | PHContentEditingInput.h | `PHContentEditingInput` |
| PHContentEditingInputCancelledKey | const | PHAssetChangeRequest.h | `PHAsset::request_content_editing_input` cancelled-error mapping |
| PHContentEditingInputErrorKey | const | PHAssetChangeRequest.h | `PHAsset::request_content_editing_input` framework-error mapping |
| PHContentEditingInputRequestOptions | interface | PHAssetChangeRequest.h | `PHContentEditingInputRequestOptions` |
| PHContentEditingInputResultIsInCloudKey | const | PHAssetChangeRequest.h | `PHAsset::request_content_editing_input` iCloud-error mapping |
| PHContentEditingOutput | interface | PHContentEditingOutput.h | `PHContentEditingOutput`, `PHContentEditingOutputInfo` |
| PHFetchOptions | interface | PHFetchOptions.h | `PHFetchOptions` |
| PHFetchResult | interface | PHFetchResult.h | `PHFetchResult<T>` |
| PHImageCancelledKey | const | PHImageManager.h | `PHImageResult.cancelled`, `PHImageDataResult.cancelled`, `PHLivePhotoResult.cancelled` |
| PHImageContentMode | enum | PhotosTypes.h | `PHImageContentMode` |
| PHImageManager | interface | PHImageManager.h | `PHImageManager` |
| PHImageRequestOptions | interface | PHImageManager.h | `PHImageRequest` |
| PHImageRequestOptionsDeliveryMode | enum | PHImageManager.h | `PHImageRequestOptionsDeliveryMode` |
| PHImageRequestOptionsResizeMode | enum | PHImageManager.h | `PHImageRequestOptionsResizeMode` |
| PHImageRequestOptionsVersion | enum | PHImageManager.h | `PHImageRequestOptionsVersion` |
| PHImageResultIsDegradedKey | const | PHImageManager.h | `PHImageResult.degraded`, `PHImageDataResult.degraded`, `PHLivePhotoResult.degraded` |
| PHImageResultIsInCloudKey | const | PHImageManager.h | `PHImageDataResult.is_in_cloud` |
| PHLivePhoto | interface | PHLivePhoto.h | `PHLivePhoto` |
| PHLivePhotoInfoCancelledKey | const | PHLivePhoto.h | `PHLivePhotoResult.cancelled` |
| PHLivePhotoInfoIsDegradedKey | const | PHLivePhoto.h | `PHLivePhotoResult.degraded` |
| PHLivePhotoRequestOptions | interface | PHImageManager.h | `PHImageRequest` + `PHImageManager::request_live_photo` |
| PHLocalIdentifierMapping | interface | PHCloudIdentifier.h | `PHLocalIdentifierMapping` |
| PHObjectChangeDetails | interface | PHChange.h | `PHObjectChangeDetails<T>` |
| PHPhotoLibrary | interface | PHPhotoLibrary.h | `PHPhotoLibrary` |
| PHPhotoLibraryChangeObserver | protocol | PHPhotoLibrary.h | `PHChangeObserver` + `PHPhotoLibrary::{register_change_observer, register_detailed_change_observer}` |

## 🔴 GAPS
| Symbol | Kind | Header | Notes |
| --- | --- | --- | --- |
| PHAssetBurstSelectionType | enum | PhotosTypes.h | Exposed only as raw `u64` on `PHAsset.burst_selection_types`. |
| PHAssetChangeRequest | interface | PHAssetChangeRequest.h | No general asset mutation/delete/revert wrapper beyond `PHAssetCreationRequest`. |
| PHAssetCollectionChangeRequest | interface | PHAssetCollectionChangeRequest.h | Album create/update/reorder APIs are not wrapped. |
| PHAssetCollectionSubtype | enum | PhotosTypes.h | Exposed only as raw `i64` on `PHAssetCollection.collection_subtype`. |
| PHAssetMediaSubtype | enum | PhotosTypes.h | Exposed only as raw `u64` on `PHAsset.media_subtypes`. |
| PHAssetResourceManager | interface | PHAssetResourceManager.h | No asset-resource download/read/write bridge. |
| PHAssetResourceRequestOptions | interface | PHAssetResourceManager.h | No options type for asset-resource transfers. |
| PHAssetResourceType | enum | PhotosTypes.h | Exposed only as raw `i64` on `PHAssetResource.resource_type` and `PHAssetCreationResource.resource_type`. |
| PHAssetSourceType | enum | PhotosTypes.h | Exposed only as raw `u64` on `PHAsset.source_type` / `PHFetchOptions.include_asset_source_types`. |
| PHChangeRequest | interface | PHChangeRequest.h | No generic change-request surface. |
| PHCollection | interface | PHCollection.h | No generic collection wrapper or top-level collection fetch APIs. |
| PHCollectionListChangeRequest | interface | PHCollectionListChangeRequest.h | Folder create/update/reorder APIs are not wrapped. |
| PHCollectionListSubtype | enum | PhotosTypes.h | Exposed only as raw `i64` on `PHCollectionList.collection_list_subtype`. |
| PHFetchResultChangeDetails | interface | PHChange.h | No fetch-result diff/change-details wrapper. |
| PHImageErrorKey | const | PHImageManager.h | Request-result `NSError` values are not surfaced in the public result structs. |
| PHImageManagerMaximumSize | const | PHImageManager.h | No exported sentinel for original/full-size image requests. |
| PHImageResultRequestIDKey | const | PHImageManager.h | Request handles hide the request-id info dictionary key. |
| PHLivePhotoEditingContext | interface | PHLivePhotoEditingContext.h | No live-photo editing bridge. |
| PHLivePhotoFrame | protocol | PHLivePhotoEditingContext.h | No frame-processing surface. |
| PHLivePhotoFrameType | enum | PHLivePhotoEditingContext.h | No live-photo editing frame enum wrapper. |
| PHLivePhotoInfoErrorKey | const | PHLivePhoto.h | Live-photo request errors are not surfaced from the info dictionary. |
| PHLocalIdentifiersErrorKey | const | PHError.h | `NSError.userInfo` is reduced to `NSErrorInfo { domain, code, message }`. |
| PHObject | interface | PHObject.h | No shared base wrapper; only concrete snapshot structs expose `local_identifier`. |
| PHObjectPlaceholder | interface | PHObject.h | Creation flows return identifiers, not placeholder objects. |
| PHObjectType | enum | PhotosTypes.h | Persistent-change APIs are not wrapped. |
| PHPersistentChange | interface | PHPersistentChange.h | No persistent change-history support. |
| PHPersistentChangeFetchResult | interface | PHPersistentChangeFetchResult.h | No persistent change-history support. |
| PHPersistentChangeToken | interface | PHPersistentChangeToken.h | No persistent change-history support. |
| PHPersistentObjectChangeDetails | interface | PHPersistentObjectChangeDetails.h | No persistent change-history support. |
| PHPhotoLibraryAvailabilityObserver | protocol | PHPhotoLibrary.h | Availability observer APIs are not wrapped. |
| PHPhotosError | enum | PHError.h | Errors surface as generic `NSErrorInfo` / `PhotoKitError`, not a typed Photos error enum. |
| PHPhotosErrorDomain | const | PHError.h | No typed Photos error-domain constant wrapper. |
| PHProject | interface | PHProject.h | Project-extension APIs are not wrapped. |
| PHProjectChangeRequest | interface | PHProjectChangeRequest.h | Project mutation APIs are not wrapped. |
| PHVideoRequestOptions | interface | PHImageManager.h | Video request APIs are not wrapped. |
| PHVideoRequestOptionsDeliveryMode | enum | PHImageManager.h | No video request-options wrapper. |
| PHVideoRequestOptionsVersion | enum | PHImageManager.h | No video request-options wrapper. |

## ⏭️ EXEMPT
| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
| PHLivePhotoEditingErrorCode | enum | PHLivePhotoEditingContext.h | Deprecated live-photo editing error codes; modern callers should use `PHPhotosError` / `NSError`. | Per-case `API_DEPRECATED_WITH_REPLACEMENT(...)` on macOS 10.12–10.15 |
| PHLivePhotoEditingErrorDomain | const | PHLivePhotoEditingContext.h | Deprecated live-photo editing error domain replaced by `PHPhotosErrorDomain`. | `API_DEPRECATED_WITH_REPLACEMENT("PHPhotosErrorDomain", macos(10.12, 10.15))` |
| PHLocalIdentifierNotFound | const | PHCloudIdentifier.h | Deprecated cloud/local-id sentinel replaced by `PHLocalIdentifierMapping.error`. | `API_DEPRECATED_WITH_REPLACEMENT("Check for PHPhotosErrorIdentifierNotFound in PHLocalIdentifierMapping.error", macos(10.13, 12))` |
