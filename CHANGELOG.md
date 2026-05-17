# Changelog

## 0.3.1 - 2026-06-10

- Availability sweep: audited all Photos.framework symbols against the macOS 26
  SDK headers and confirmed `@available`/`#available` guards are in place for
  every macOS 26-only property used in the Swift bridge:
  - `PHAsset.addedDate` (macOS 26) — guarded in `pkrEncodeAsset`
  - `PHAsset.contentType` (macOS 26) — guarded in `pkrEncodeAsset`
  - `PHAssetResource.contentType` (macOS 26) — guarded in `pkrEncodeResource`
  - `PHContentEditingInput.contentType` (macOS 26) — guarded in
    `pkrEncodeContentEditingInput`
  - `PHAssetResourceCreationOptions.contentType` (macOS 26) — guarded in
    `pkrAssetResourceCreationOptions`
- Replaced Objective-C runtime selector probe for `PHAssetResource.pixelWidth`
  / `pixelHeight` (macOS 13+) with idiomatic `if #available(macOS 13.0, *)`
  guards.

## 0.3.0 - 2026-05-17

- Added `async_api` module (gated behind `async` feature) — Tier-1 async wrappers for:
  - `PHPhotoLibrary.requestAuthorization(for:handler:)` → `AsyncPHPhotoLibrary::request_authorization`
  - `PHPhotoLibrary.performChanges(_:completionHandler:)` → `AsyncPHPhotoLibrary::perform_asset_change` / `perform_collection_change` / `perform_collection_list_change`
  - `PHAssetChangeRequest`, `PHAssetCollectionChangeRequest`, `PHCollectionListChangeRequest` — async wrappers via `AsyncPHPhotoLibrary`
  - `PHImageManager.requestImage(for:)` → `AsyncPHImageManager::request_image` (one-shot Future, final delivery)
  - `PHImageManager.requestImageDataAndOrientation(for:)` → `AsyncPHImageManager::request_image_data`
  - `PHLivePhotoEditingContext.saveLivePhoto(to:options:completionHandler:)` → `AsyncPHLivePhotoEditingContext::save_live_photo`
  - `PHLivePhotoEditingContext.prepareLivePhotoForPlayback(withTargetSize:)` → `AsyncPHLivePhotoEditingContext::prepare_live_photo`
- All futures are executor-agnostic (no tokio/async-std dependency).
- Added `doom-fish-utils` dependency for `AsyncCompletion`/`AsyncCompletionFuture`.
- Added 1 async example (`16_async_api`) and async API integration tests.

## 0.2.1 - 2026-05-17

- Closed the remaining symbol-level Photos.framework audit gaps and reached 100% audited coverage on macOS.
- Added typed wrappers for Photos enums/constants/errors including asset/media/resource/source subtypes, `PHPhotosError`, and request-info keys.
- Added safe wrappers for `PHAssetResourceManager`, `PHCollection`, `PHChangeRequest`, `PHAssetChangeRequest`, `PHAssetCollectionChangeRequest`, `PHCollectionListChangeRequest`, `PHProject`, and `PHProjectChangeRequest`.
- Added persistent change history, availability observer support, video request APIs, and `PHLivePhotoEditingContext` frame-processing/playback helpers.
- Added exhaustive smoke tests for the new surfaces and refreshed the coverage/release documentation for v0.2.1.

## 0.2.0 - 2026-05-16

- Split the Swift bridge into per-area files and reorganized the Rust API into area modules.
- Added safe wrappers for `PHCollectionList`, `PHChange`, `PHContentEditingInput`, `PHContentEditingOutput`, `PHObjectChangeDetails`, `PHAssetCreationRequest`, and `PHCloudIdentifier`.
- Expanded `PHAsset`, `PHAssetCollection`, `PHPhotoLibrary`, `PHImageManager`, `PHFetchOptions`, `PHFetchResult`, and `PHLivePhoto` coverage.
- Added numbered examples and per-area smoke tests for every requested logical area.
- Added `COVERAGE.md` documenting implemented, partial, and deferred Photos.framework rows.

## 0.1.0 - 2026-05-16

- Initial release.
- Added safe Rust bindings for `PHPhotoLibrary`, `PHAsset`, `PHFetchResult`, `PHAssetCollection`, `PHFetchOptions`, `PHImageManager`, `PHCachingImageManager`, and `PHAssetResource`.
- Added cancellable request handles for image, image-data, and basic live-photo lookups.
- Added a non-interactive smoke example that reports authorization and counts visible asset collections.
