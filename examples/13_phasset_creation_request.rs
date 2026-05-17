use photokit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let supported = PHAssetCreationRequest::supports_asset_resource_types(&[
        PHAssetResourceType::PHOTO,
        PHAssetResourceType::PAIRED_VIDEO,
    ])?;
    println!("photo + paired video supported: {supported}");

    let request = PHAssetCreationRequest::new().add_data_resource_bytes(1, b"example", None);
    println!("resource count: {}", request.resources.len());
    Ok(())
}
