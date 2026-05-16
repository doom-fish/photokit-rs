mod common;

use photokit::prelude::*;

#[test]
fn cloud_identifier_constructor_sets_string_value() {
    let identifier = PHCloudIdentifier::new("sample-cloud-id");
    assert_eq!(identifier.string_value, "sample-cloud-id");
}

#[test]
fn cloud_identifier_mapping_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let Some(library) = common::authorized_library() else {
        return Ok(());
    };
    let Some(asset) = common::first_asset() else {
        return Ok(());
    };

    let mappings = library.cloud_identifier_mappings_for_local_identifiers(std::slice::from_ref(&asset.local_identifier))?;
    assert!(mappings.contains_key(&asset.local_identifier));
    Ok(())
}
