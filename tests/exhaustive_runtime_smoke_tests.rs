mod common;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use photokit::prelude::*;

#[test]
fn collection_project_history_and_media_runtime_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let Some(library) = common::authorized_library() else {
        return Ok(());
    };

    let _availability = library.unavailability_reason()?;
    let availability_observer = library.register_availability_observer(|_change| {})?;
    let _collections = PHCollection::fetch_top_level_user_collections(
        &PHFetchOptions::default().with_fetch_limit(5),
    )?;
    let _projects = PHProject::fetch_top_level_user_collections(
        &PHFetchOptions::default().with_fetch_limit(5),
    )?;
    let token = library.current_change_token()?;
    let _changes = library.fetch_persistent_changes_since_token(&token)?;
    if let Some(list) = common::first_collection_list() {
        let _ = PHCollection::fetch_collections_in_collection_list(
            &list,
            &PHFetchOptions::default().with_fetch_limit(5),
        )?;
    }
    drop(availability_observer);

    if let Some(asset) = common::first_asset() {
        let resources = asset.resources()?;
        if let Some(resource) = resources.first() {
            let manager = PHAssetResourceManager;
            let options = PHAssetResourceRequestOptions {
                network_access_allowed: true,
            };
            match manager.request_data_for_asset_resource(resource, &options, 10_000) {
                Ok(_) => {}
                Err(error) if common::is_skippable_media_request_error(&error) => {}
                Err(error) => return Err(error.into()),
            }
        }
    }

    if let Some(video_asset) = common::first_video_asset() {
        let manager = PHImageManager::shared()?;
        let options = PHVideoRequestOptions {
            network_access_allowed: true,
            version: PHVideoRequestOptionsVersion::CURRENT,
            delivery_mode: PHVideoRequestOptionsDeliveryMode::AUTOMATIC,
        };
        match manager.request_av_asset_for_video(&video_asset, &options, 10_000) {
            Ok(_) => {}
            Err(error) if common::is_skippable_media_request_error(&error) => {}
            Err(error) => return Err(error.into()),
        }
        match manager.request_player_item_for_video(&video_asset, &options, 10_000) {
            Ok(_) => {}
            Err(error) if common::is_skippable_media_request_error(&error) => {}
            Err(error) => return Err(error.into()),
        }
    }

    Ok(())
}

#[test]
fn live_photo_editing_runtime_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let Some(asset) = common::first_live_photo_asset() else {
        return Ok(());
    };
    let Some(input) = common::request_content_editing_input(&asset)? else {
        return Ok(());
    };

    let seen = Arc::new(AtomicBool::new(false));
    let seen_clone = Arc::clone(&seen);
    let mut context = match PHLivePhotoEditingContext::new(&input) {
        Ok(context) => context,
        Err(error) if common::is_skippable_media_request_error(&error) => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    context.set_audio_volume(0.5)?;
    context.set_frame_processor(move |frame| {
        seen_clone.store(
            frame.frame_type == PHLivePhotoFrameType::PHOTO,
            Ordering::Relaxed,
        );
        PHLivePhotoFrameProcessingDecision::KeepOriginal
    })?;
    match context.prepare_live_photo_for_playback(320.0, 240.0, 10_000) {
        Ok(_) => {
            let _ = seen.load(Ordering::Relaxed);
        }
        Err(error) if common::is_skippable_media_request_error(&error) => return Ok(()),
        Err(error) => return Err(error.into()),
    }
    context.cancel();
    context.clear_frame_processor();
    Ok(())
}
