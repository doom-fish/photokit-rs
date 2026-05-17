use photokit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !PHPhotoLibrary::authorization_status().is_authorized() {
        println!("photos access not granted; skipping PHAssetCollection example");
        return Ok(());
    }

    let library = PHPhotoLibrary::shared()?;
    let collections = library.fetch_asset_collections(&PHFetchOptions::default())?;
    println!("collections: {}", collections.len());
    if let Some(collection) = collections.first() {
        println!("first collection: {:?}", collection.localized_title);
        println!(
            "assets in first collection: {}",
            collection.assets(&PHFetchOptions::default())?.len()
        );
    }
    Ok(())
}
