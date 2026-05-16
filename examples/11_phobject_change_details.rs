use photokit::prelude::*;

fn main() {
    let asset = PHAsset {
        local_identifier: "example".to_owned(),
        creation_date: None,
        modification_date: None,
        added_date: None,
        pixel_width: 1,
        pixel_height: 1,
        location: None,
        media_type: PHMediaType::Image,
        media_subtypes: 0,
        duration: 0.0,
        is_hidden: false,
        is_favorite: false,
        playback_style: None,
        content_type_identifier: None,
        burst_identifier: None,
        burst_selection_types: 0,
        represents_burst: false,
        source_type: 0,
        has_adjustments: false,
        adjustment_format_identifier: None,
    };
    let details = PHObjectChangeDetails {
        object_before_changes: asset.clone(),
        object_after_changes: Some(asset),
        asset_content_changed: false,
        object_was_deleted: false,
    };
    println!("object change deleted? {}", details.object_was_deleted);
}
