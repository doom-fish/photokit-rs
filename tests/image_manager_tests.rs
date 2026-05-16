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
