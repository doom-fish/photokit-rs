use photokit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !PHPhotoLibrary::authorization_status().is_authorized() {
        println!("photos access not granted; skipping PHImageManager example");
        return Ok(());
    }

    let assets = PHAsset::fetch(&PHFetchOptions::default().with_fetch_limit(1))?;
    let Some(asset) = assets.first() else {
        println!("no assets available");
        return Ok(());
    };

    let manager = PHImageManager::shared()?;
    let mut request_options = PHImageRequest::new(320.0, 240.0, PHImageContentMode::AspectFit);
    request_options.network_access_allowed = true;
    let request = manager.request_image_data(asset, &request_options)?;
    let result = match request.wait(10_000) {
        Ok(result) => result,
        Err(PhotoKitError::Framework(error))
            if error.message.contains("timed out")
                || error.message.contains("cancelled")
                || error.message.contains("in cloud")
                || error.message.contains("in iCloud") =>
        {
            println!("image request unavailable for this asset: {}", error.message);
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };
    println!("image bytes: {}", result.data().len());
    Ok(())
}
