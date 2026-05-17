use photokit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !PHPhotoLibrary::authorization_status().is_authorized() {
        println!("photos access not granted; skipping PHContentEditingOutput example");
        return Ok(());
    }

    let assets = PHAsset::fetch(&PHFetchOptions::default().with_fetch_limit(1))?;
    let Some(asset) = assets.first() else {
        println!("no assets available");
        return Ok(());
    };

    let options = PHContentEditingInputRequestOptions {
        network_access_allowed: true,
        ..PHContentEditingInputRequestOptions::default()
    };
    let input = match asset.request_content_editing_input(&options, 10_000) {
        Ok(input) => input,
        Err(PhotoKitError::Framework(error))
            if error.message.contains("timed out")
                || error.message.contains("cancelled")
                || error.message.contains("in iCloud") =>
        {
            println!(
                "content editing output unavailable for this asset: {}",
                error.message
            );
            return Ok(());
        }
        Err(PhotoKitError::OperationFailed(message))
            if message.contains("missing PHContentEditingInput result") =>
        {
            println!("content editing output unavailable for this asset: {message}");
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };
    let output = input.create_content_editing_output()?;
    println!("rendered content url: {}", output.rendered_content_url);
    Ok(())
}
