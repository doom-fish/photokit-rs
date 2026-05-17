use photokit::prelude::*;

const fn accepts_change_request<R: PHChangeRequest>(_request: &R) {}

fn dummy_asset() -> PHAsset {
    PHAsset {
        local_identifier: "asset-1".to_owned(),
        creation_date: None,
        modification_date: None,
        added_date: None,
        pixel_width: 1,
        pixel_height: 1,
        location: None,
        media_type: PHMediaType::Image,
        media_subtypes: PHAssetMediaSubtype::PHOTO_LIVE,
        duration: 0.0,
        is_hidden: false,
        is_favorite: false,
        playback_style: Some(PHAssetPlaybackStyle::LivePhoto),
        content_type_identifier: None,
        burst_identifier: None,
        burst_selection_types: PHAssetBurstSelectionType::NONE,
        represents_burst: false,
        source_type: PHAssetSourceType::USER_LIBRARY,
        has_adjustments: false,
        adjustment_format_identifier: None,
    }
}

fn dummy_collection() -> PHAssetCollection {
    PHAssetCollection {
        local_identifier: "collection-1".to_owned(),
        localized_title: Some("Album".to_owned()),
        collection_type: PHAssetCollectionType::Album,
        collection_subtype: PHAssetCollectionSubtype::ALBUM_REGULAR,
        estimated_asset_count: Some(1),
        start_date: None,
        end_date: None,
        approximate_location: None,
        localized_location_names: Vec::new(),
        can_contain_assets: true,
        can_contain_collections: false,
    }
}

