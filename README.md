# photokit

Safe Rust bindings for Apple's [Photos](https://developer.apple.com/documentation/photos) framework on macOS.

> **Status:** 0.5.0 is a soundness release. Change requests return errors instead of aborting the process, callback contexts outlive in-flight PhotoKit calls, image requests complete on PhotoKit's final result, and fetch predicates can no longer read undefined memory. See the [CHANGELOG](CHANGELOG.md) for the breaking changes.

## Installation

```toml
[dependencies]
photokit = "0.5"
```

## Requirements

- macOS 13 or later. A few properties need macOS 14 or 26; the bridge checks availability at runtime and reports them as absent or returns an error on older systems.
- Xcode or the Swift toolchain: `build.rs` builds the Swift bridge with `swift build`.
- Photos access. Apps need `NSPhotoLibraryUsageDescription` (and `NSPhotoLibraryAddUsageDescription` for add-only access) in `Info.plist`; sandboxed apps also need the `com.apple.security.personal-information.photos-library` entitlement. Command-line tools get the system prompt attributed to the terminal that runs them. `request_authorization` shows that prompt; the blocking variant waits up to 30 seconds for an answer and the async one until the user responds.

## Threading

- Image and image data requests (`request_image`, `request_image_data` and their async versions) run as synchronous PhotoKit requests on a background queue, so they complete on any thread without a main run loop. `cancel()` returns the cancelled result at once: a queued request is skipped and an iCloud download stops, while a decode that has already started finishes in the background.
- PhotoKit has no synchronous form of `request_live_photo`, `PHLivePhoto::request_with_resource_file_urls` or `request_content_editing_input` and delivers their results on the main queue. A blocking `wait()` on the main thread runs the main run loop while it waits. From any other thread, the main thread must be running a run loop (an AppKit app, or `CFRunLoopRun`); otherwise the wait times out.
- Library change and availability observers run on a PhotoKit background queue, so their callbacks must be `Send + Sync`. Once the observer is dropped, its callback is not called again.
- A timeout of `u64::MAX` milliseconds waits indefinitely.
- `PHLivePhotoView` and `PHPickerViewController` are AppKit objects: create them on the main thread (their wrappers are `!Send`); `new()` returns an error on any other thread. Their delegate callbacks run on the main thread and don't need to be `Send`.
- A live photo frame processor either keeps each frame's original image or aborts the edit. `PHLivePhotoFrameProcessingDecision::Abort`, a panic in the processor, or a processor cleared during a render makes the prepare or save fail with an error.
- Fetch predicates are NSPredicate format strings parsed without arguments. Format specifiers such as `%@`, malformed predicates, and keys PhotoKit doesn't support return an error.

## Quick start

```rust,no_run
use photokit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let library = PHPhotoLibrary::shared()?;
    println!("status: {:?}", PHPhotoLibrary::authorization_status());

    let assets = PHAsset::fetch(&PHFetchOptions::default())?;
    println!("assets: {}", assets.len());

    if let Some(asset) = assets.first() {
        let manager = PHImageManager::shared()?;
        let request = manager.request_image_data(
            asset,
            &PHImageRequest::new(320.0, 240.0, PHImageContentMode::AspectFit),
        )?;
        let image = request.wait(10_000)?;
        println!("image bytes: {}", image.data().len());
    }

    let _ = library.fetch_asset_collections(&PHFetchOptions::default())?;
    Ok(())
}
```

## Async API

Enable the `async` feature to access non-blocking wrappers for Photos.framework completion-handler APIs:

```toml
[dependencies]
photokit = { version = "0.5", features = ["async"] }
```

```rust
# #[cfg(feature = "async")]
# async fn demo() -> Result<(), Box<dyn std::error::Error>> {
use photokit::async_api::AsyncPHPhotoLibrary;
use photokit::PHAccessLevel;

let status = AsyncPHPhotoLibrary::request_authorization(PHAccessLevel::ReadWrite).await?;
# Ok(())
# }
```

All async types implement `std::future::Future` and are executor-agnostic.
See [`async_api`](src/async_api.rs) for the full API surface.

## Highlights

- `PHPhotoLibrary` authorization helpers plus summary/detailed change observers, availability observers, and persistent-change history helpers.
- `PHAsset`, `PHCollection`, `PHAssetCollection`, and `PHCollectionList` fetch helpers with typed subtype/source/resource wrappers.
- `PHChangeRequest` builders for asset, album, folder, and project mutation flows.
- `PHImageManager` request handles for images, image data, live photos and video requests, and `PHCachingImageManager` start/stop caching. Requests through a caching manager are not bridged, so nothing in this crate reads its cache yet.
- `PHAssetResourceManager` transfer helpers for reading or writing asset resources.
- `PHContentEditingInput` / `PHContentEditingOutput` plus `PHLivePhotoEditingContext` for non-destructive editing workflows.
- `PHCloudIdentifier` batch lookup helpers and richer Photos-specific error metadata.

## Coverage audit

See [`COVERAGE.md`](COVERAGE.md) for the framework audit, implemented rows, partial rows, and deferred macOS-unavailable or deprecated APIs.

## Examples

The crate ships with numbered examples covering every logical area:

- `01_photokit_smoke`
- `02_phasset_fetch`
- `03_phasset_collection_fetch`
- `04_phcollection_list_fetch`
- `05_phphoto_library_authorization`
- `06_phimage_manager_requests`
- `07_phfetch_result_methods`
- `08_phchange_observer`
- `09_phcontent_editing_input`
- `10_phcontent_editing_output`
- `11_phobject_change_details`
- `12_phfetch_options_builder`
- `13_phasset_creation_request`
- `14_phlive_photo`
- `15_phcloud_identifier`
- `16_async_api` *(requires `--features async`)*

Run them with:

```bash
for ex in examples/*.rs; do
  name="$(basename "$ex" .rs)"
  if [ "$name" = "16_async_api" ]; then
    cargo run --example "$name" --features async
  else
    cargo run --example "$name"
  fi
done
```

## Verification

```bash
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
for ex in examples/*.rs; do
  name="$(basename "$ex" .rs)"
  if [ "$name" = "16_async_api" ]; then
    cargo run --example "$name" --features async
  else
    cargo run --example "$name"
  fi
done
```

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
