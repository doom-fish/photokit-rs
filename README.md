# photokit

Safe Rust bindings for Apple's [Photos](https://developer.apple.com/documentation/photos) framework on macOS.

> **Status:** v0.1.0 covers practical `PHPhotoLibrary`, `PHAsset`, `PHFetchResult`, `PHAssetCollection`, `PHFetchOptions`, `PHImageManager`, `PHCachingImageManager`, `PHAssetResource`, change observers, and cancellable image/data requests.

## Quick start

```rust,no_run
use photokit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let library = PHPhotoLibrary::shared()?;
    println!("status: {:?}", PHPhotoLibrary::authorization_status());

    let collections = library.fetch_asset_collections(&PHFetchOptions::default())?;
    println!("collections: {}", collections.len());

    let assets = PHPhotoLibrary::fetch_assets(&PHFetchOptions::default())?;
    if let Some(asset) = assets.first() {
        let manager = PHImageManager::shared()?;
        let request = manager.request_image(
            asset,
            PHImageRequest::new(320.0, 240.0, PHImageContentMode::AspectFit),
        )?;
        let image = request.wait(5_000)?;
        println!("image bytes: {}", image.tiff_data().len());
    }

    Ok(())
}
```

## Highlights

- `PHPhotoLibrary::authorization_status`, `request_authorization`, `register_change_observer`
- `PHFetchOptions` with predicate, sort descriptors, and fetch limits
- `PHAsset` snapshots for dates, size, location, media type/subtypes, duration, and favorite state
- `PHImageManager` / `PHCachingImageManager` with cancellable request handles for image, image-data, and basic live-photo lookups
- `PHAssetResource::for_asset` for original filename / resource metadata
- `PHFetchResult<T>` convenience wrapper with `len`, `iter`, and `first`

## Authorization

The smoke example never prompts for Photos access. It reports the current authorization state and only performs non-interactive library queries.

## Smoke example

Run the framework smoke test with:

```bash
cargo run --all-features --example 01_photokit_smoke
```

Expected success footer:

```text
✅ photokit OK
```

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
