mod common;

use photokit::prelude::*;

#[test]
fn asset_live_photo_helper_checks_media_subtype_flag() {
    let asset = PHAsset {
        local_identifier: "asset".to_owned(),
        creation_date: None,
        modification_date: None,
        added_date: None,
        pixel_width: 1,
        pixel_height: 1,
        location: None,
        media_type: PHMediaType::Image,
        media_subtypes: 1 << 3,
        duration: 0.0,
        is_hidden: false,
        is_favorite: false,
        playback_style: Some(PHAssetPlaybackStyle::LivePhoto),
        content_type_identifier: None,
        burst_identifier: None,
        burst_selection_types: 0,
        represents_burst: false,
        source_type: 0,
        has_adjustments: false,
        adjustment_format_identifier: None,
    };

    assert!(asset.is_live_photo());
}

#[test]
fn asset_fetch_smoke() -> Result<(), Box<dyn std::error::Error>> {
    if !PHPhotoLibrary::authorization_status().is_authorized() {
        return Ok(());
    }

    let assets = PHAsset::fetch(&PHFetchOptions::default().with_fetch_limit(1))?;
    if let Some(asset) = assets.first() {
        let _ = asset.can_perform_edit_operation(PHAssetEditOperation::Properties)?;
        let _ = asset.resources()?;
    }
    Ok(())
}
