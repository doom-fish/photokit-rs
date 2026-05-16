use photokit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !PHPhotoLibrary::authorization_status().is_authorized() {
        println!("photos access not granted; skipping PHAsset example");
        return Ok(());
    }

    let assets = PHAsset::fetch(&PHFetchOptions::default())?;
    println!("assets: {}", assets.len());
    if let Some(asset) = assets.first() {
        println!("first asset: {}", asset.local_identifier);
        println!("resources: {}", asset.resources()?.len());
    }
    Ok(())
}
