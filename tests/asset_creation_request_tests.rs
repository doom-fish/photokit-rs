use photokit::prelude::*;

#[test]
fn asset_creation_request_builder_round_trips_resources() {
    let request = PHAssetCreationRequest::new()
        .add_file_resource(1, "example.jpg", None)
        .add_data_resource_bytes(1, b"hello", Some(PHAssetResourceCreationOptions::default()));

    assert_eq!(request.resources.len(), 2);
    assert!(!request.is_empty());
    assert_eq!(request.resources[0].resource_type, 1);
    assert_eq!(request.resources[1].options.as_ref().map(|options| options.should_move_file), Some(false));
}

#[test]
fn asset_creation_request_supports_simple_resource_sets() -> Result<(), Box<dyn std::error::Error>> {
    let supported = PHAssetCreationRequest::supports_asset_resource_types(&[1, 9])?;
    assert!(supported);
    Ok(())
}
