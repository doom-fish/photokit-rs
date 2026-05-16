use photokit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(resources) = std::env::var("PHOTOKIT_LIVE_PHOTO_RESOURCES") {
        let files = resources
            .split(';')
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        if !files.is_empty() {
            let mut request_options = PHImageRequest::new(320.0, 240.0, PHImageContentMode::AspectFit);
            request_options.network_access_allowed = true;
            let request = PHLivePhoto::request_with_resource_file_urls(&files, &request_options)?;
            let result = request.wait(10_000)?;
            println!("live photo from files: {}", result.has_live_photo);
            return Ok(());
        }
    }

    if !PHPhotoLibrary::authorization_status().is_authorized() {
        println!("photos access not granted; skipping PHLivePhoto example");
        return Ok(());
    }

    let assets = PHAsset::fetch(&PHFetchOptions::default())?;
    let Some(asset) = assets.into_vec().into_iter().find(PHAsset::is_live_photo) else {
        println!("no live photo asset available");
        return Ok(());
    };

    let manager = PHImageManager::shared()?;
    let mut request_options = PHImageRequest::new(320.0, 240.0, PHImageContentMode::AspectFit);
    request_options.network_access_allowed = true;
    let request = manager.request_live_photo(&asset, request_options)?;
    let result = match request.wait(10_000) {
        Ok(result) => result,
        Err(PhotoKitError::Framework(error))
            if error.message.contains("timed out")
                || error.message.contains("cancelled")
                || error.message.contains("in cloud")
                || error.message.contains("in iCloud") =>
        {
            println!("live photo request unavailable for this asset: {}", error.message);
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };
    println!("requested live photo: {}", result.has_live_photo);
    Ok(())
}
