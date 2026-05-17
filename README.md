# photokit

Safe Rust bindings for Apple's [Photos](https://developer.apple.com/documentation/photos) framework on macOS.

> **Status:** v0.2.1 closes the symbol-level Photos.framework audit at 100% coverage, adding `PHAssetResourceManager`, generic `PHCollection`/`PHChangeRequest` mutation APIs, persistent change history, `PHLivePhotoEditingContext`, video requests, project wrappers, and typed Photos constants/errors.

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

## Highlights

- `PHPhotoLibrary` authorization helpers plus summary/detailed change observers, availability observers, and persistent-change history helpers.
- `PHAsset`, `PHCollection`, `PHAssetCollection`, and `PHCollectionList` fetch helpers with typed subtype/source/resource wrappers.
- `PHChangeRequest` builders for asset, album, folder, and project mutation flows.
- `PHImageManager` / `PHCachingImageManager` request handles for images, image data, live photos, video requests, and caching.
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

Run them with:

```bash
for ex in examples/*.rs; do
  cargo run --example "$(basename "$ex" .rs)"
done
```

## Verification

```bash
cargo clippy --all-targets -- -D warnings
cargo test
for ex in examples/*.rs; do cargo run --example "$(basename "$ex" .rs)"; done
```

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
