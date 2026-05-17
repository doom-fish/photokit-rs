# Changelog

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
