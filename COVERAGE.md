# Photos.framework coverage (photokit v0.2.1)

Legend:

- ✅ implemented
- 🟡 partial
- ⏭️ skipped

| Area | API row | Status | Notes |
| --- | --- | --- | --- |
| PHAsset | snapshot of supported macOS properties (`localIdentifier`, dates, size, location, media metadata, playback style, burst flags, source type, adjustment metadata) | ✅ | Bridged via `PHAsset.swift` + `src/asset.rs` with typed media/burst/source/resource wrappers. |
| PHAsset | `canPerform(_:)` | ✅ | Exposed as `PHAsset::can_perform_edit_operation`. |
| PHAsset | `fetchAssets(with:)`, `fetchAssets(with:options:)`, `fetchAssets(withLocalIdentifiers:options:)`, `fetchAssets(in:options:)`, `fetchKeyAssets(in:options:)` | ✅ | Covered by `PHAsset::fetch*` helpers. |
| PHAsset | `fetchAssetsWithBurstIdentifier:options:` | 🟡 | Burst fetch-by-identifier is still not wrapped. |
| PHAsset | `fetchAssetsWithALAssetURLs:options:` | ⏭️ | macOS unavailable legacy API. |
| PHAssetChangeRequest | generic asset create/update/delete/revert flows | ✅ | Covered by `PHAssetChangeRequest` and `PHAssetCreationRequest`. |
| PHAssetCollection | snapshot of collection properties (`localizedTitle`, type/subtype, estimated count, date/location metadata, containment flags) | ✅ | Bridged via `PHAssetCollection.swift` + `src/asset_collection.rs`. |
| PHAssetCollection | `fetchAssetCollections(withLocalIdentifiers:options:)` | ✅ | Exposed as `fetch_with_local_identifiers`. |
| PHAssetCollection | `fetchAssetCollections(with:subtype:options:)` | ✅ | Exposed as `fetch_with_type`. |
| PHAssetCollection | `fetchAssetCollectionsContainingAsset(_:with:options:)` | ✅ | Exposed as `fetch_containing_asset`. |
| PHAssetCollection | transient asset collections | 🟡 | Transient collection constructors are not yet bridged. |
| PHAssetCollection | `fetchAssetCollectionsWithALAssetGroupURLs:options:` | ⏭️ | Deprecated legacy importer path. |
| PHAssetCollectionChangeRequest | album create/update/reorder/remove operations | ✅ | Covered by `PHAssetCollectionChangeRequest`. |
| PHCollection | top-level collection fetches and edit-capability checks | ✅ | Covered by `PHCollection` plus in-list/top-level fetch helpers. |
| PHCollectionList | snapshot of list properties (`localizedTitle`, type/subtype, dates, location names, containment flags) | ✅ | Bridged via `PHCollectionList.swift` + `src/collection_list.rs`. |
| PHCollectionList | `fetchCollectionListsContainingCollection:options:` | ✅ | Exposed as `fetch_containing_collection_local_identifier`. |
| PHCollectionList | `fetchCollectionListsWithLocalIdentifiers:options:` | ✅ | Exposed as `fetch_with_local_identifiers`. |
| PHCollectionList | `fetchCollectionListsWithType:subtype:options:` | ✅ | Exposed as `fetch_with_type`. |
| PHCollectionList | transient collection lists | 🟡 | Transient list constructors are not yet bridged. |
| PHCollectionList | deprecated moment-list APIs | ⏭️ | Deprecated / macOS unavailable. |
| PHCollectionListChangeRequest | folder/top-level collection mutation flows | ✅ | Covered by `PHCollectionListChangeRequest`. |
| PHPhotoLibrary | shared library handle | ✅ | `PHPhotoLibrary::shared`. |
| PHPhotoLibrary | authorization status + request authorization | ✅ | Read/write + access-level-aware coverage via `PHAccessLevel`. |
| PHPhotoLibrary | change observer registration / unregistration | ✅ | Summary and detailed observer registrations are covered. |
| PHPhotoLibrary | availability observer APIs | ✅ | Covered by `PHPhotoLibrary::register_availability_observer`. |
| PHPhotoLibrary | `performChangesAndWait` mutation flows | ✅ | Used by asset/album/folder/project change-request wrappers. |
| PHPhotoLibrary | persistent change history (`fetchPersistentChangesSinceToken`, `currentChangeToken`) | ✅ | Covered by `PHPersistentChange*` wrappers on `PHPhotoLibrary`. |
| PHImageManager | request image | ✅ | `PHImageManager::request_image`. |
| PHImageManager | request image data + orientation | ✅ | `PHImageManager::request_image_data`. |
| PHImageManager | request live photo | ✅ | `PHImageManager::request_live_photo`. |
| PHImageManager | video requests (`requestPlayerItem`, `requestExportSession`, `requestAVAsset`) | ✅ | Covered by `PHVideoRequestOptions` and the three synchronous video request helpers. |
| PHImageManager | cancel image requests | ✅ | Shared request-handle cancellation path. |
| PHAssetResourceManager | request data / write data for asset resources | ✅ | Covered by `PHAssetResourceManager` + `PHAssetResourceRequestOptions`. |
| PHCachingImageManager | start/stop caching + stop all | ✅ | Covered by `PHCachingImageManager`. |
| PHFetchResult | `count`, `firstObject`, `lastObject`, indexed access, iteration | ✅ | Exposed on generic `PHFetchResult<T>`. |
| PHFetchResult | `containsObject`, `indexOfObject`, `objectsAtIndexes`, `countOfAssetsWithMediaType` | ✅ | Exposed on the Rust convenience wrapper. |
| PHChange | `changeDetails(for:)` object lookups | ✅ | Asset, asset-collection, and collection-list details are bridged. |
| PHChange | `changeDetails(forFetchResult:)` | ✅ | Covered by the idiomatic `PHFetchResultChangeDetails` diff helper. |
| PHObject / PHObjectChangeDetails | shared object wrapper plus before/after/delete/content-change details | ✅ | Covered by `PHObject`, `PHObjectPlaceholder`, and `PHObjectChangeDetails<T>`. |
| PHContentEditingInput | `requestContentEditingInput` + timeout cancel path | ✅ | Bridged synchronously into an owned input handle. |
| PHContentEditingInput | snapshot of supported macOS properties | ✅ | Includes adjustment data, image URLs, orientation, AVAsset class, live-photo size. |
| PHContentEditingOutput | `init(contentEditingInput:)` | ✅ | Exposed as `PHContentEditingInput::create_content_editing_output`. |
| PHContentEditingOutput | `adjustmentData`, `renderedContentURL` | ✅ | Getter/setter and snapshot support are bridged. |
| PHContentEditingOutput | `defaultRenderedContentType`, `supportedRenderedContentTypes`, `renderedContentURLForType` | ✅ | Available on supported macOS versions. |
| PHContentEditingOutput | `init(placeholderForCreatedAsset:)` | 🟡 | Deferred. |
| PHFetchOptions | predicate, sort descriptors, hidden/burst/source-type filters, fetch limit, incremental change flag | ✅ | Fully bridged, including typed source-type option sets. |
| PHAssetCreationRequest | `creationRequestForAsset`, `supportsAssetResourceTypes:` | ✅ | Bridged via synchronous perform-changes helper. |
| PHAssetCreationRequest | `addResource(with:fileURL:options:)`, `addResource(with:data:options:)` | ✅ | File + in-memory resource creation are supported. |
| PHAssetResourceCreationOptions | `originalFilename`, `uniformTypeIdentifier`, `contentType`, `shouldMoveFile` | ✅ | Bridged on supported macOS versions. |
| PHLivePhoto | request from resource file URLs | ✅ | Bridged with shared request-handle wait/cancel support. |
| PHLivePhoto | cancel live-photo request | ✅ | Covered via `PHLivePhoto::request_with_resource_file_urls` request handle. |
| PHLivePhoto | `size` | ✅ | Surfaced on `PHLivePhotoResult::live_photo()`. |
| PHLivePhotoEditingContext | create context, process frames, prepare playback, save, cancel | ✅ | Covered by `PHLivePhotoEditingContext`, `PHLivePhotoFrame`, and `PHLivePhotoFrameType`. |
| PHCloudIdentifier | `stringValue`, `init(stringValue:)` | ✅ | Bridged as value type wrapper. |
| PHCloudIdentifier | bulk cloud/local identifier lookup | 🟡 | Bridged via the deprecated batch lookup APIs on macOS; mapping-error detail objects are still not surfaced. |
| PHProject | fetch and inspect project collections | ✅ | Covered by `PHProject` and `PHCollection::is_project`. |
| PHProjectChangeRequest | mutate project title/data/preview/assets | ✅ | Covered by `PHProjectChangeRequest`. |
| Errors/constants | Photos error-domain constants and request info keys | ✅ | Covered by `PHPhotosError`, `PHPhotosErrorDomain`, `PHLocalIdentifiersErrorKey`, `PHImageErrorKey`, `PHImageResultRequestIDKey`, and `PHLivePhotoInfoErrorKey`. |

## Deferred families

| Header family | Status | Reason |
| --- | --- | --- |
| `PHAssetResourceUploadJob*` | ⏭️ | iOS-only upload-job APIs are unavailable on macOS. |
