# Changelog

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
