mod common;

use photokit::prelude::*;

#[test]
fn photo_library_authorization_queries_do_not_fail() {
    let _ = PHPhotoLibrary::authorization_status();
    let _ = PHPhotoLibrary::authorization_status_for_access_level(PHAccessLevel::ReadWrite);
}

#[test]
fn photo_library_can_register_change_observers() -> Result<(), Box<dyn std::error::Error>> {
    let Some(library) = common::authorized_library() else {
        return Ok(());
    };

    let summary_observer = library.register_change_observer(|_change| {})?;
    let detailed_observer = library.register_detailed_change_observer(|_change| {})?;
    drop(summary_observer);
    drop(detailed_observer);
    Ok(())
}
