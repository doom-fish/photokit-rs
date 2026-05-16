use photokit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let status = PHPhotoLibrary::authorization_status();
    let scoped_status = PHPhotoLibrary::authorization_status_for_access_level(PHAccessLevel::ReadWrite);
    println!("authorization: {status:?} / {scoped_status:?}");

    if scoped_status.is_authorized() {
        let library = PHPhotoLibrary::shared()?;
        let observer = library.register_change_observer(|change| {
            println!("observed {} change batch(es)", change.change_count);
        })?;
        drop(observer);
    }

    Ok(())
}
