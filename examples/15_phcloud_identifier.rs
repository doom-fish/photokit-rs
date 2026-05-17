use photokit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sample = PHCloudIdentifier::new("example-cloud-identifier");
    println!("sample cloud identifier: {}", sample.string_value);

    if !PHPhotoLibrary::authorization_status().is_authorized() {
        println!("photos access not granted; skipping PHCloudIdentifier lookup");
        return Ok(());
    }

    let library = PHPhotoLibrary::shared()?;
    let assets = PHAsset::fetch(&PHFetchOptions::default().with_fetch_limit(1))?;
    let Some(asset) = assets.first() else {
        println!("no assets available");
        return Ok(());
    };

    let mappings = library.cloud_identifier_mappings_for_local_identifiers(
        std::slice::from_ref(&asset.local_identifier),
    )?;
    println!("mapping entries: {}", mappings.len());
    Ok(())
}
