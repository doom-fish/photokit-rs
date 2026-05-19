# Changelog

## [0.4.1] - 2026-05-19

- Bump MSRV from 1.70 to 1.76 to match fleet baseline.

## 0.4.0 - 2026-05-19

- Added PhotosUI / project-extension wrappers for `PHPickerConfiguration`, `PHPickerFilter`, `PHPickerResult`, `PHPickerViewController`, `PHPickerViewControllerDelegate`, `PHLivePhotoView`, `PHLivePhotoViewDelegate`, `PHContentEditingController`, `PHProjectInfo`, `PHProjectElement`, `PHProjectAssetElement`, `PHProjectRegionOfInterest`, `PHProjectSection`, `PHProjectSectionContent`, `PHProjectTextElement`, `PHProjectJournalEntryElement`, `PHProjectMapElement`, `PHProjectTypeDescription`, `PHProjectTypeDescriptionDataSource`, `PHProjectTypeDescriptionInvalidator`, `PHProjectExtensionContext`, and `PHProjectExtensionController`.
- Added PhotosUI model/runtime smoke coverage plus a main-thread `17_photosui_runtime_smoke` example for the AppKit-backed picker and live-photo view wrappers.
- Accepted both `requestId` and `requestID` when deserializing request-backed Swift bridge payloads.

## 0.3.4 - 2026-05-18

- Added rustdoc coverage across the public API, including modules, types, variants, fields, and methods with Photos framework counterpart references.
- Reached 100.0% rustdoc item coverage for the crate.

## 0.3.3 - 2026-05-18

- Re-exported `JsonCallback` from `doom-fish-utils::ffi_callbacks` and removed the duplicate local FFI typedef.

## 0.3.2 - 2026-06-10

- **Async/unsafe audit (quality pass)**
  - Added `catch_user_panic` guards (from `doom-fish-utils::panic_safe`) to all
    three `extern "C"` trampolines that invoke user-supplied closures
    (`change_observer_trampoline`, `availability_observer_trampoline`,
    `live_photo_frame_processor_trampoline`).  A panic crossing the FFI
    boundary into Swift is undefined behaviour; it is now caught and logged.
  - Added `// SAFETY:` doc comments to every `Box::from_raw`,
    `Box::into_raw`, and `NonNull::new_unchecked` call site in
    `photo_library.rs` and `live_photo_editing_context.rs`.
  - Widened the `doom-fish-utils` version constraint from `"0.1"` to
    `">=0.1, <0.3"` to permit the next minor release without a breaking
    Cargo.toml edit.

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
