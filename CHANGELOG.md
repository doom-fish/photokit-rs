# Changelog

All notable changes to `photokit` are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - Unreleased

### Security

- `PHChangeObserver` and `PHAvailabilityObserver` no longer free their callback
  while a notification is in flight: each Swift observer holds its own
  reference to the callback context until it is deallocated. Up to 0.4.5,
  dropping an observer during delivery could use freed memory.
- The live photo frame processor stays alive for as long as PhotoKit can call
  it. Clearing or dropping it while a prepare or save was rendering (PhotoKit
  renders with a copy of the processor block) could call freed memory.
- Fetch predicates are parsed without variadic arguments. A format specifier
  such as `%@` in `PHFetchOptions::with_predicate` read a nonexistent argument
  (undefined behaviour).
- The synchronous live photo save no longer writes the caller's error pointer
  from the completion handler, which after a timeout wrote into a dead stack
  frame.
- `PHLivePhotoView` and `PHPickerViewController` delegates keep their callback
  alive while it runs. A callback that dropped its own registration freed the
  closure it was executing; the Swift delegate now holds a reference to the
  callback context until it is deallocated.

### Fixed

- Change requests (`PHAssetChangeRequest`, `PHAssetCollectionChangeRequest`,
  `PHCollectionListChangeRequest`, `PHAssetCreationRequest`,
  `PHProjectChangeRequest`, sync and async) validate local identifiers, file
  URLs, base64 data, creation dates and mutation indexes before calling
  PhotoKit, and return errors instead of aborting the process through `try!`,
  `fatalError` or an Objective-C exception.
- Malformed fetch predicates and unsupported predicate or sort keys return an
  error instead of aborting the process.
- Creating assets from image or video file URLs, file-backed creation
  resources and project preview images work. The URL fields were serialized
  as `createImageFileUrl`, `fileUrl` and so on, while the bridge decodes
  `createImageFileURL` and `fileURL`, so they never reached PhotoKit.
- `PHContentEditingOutput` snapshots and `write_data_for_asset_resource`
  results parse again, and `PHContentEditingInputInfo::full_size_image_url`
  and `PHVideoResult::asset_url` are populated (`renderedContentURL`,
  `fileURL`, `fullSizeImageURL` and `assetURL` keys).
- `PHLivePhotoEditingContext::new` and live photo saves passed the retained
  bridge boxes to PhotoKit as if they were `PHContentEditingInput` and
  `PHContentEditingOutput` objects.
- Async `request_image` completes with `fastFormat` and `highQualityFormat`;
  it waited for a non-degraded result that PhotoKit never sends, and leaked
  the future's context. Results report the real `degraded` flag.
- Synchronous image and live photo request handles return the final result in
  opportunistic mode instead of the degraded preview, and their completion
  state is lock-protected against a concurrent `cancel()`.
- Timeouts above `i64::MAX` milliseconds (such as `u64::MAX` meaning "wait
  forever") wait indefinitely instead of crashing the process.
- Waits on the main thread run the main run loop, so PhotoKit's main-queue
  deliveries arrive instead of timing out.
- `request_data_for_asset_resource` streams the bytes into the result instead
  of building base64 JSON copies, which peaked at four to five times the
  resource size.
- `write_data_for_asset_resource` cancels the transfer and removes the partial
  file when it times out; it used to return an error while PhotoKit kept
  writing.
- The JSON snapshots of content editing inputs and outputs return an error
  instead of aborting when a value cannot be encoded, and integer conversions
  of framework values no longer trap.
- The async authorization tests skip instead of blocking on a Photos prompt
  when the authorization status is undetermined.
- Image and image data requests, sync and async, complete without the main
  thread running a run loop. They run as synchronous PhotoKit requests on a
  bounded background queue; before, a `wait()` off the main thread timed out
  and an awaited future never resolved unless the main thread ran a run loop.
- Library change and availability callbacks that arrive after their observer
  was dropped are not run, and a `PHChange` delivered then is released.
- A live photo frame processor that aborts now fails the edit with an error;
  `SkipFrame` returned nil without one. A processor that panics, was cleared
  during a render, or receives a frame it cannot decode aborts the edit instead
  of keeping the original image for that frame.

### Changed

- **Breaking:** `PHAssetResourceDataResult` carries the bytes as
  `data: Vec<u8>` instead of `data_base64: String`.
- **Breaking:** `SaveLivePhotoFuture` and `PrepareLivePhotoFuture` borrow the
  editing context (and output) they were created from.
- **Breaking:** `PHLivePhotoView::new` and `PHPickerViewController::new`
  return an error when called off the main thread.
- **Breaking:** `register_change_observer`,
  `register_detailed_change_observer` and `register_availability_observer`
  require `Fn + Send + Sync` callbacks.
- **Breaking:** `PHLivePhotoFrameProcessingDecision::SkipFrame` is renamed to
  `Abort`, which is what PhotoKit does with it: the whole live photo edit fails.
- **Breaking:** cancelling an image or image data request returns the
  cancelled result at once but no longer interrupts a decode that has already
  started; queued requests are skipped and iCloud downloads stop.
- The `register_delegate` callbacks of `PHLivePhotoView` and
  `PHPickerViewController` no longer need to be `Send`; they always run on the
  main thread.
- A failed synchronous `save_live_photo_to_output` returns an error instead of
  `Ok` with `success: false`.
- Index-based collection mutations after `add` or `remove` in the same change
  request are rejected, because their range cannot be checked.
- `write_data_for_asset_resource` reports an error in its result when the
  destination file already exists, like `writeData`.
- URL fields serialize under the bridge's key names (`fileURL`,
  `createImageFileURL`, ...); the previous camelCase spellings are still
  accepted when deserializing.
- Requires `doom-fish-utils` 0.4.1. `rust-version` is now 1.82.

### Removed

- **Breaking:** `PHAssetResourceDataResult::data()`; read the `data` field.
- **Breaking:** `PHImageRequest::synchronous` and
  `PHImageRequest::allow_secondary_degraded_image`. Every image request runs
  synchronously on a background queue, and only the final result was ever
  returned.

## [0.4.5] - 2026-05-20

- Migrated local `take_string` body to call `doom_fish_utils::ffi_string::take_owned_cstring_c`. Centralises the duplicated FFI take-string pattern fleet-wide. No public API change.

## [0.4.4] - 2026-05-20

- Added in-`src/` unit tests across src/error.rs, src/fetch_options.rs, and src/geometry.rs (Tier 2 quality polish), providing fast `cargo test --lib` fail-fast signal alongside the existing integration tests under `tests/`.

## [0.4.3] - 2026-05-20

- Clippy hygiene sweep: cleared all `-D warnings` lints across the crate. No public API change.

## [0.4.2] - 2026-05-20

- Widen `doom-fish-utils` dependency bound to `<0.4` so the 0.3.x SPSC-ring release resolves cleanly. No source changes.

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