fn dummy_collection_list() -> PHCollectionList {
    PHCollectionList {
        local_identifier: "list-1".to_owned(),
        localized_title: Some("Folder".to_owned()),
        collection_list_type: PHCollectionListType::Folder,
        collection_list_subtype: PHCollectionListSubtype::REGULAR_FOLDER,
        start_date: None,
        end_date: None,
        localized_location_names: Vec::new(),
        can_contain_assets: true,
        can_contain_collections: true,
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn exhaustive_gap_models_compile_and_round_trip() {
    let object = PHObject::new("object-1");
    let placeholder = PHObjectPlaceholder::new("object-1");
    assert_eq!(placeholder.object(), object);

    let media_subtypes = PHAssetMediaSubtype::PHOTO_LIVE | PHAssetMediaSubtype::PHOTO_HDR;
    assert!(media_subtypes.contains(PHAssetMediaSubtype::PHOTO_LIVE));
    assert!(PHAssetBurstSelectionType::AUTO_PICK.contains(PHAssetBurstSelectionType::AUTO_PICK));
    assert_eq!(PHAssetSourceType::CLOUD_SHARED.bits(), 1 << 1);
    assert_eq!(PHAssetResourceType::PAIRED_VIDEO.raw_value(), 9);
    assert_eq!(PHCollectionListSubtype::ANY.raw_value(), i64::MAX);
    assert_eq!(
        PHAssetCollectionSubtype::SMART_ALBUM_SPATIAL.raw_value(),
        219
    );
    assert_eq!(PHObjectType::COLLECTION_LIST.0, 3);
    assert_eq!(PHVideoRequestOptionsVersion::CURRENT.0, 0);
    assert_eq!(PHVideoRequestOptionsDeliveryMode::FAST_FORMAT.0, 3);
    assert_eq!(PHLivePhotoFrameType::VIDEO.0, 1);

    assert_eq!(PHPhotosError::USER_CANCELLED.raw_value(), 3072);
    assert_eq!(PHPhotosErrorDomain, "PHPhotosErrorDomain");
    assert_eq!(PHLocalIdentifiersErrorKey, "PHLocalIdentifiersErrorKey");
    assert_eq!(PHImageErrorKey, "PHImageErrorKey");
    assert_eq!(PHImageResultRequestIDKey, "PHImageResultRequestIDKey");
    assert_eq!(PHLivePhotoInfoErrorKey, "PHLivePhotoInfoErrorKey");
    assert_eq!(
        PHImageManagerMaximumSize.width.to_bits(),
        (-1.0_f64).to_bits()
    );
    assert_eq!(
        PHImageManagerMaximumSize.height.to_bits(),
        (-1.0_f64).to_bits()
    );

    let generic_collection = PHCollection::from(&dummy_collection());
    assert!(generic_collection.is_asset_collection());
    assert_eq!(generic_collection.object().local_identifier, "collection-1");
    let generic_list = PHCollection::from(&dummy_collection_list());
    assert!(generic_list.is_collection_list());

    let project = PHProject {
        asset_collection: dummy_collection(),
        project_extension_data_base64: "aGVsbG8=".to_owned(),
        has_project_preview: false,
    };
    assert_eq!(project.project_extension_data(), b"hello");
    let project_collection = PHCollection::from(&project);
    assert!(project_collection.is_project());

    let before = PHFetchResult::from(vec![PHObject::new("before")]);
    let after = PHFetchResult::from(vec![PHObject::new("after")]);
    let change_details =
        PHFetchResultChangeDetails::change_details_from_fetch_result(&before, &after, &[]);
    assert!(change_details.has_incremental_changes);
    assert_eq!(change_details.removed_indexes, vec![0]);
    assert_eq!(change_details.inserted_indexes, vec![0]);

    let persistent_token = PHPersistentChangeToken {
        data_base64: "aGVsbG8=".to_owned(),
    };
    let object_details = PHPersistentObjectChangeDetails {
        object_type: PHObjectType::ASSET,
        inserted_local_identifiers: vec!["a".to_owned()],
        updated_local_identifiers: vec!["b".to_owned()],
        deleted_local_identifiers: vec!["c".to_owned()],
    };
    let persistent_change = PHPersistentChange {
        change_token: persistent_token,
        change_details: vec![object_details],
    };
    assert!(persistent_change
        .change_details_for_object_type(PHObjectType::ASSET)
        .is_some());
    let persistent_result = PHPersistentChangeFetchResult {
        changes: vec![persistent_change],
    };
    assert_eq!(persistent_result.len(), 1);
    assert!(!persistent_result.is_empty());

    let asset = dummy_asset();
    let asset_change_request = PHAssetChangeRequest::change_request_for_asset(&asset)
        .set_favorite(true)
        .set_hidden(true)
        .clear_creation_date()
        .clear_location()
        .revert_asset_content_to_original();
    accepts_change_request(&asset_change_request);
    let _ = <PHAssetChangeRequest as PHChangeRequest>::perform;
    let _ = PHAssetChangeRequest::delete_assets as fn(&[PHAsset]) -> Result<(), PhotoKitError>;
    let _ = PHAssetChangeRequest::creation_request_for_asset_from_image_file_url("/dev/null");
    let _ = PHAssetChangeRequest::creation_request_for_asset_from_image_data(b"img");
    let _ = PHAssetChangeRequest::creation_request_for_asset_from_video_file_url("/dev/null");

    let collection_request =
        PHAssetCollectionChangeRequest::creation_request_for_asset_collection("New Album")
            .set_title("Updated Album")
            .add_assets(std::slice::from_ref(&asset))
            .insert_assets(std::slice::from_ref(&asset), &[0])
            .remove_assets(std::slice::from_ref(&asset))
            .remove_assets_at_indexes(&[0])
            .replace_assets_at_indexes(&[0], std::slice::from_ref(&asset))
            .move_assets_at_indexes(&[0], 0);
    accepts_change_request(&collection_request);
    let _ = <PHAssetCollectionChangeRequest as PHChangeRequest>::perform;
    let _ = PHAssetCollectionChangeRequest::delete_asset_collections
        as fn(&[PHAssetCollection]) -> Result<(), PhotoKitError>;

    let list_request =
        PHCollectionListChangeRequest::change_request_for_top_level_user_collections()
            .set_title("Updated Folder")
            .add_child_collections(std::slice::from_ref(&generic_collection))
            .insert_child_collections(std::slice::from_ref(&generic_list), &[0])
            .remove_child_collections(std::slice::from_ref(&generic_collection))
            .remove_child_collections_at_indexes(&[0])
            .replace_child_collections_at_indexes(&[0], std::slice::from_ref(&generic_list))
            .move_child_collections_at_indexes(&[0], 0);
    accepts_change_request(&list_request);
    let _ = <PHCollectionListChangeRequest as PHChangeRequest>::perform;
    let _ = PHCollectionListChangeRequest::delete_collection_lists
        as fn(&[PHCollectionList]) -> Result<(), PhotoKitError>;

    let project_request = PHProjectChangeRequest::change_request_for_project(&project)
        .set_title("Project")
        .set_project_extension_data_bytes(b"data")
        .set_project_preview_image_file_url("/dev/null")
        .remove_assets(std::slice::from_ref(&asset));
    accepts_change_request(&project_request);
    let _ = <PHProjectChangeRequest as PHChangeRequest>::perform;

    let video_options = PHVideoRequestOptions {
        network_access_allowed: true,
        version: PHVideoRequestOptionsVersion::CURRENT,
        delivery_mode: PHVideoRequestOptionsDeliveryMode::AUTOMATIC,
    };
    assert!(video_options.network_access_allowed);
    let _ = PHImageManager::request_player_item_for_video
        as fn(
            &PHImageManager,
            &PHAsset,
            &PHVideoRequestOptions,
            u64,
        ) -> Result<PHVideoResult, PhotoKitError>;
    let _ = PHImageManager::request_export_session_for_video
        as fn(
            &PHImageManager,
            &PHAsset,
            &PHVideoRequestOptions,
            &str,
            u64,
        ) -> Result<PHVideoResult, PhotoKitError>;
    let _ = PHImageManager::request_av_asset_for_video
        as fn(
            &PHImageManager,
            &PHAsset,
            &PHVideoRequestOptions,
            u64,
        ) -> Result<PHVideoResult, PhotoKitError>;

    let _ = PHAssetResourceManager::request_data_for_asset_resource
        as fn(
            &PHAssetResourceManager,
            &PHAssetResource,
            &PHAssetResourceRequestOptions,
            u64,
        ) -> Result<PHAssetResourceDataResult, PhotoKitError>;
    let _ = PHAssetResourceManager::write_data_for_asset_resource
        as fn(
            &PHAssetResourceManager,
            &PHAssetResource,
            &str,
            &PHAssetResourceRequestOptions,
            u64,
        ) -> Result<PHAssetResourceWriteResult, PhotoKitError>;

    let frame = PHLivePhotoFrame {
        frame_type: PHLivePhotoFrameType::PHOTO,
        time_seconds: 0.0,
        render_scale: 1.0,
        image_width: 10.0,
        image_height: 10.0,
    };
    assert_eq!(frame.frame_type, PHLivePhotoFrameType::PHOTO);
    let _ = PHLivePhotoEditingContext::set_frame_processor::<
        fn(PHLivePhotoFrame) -> PHLivePhotoFrameProcessingDecision,
    >;
    let _ = PHLivePhotoEditingContext::prepare_live_photo_for_playback
        as fn(
            &PHLivePhotoEditingContext,
            f64,
            f64,
            u64,
        ) -> Result<PHLivePhotoResult, PhotoKitError>;
    let _ = PHLivePhotoEditingContext::save_live_photo_to_output
        as fn(
            &PHLivePhotoEditingContext,
            &PHContentEditingOutput,
            u64,
        ) -> Result<PHLivePhotoEditingSaveResult, PhotoKitError>;
}
