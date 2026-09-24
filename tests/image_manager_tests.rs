mod common;

use photokit::prelude::*;

#[test]
fn image_manager_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let Some(asset) = common::first_asset() else {
        return Ok(());
    };

    let manager = PHImageManager::shared()?;
    let request = manager.request_image_data(&asset, &common::default_image_request())?;
    match request.wait(10_000) {
        Ok(_) => {}
        Err(error) if common::is_skippable_media_request_error(&error) => return Ok(()),
        Err(error) => return Err(error.into()),
    }

    let caching_manager = PHCachingImageManager::new()?;
    caching_manager.stop_caching_images_for_all_assets();
    Ok(())
}

#[test]
fn image_requests_complete_while_the_main_thread_is_blocked(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(asset) = common::first_asset() else {
        return Ok(());
    };

    let manager = PHImageManager::shared()?;
    let request = PHImageRequest::new(64.0, 64.0, PHImageContentMode::AspectFit);
    let image = manager.request_image(&asset, request)?.wait(20_000).err();
    let data = manager
        .request_image_data(&asset, &request)?
        .wait(20_000)
        .err();
    for error in [image, data].iter().flatten() {
        assert!(
            !matches!(error, PhotoKitError::Framework(error) if error.message.contains("timed out")),
            "{error:?}"
        );
    }
    Ok(())
}
