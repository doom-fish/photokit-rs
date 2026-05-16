use photokit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let supported = PHAssetCreationRequest::supports_asset_resource_types(&[1, 9])?;
    println!("photo + paired video supported: {supported}");

    let request = PHAssetCreationRequest::new().add_data_resource_bytes(1, b"example", None);
    println!("resource count: {}", request.resources.len());
    Ok(())
}
