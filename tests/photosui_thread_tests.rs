use std::thread;

use photokit::prelude::*;

fn off_main_thread<F>(create: F) -> String
where
    F: FnOnce() -> Result<(), PhotoKitError> + Send + 'static,
{
    let error = thread::spawn(create)
        .join()
        .expect("creation thread panicked")
        .expect_err("creation off the main thread should fail");
    match error {
        PhotoKitError::Framework(info) => info.message,
        other => other.to_string(),
    }
}

#[test]
fn live_photo_view_rejects_creation_off_the_main_thread() {
    let message = off_main_thread(|| PHLivePhotoView::new().map(drop));
    assert!(message.contains("main thread"), "{message}");
}

#[test]
fn picker_rejects_creation_off_the_main_thread() {
    let message =
        off_main_thread(|| PHPickerViewController::new(&PHPickerConfiguration::new()).map(drop));
    assert!(message.contains("main thread"), "{message}");
}
