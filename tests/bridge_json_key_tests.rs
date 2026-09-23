use photokit::prelude::*;
use serde_json::json;

#[test]
fn file_url_requests_use_the_keys_the_bridge_decodes() {
    let image = serde_json::to_value(
        PHAssetChangeRequest::creation_request_for_asset_from_image_file_url("/photos/a.jpg"),
    )
    .unwrap();
    assert_eq!(image["createImageFileURL"], "/photos/a.jpg");
    assert!(image.get("createImageFileUrl").is_none());

    let video = serde_json::to_value(
        PHAssetChangeRequest::creation_request_for_asset_from_video_file_url("/photos/a.mov"),
    )
    .unwrap();
    assert_eq!(video["createVideoFileURL"], "/photos/a.mov");

    let creation = serde_json::to_value(PHAssetCreationRequest::new().add_file_resource(
        PHAssetResourceType::PHOTO,
        "/photos/a.heic",
        None,
    ))
    .unwrap();
    assert_eq!(creation["resources"][0]["fileURL"], "/photos/a.heic");

    let project = serde_json::to_value(PHProjectChangeRequest {
        project_preview_image_file_url: Some("/photos/preview.png".to_owned()),
        ..PHProjectChangeRequest::default()
    })
    .unwrap();
    assert_eq!(project["projectPreviewImageFileURL"], "/photos/preview.png");
}

#[test]
fn bridge_results_with_url_keys_deserialize() {
    let output: PHContentEditingOutputInfo = serde_json::from_value(json!({
        "renderedContentURL": "/edits/out.jpg",
        "supportedRenderedContentTypeIdentifiers": []
    }))
    .unwrap();
    assert_eq!(output.rendered_content_url, "/edits/out.jpg");

    let write: PHAssetResourceWriteResult = serde_json::from_value(json!({
        "fileURL": "file:///exports/original.heic",
        "success": true
    }))
    .unwrap();
    assert_eq!(write.file_url, "file:///exports/original.heic");
    assert!(write.success);

    let video: PHVideoResult = serde_json::from_value(json!({
        "resultType": "avAsset",
        "assetURL": "file:///videos/clip.mov"
    }))
    .unwrap();
    assert_eq!(video.asset_url.as_deref(), Some("file:///videos/clip.mov"));

    let input: PHContentEditingInputInfo = serde_json::from_value(json!({
        "mediaType": "image",
        "mediaSubtypes": 0,
        "fullSizeImageURL": "/edits/in.heic",
        "fullSizeImageOrientation": 1
    }))
    .unwrap();
    assert_eq!(input.full_size_image_url.as_deref(), Some("/edits/in.heic"));
}

#[test]
fn previous_camel_case_url_keys_still_deserialize() {
    let output: PHContentEditingOutputInfo =
        serde_json::from_value(json!({ "renderedContentUrl": "/edits/out.jpg" })).unwrap();
    assert_eq!(output.rendered_content_url, "/edits/out.jpg");

    let write: PHAssetResourceWriteResult =
        serde_json::from_value(json!({ "fileUrl": "file:///x", "success": false })).unwrap();
    assert_eq!(write.file_url, "file:///x");
}

