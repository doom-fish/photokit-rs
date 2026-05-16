use photokit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let status = PHPhotoLibrary::authorization_status();
    println!("photos authorization: {status:?}");

    if status.is_authorized() {
        let library = PHPhotoLibrary::shared()?;
        let collections = library.fetch_asset_collections(&PHFetchOptions::default())?;
        println!("asset collections: {}", collections.len());
    } else {
        println!("asset collections: skipped (authorization not granted)");
    }

    println!("✅ photokit OK");
    Ok(())
}
