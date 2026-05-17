use photokit::prelude::*;

#[test]
fn fetch_result_helpers_cover_common_patterns() {
    let result = PHFetchResult::from(vec![1, 2, 3, 2]);

    assert_eq!(result.len(), 4);
    assert!(!result.is_empty());
    assert_eq!(result.first(), Some(&1));
    assert_eq!(result.last(), Some(&2));
    assert_eq!(result.get(2), Some(&3));
    assert!(result.contains(&2));
    assert_eq!(result.index_of(&3), Some(2));
    assert_eq!(result.objects_at_indexes(&[0, 3]), vec![1, 2]);
}

#[test]
fn fetch_result_can_count_assets_by_media_type() {
    let result = PHFetchResult::from(vec![
        PHAsset {
            local_identifier: "a".to_owned(),
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
        },
        PHAsset {
            local_identifier: "b".to_owned(),
            creation_date: None,
            modification_date: None,
            added_date: None,
            pixel_width: 1,
            pixel_height: 1,
            location: None,
            media_type: PHMediaType::Video,
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
        },
    ]);

    assert_eq!(
        result.count_of_assets_with_media_type(PHMediaType::Image),
        1
    );
    assert_eq!(
        result.count_of_assets_with_media_type(PHMediaType::Video),
        1
    );
}
