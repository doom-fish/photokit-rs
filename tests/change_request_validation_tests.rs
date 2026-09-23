use photokit::prelude::*;

const ONE_PIXEL_PNG_BASE64: &str =
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";

fn error_message(error: &PhotoKitError) -> String {
    match error {
        PhotoKitError::Framework(info) => info.message.clone(),
        other => other.to_string(),
    }
}

fn assert_rejected<T: std::fmt::Debug>(result: Result<T, PhotoKitError>, expected: &str) {
    let error = result.expect_err("request should be rejected before reaching Photos");
    let message = error_message(&error);
    assert!(
        message.contains(expected),
        "expected an error containing {expected:?}, got {message:?}"
    );
}

#[test]
fn asset_change_rejects_invalid_base64_image_data() {
    let request = PHAssetChangeRequest {
        create_image_data_base64: Some("not base64 ***".to_owned()),
        ..PHAssetChangeRequest::default()
    };
    assert_rejected(request.perform(), "not valid base64");
}

#[test]
fn asset_change_rejects_image_data_that_is_not_an_image() {
    let request = PHAssetChangeRequest::creation_request_for_asset_from_image_data(b"plain text");
    assert_rejected(request.perform(), "not a decodable image");
}

#[test]
fn asset_change_rejects_unreadable_image_file() {
    let request = PHAssetChangeRequest::creation_request_for_asset_from_image_file_url(
        "/nonexistent/photokit-missing-image.jpg",
    );
    assert_rejected(request.perform(), "not readable");
}

#[test]
fn asset_change_rejects_malformed_file_url() {
    let request =
        PHAssetChangeRequest::creation_request_for_asset_from_video_file_url("file://%zz");
    assert_rejected(request.perform(), "invalid file URL");
}

#[test]
fn asset_change_rejects_request_without_target() {
    assert_rejected(
        PHAssetChangeRequest::default().perform(),
        "needs an asset identifier or a creation source",
    );
}

#[test]
fn asset_change_rejects_invalid_creation_date() {
    let request = PHAssetChangeRequest {
        create_image_data_base64: Some(ONE_PIXEL_PNG_BASE64.to_owned()),
        ..PHAssetChangeRequest::default()
    }
    .set_creation_date("yesterday");
    assert_rejected(request.perform(), "invalid ISO 8601 date");
}

#[test]
fn asset_collection_change_rejects_unknown_mutation_kind() {
    let mut request = PHAssetCollectionChangeRequest::creation_request_for_asset_collection("x");
    request
        .asset_mutations
        .push(PHAssetCollectionAssetMutation {
            kind: "shuffle".to_owned(),
            asset_local_identifiers: Vec::new(),
            indexes: Vec::new(),
            to_index: None,
        });
    assert_rejected(request.perform(), "unsupported mutation kind");
}

#[test]
fn asset_collection_change_rejects_out_of_range_indexes() {
    let request = PHAssetCollectionChangeRequest::creation_request_for_asset_collection("x")
        .remove_assets_at_indexes(&[0]);
    assert_rejected(request.perform(), "outside 0..<0");
}

#[test]
fn asset_collection_change_rejects_mismatched_insert_indexes() {
    let request = PHAssetCollectionChangeRequest::creation_request_for_asset_collection("x")
        .insert_assets(&[], &[0]);
    assert_rejected(request.perform(), "exactly one index per object");
}

#[test]
fn asset_collection_change_rejects_move_without_destination() {
    let mut request = PHAssetCollectionChangeRequest::creation_request_for_asset_collection("x");
    request
        .asset_mutations
        .push(PHAssetCollectionAssetMutation {
            kind: "move".to_owned(),
            asset_local_identifiers: Vec::new(),
            indexes: Vec::new(),
            to_index: None,
        });
    assert_rejected(request.perform(), "move destination index");
}

#[test]
fn asset_collection_change_rejects_index_mutation_after_add() {
    let request = PHAssetCollectionChangeRequest::creation_request_for_asset_collection("x")
        .add_assets(&[])
        .remove_assets_at_indexes(&[]);
    assert_rejected(request.perform(), "cannot follow add or remove");
}

#[test]
fn asset_collection_change_rejects_request_without_target() {
    assert_rejected(
        PHAssetCollectionChangeRequest::default().perform(),
        "needs a creation title or a collection identifier",
    );
}

#[test]
fn collection_list_change_rejects_out_of_range_move() {
    let request = PHCollectionListChangeRequest::creation_request_for_collection_list("x")
        .move_child_collections_at_indexes(&[], 1);
    assert_rejected(request.perform(), "move destination index is outside 0...0");
}

#[test]
fn collection_list_change_rejects_mismatched_replace_indexes() {
    let request = PHCollectionListChangeRequest::creation_request_for_collection_list("x")
        .replace_child_collections_at_indexes(&[0, 1], &[]);
    assert_rejected(request.perform(), "exactly one index per object");
}

#[test]
fn asset_creation_rejects_invalid_base64_resource() {
    let request = PHAssetCreationRequest {
        resources: vec![PHAssetCreationResource {
            resource_type: PHAssetResourceType::PHOTO,
            file_url: None,
            data_base64: Some("***".to_owned()),
            options: None,
        }],
    };
    assert_rejected(request.perform(), "not valid base64");
}

#[test]
fn asset_creation_rejects_unreadable_file_resource() {
    let request = PHAssetCreationRequest::new().add_file_resource(
        PHAssetResourceType::PHOTO,
        "/nonexistent/photokit-missing-resource.heic",
        None,
    );
    assert_rejected(request.perform(), "not readable");
}

#[test]
fn asset_creation_rejects_resource_without_source() {
    let request = PHAssetCreationRequest {
        resources: vec![PHAssetCreationResource {
            resource_type: PHAssetResourceType::PHOTO,
            file_url: None,
            data_base64: None,
            options: None,
        }],
    };
    assert_rejected(request.perform(), "needs a file URL or data");
}

#[cfg(feature = "async")]
#[test]
fn async_change_requests_report_validation_errors() -> Result<(), PhotoKitError> {
    use photokit::async_api::{AsyncPHAssetChangeRequest, AsyncPHAssetCollectionChangeRequest};

    let asset_request = PHAssetChangeRequest {
        create_image_data_base64: Some("***".to_owned()),
        ..PHAssetChangeRequest::default()
    };
    assert_rejected(
        pollster::block_on(AsyncPHAssetChangeRequest::perform(&asset_request)?),
        "not valid base64",
    );

    let collection_request =
        PHAssetCollectionChangeRequest::creation_request_for_asset_collection("x")
            .move_assets_at_indexes(&[3], 0);
    assert_rejected(
        pollster::block_on(AsyncPHAssetCollectionChangeRequest::perform(
            &collection_request,
        )?),
        "outside 0..<0",
    );
    Ok(())
}
