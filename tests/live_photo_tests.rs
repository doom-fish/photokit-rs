mod common;

use photokit::prelude::*;

#[test]
fn live_photo_result_exposes_live_photo_snapshot() {
    let result = PHLivePhotoResult {
        has_live_photo: true,
        cancelled: false,
        degraded: false,
        size_width: 320.0,
        size_height: 240.0,
    };

    let live_photo = result.live_photo().expect("expected live photo snapshot");
    assert!(live_photo.available);
    assert!((live_photo.size_width - 320.0).abs() < f64::EPSILON);
}

#[test]
fn live_photo_request_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let Some(asset) = common::first_live_photo_asset() else {
        return Ok(());
    };

    let manager = PHImageManager::shared()?;
    let request = manager.request_live_photo(&asset, common::default_image_request())?;
    match request.wait(10_000) {
        Ok(_) => {}
        Err(error) if common::is_skippable_media_request_error(&error) => return Ok(()),
        Err(error) => return Err(error.into()),
    }
    Ok(())
}
