#![allow(dead_code)]

use photokit::prelude::*;

pub const ANY_SUBTYPE: i64 = i64::MAX;

pub fn authorized_library() -> Option<PHPhotoLibrary> {
    PHPhotoLibrary::authorization_status()
        .is_authorized()
        .then(PHPhotoLibrary::shared)
        .and_then(Result::ok)
}

pub fn first_asset() -> Option<PHAsset> {
    authorized_library()
        .and_then(|_| PHAsset::fetch(&PHFetchOptions::default().with_fetch_limit(1)).ok())
        .and_then(|assets| assets.into_vec().into_iter().next())
}

pub fn first_asset_collection() -> Option<PHAssetCollection> {
    authorized_library().and_then(|library| {
        library
            .fetch_asset_collections(&PHFetchOptions::default().with_fetch_limit(1))
            .ok()
            .and_then(|collections| collections.into_vec().into_iter().next())
    })
}

pub fn first_collection_list() -> Option<PHCollectionList> {
    authorized_library().and_then(|_| {
        PHCollectionList::fetch_with_type(
            PHCollectionListType::Folder,
            ANY_SUBTYPE,
            &PHFetchOptions::default().with_fetch_limit(1),
        )
        .ok()
        .and_then(|lists| lists.into_vec().into_iter().next())
        .or_else(|| {
            PHCollectionList::fetch_with_type(
                PHCollectionListType::SmartFolder,
                ANY_SUBTYPE,
                &PHFetchOptions::default().with_fetch_limit(1),
            )
            .ok()
            .and_then(|lists| lists.into_vec().into_iter().next())
        })
    })
}

pub fn first_live_photo_asset() -> Option<PHAsset> {
    authorized_library().and_then(|_| {
        PHAsset::fetch(&PHFetchOptions::default())
            .ok()
            .and_then(|assets| assets.into_vec().into_iter().find(PHAsset::is_live_photo))
    })
}

pub fn default_image_request() -> PHImageRequest {
    let mut request = PHImageRequest::new(320.0, 240.0, PHImageContentMode::AspectFit);
    request.network_access_allowed = true;
    request
}

pub fn is_skippable_media_request_error(error: &PhotoKitError) -> bool {
    match error {
        PhotoKitError::Framework(error) => {
            error.message.contains("timed out")
                || error.message.contains("cancelled")
                || error.message.contains("in cloud")
                || error.message.contains("in iCloud")
        }
        _ => false,
    }
}

pub fn request_content_editing_input(
    asset: &PHAsset,
) -> Result<Option<PHContentEditingInput>, PhotoKitError> {
    let options = PHContentEditingInputRequestOptions {
        network_access_allowed: true,
        ..PHContentEditingInputRequestOptions::default()
    };

    match asset.request_content_editing_input(&options, 10_000) {
        Ok(input) => Ok(Some(input)),
        Err(PhotoKitError::Framework(error))
            if error.message.contains("timed out")
                || error.message.contains("cancelled")
                || error.message.contains("in iCloud") =>
        {
            Ok(None)
        }
        Err(PhotoKitError::OperationFailed(message))
            if message.contains("missing PHContentEditingInput result") =>
        {
            Ok(None)
        }
        Err(error) => Err(error),
    }
}
