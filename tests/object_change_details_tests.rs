use photokit::prelude::*;

#[test]
fn object_change_details_round_trip_generic_payload() {
    let before = PHAsset {
        local_identifier: "before".to_owned(),
        creation_date: None,
        modification_date: None,
        added_date: None,
        pixel_width: 1,
        pixel_height: 1,
        location: None,
        media_type: PHMediaType::Image,
        media_subtypes: PHAssetMediaSubtype::NONE,
        duration: 0.0,
        is_hidden: false,
        is_favorite: false,
        playback_style: None,
        content_type_identifier: None,
        burst_identifier: None,
        burst_selection_types: PHAssetBurstSelectionType::NONE,
        represents_burst: false,
        source_type: PHAssetSourceType::NONE,
        has_adjustments: false,
        adjustment_format_identifier: None,
    };
    let details = PHObjectChangeDetails {
        object_before_changes: before.clone(),
        object_after_changes: Some(before),
        asset_content_changed: true,
        object_was_deleted: false,
    };

    assert!(details.asset_content_changed);
    assert!(!details.object_was_deleted);
    assert!(details.object_after_changes.is_some());
}
