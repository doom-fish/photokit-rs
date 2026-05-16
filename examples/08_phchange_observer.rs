use photokit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !PHPhotoLibrary::authorization_status().is_authorized() {
        println!("photos access not granted; skipping PHChange example");
        return Ok(());
    }

    let library = PHPhotoLibrary::shared()?;
    let observer = library.register_detailed_change_observer(|_change| {
        println!("received detailed PHChange callback");
    })?;
    drop(observer);
    println!("registered and removed PHChange observer");
    Ok(())
}
