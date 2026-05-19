use photokit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let picker_filter = PHPickerFilter::images();
    assert!(!picker_filter.description()?.is_empty());

    let configuration = PHPickerConfiguration::new()
        .set_selection(PHPickerConfigurationSelection::Ordered)
        .set_selection_limit(2)
        .set_filter(picker_filter);
    let picker = PHPickerViewController::new(&configuration)?;
    let delegate = picker.register_delegate(|_results| {})?;
    assert_eq!(picker.configuration().selection_limit, 2);
    drop(delegate);

    let mut live_photo_view = PHLivePhotoView::new()?;
    let initial_info = live_photo_view.snapshot()?;
    assert!(!initial_info.has_live_photo);
    live_photo_view.set_muted(true)?;
    live_photo_view.set_audio_volume(0.25)?;
    let updated_info = live_photo_view.snapshot()?;
    assert!(updated_info.muted);
    let live_photo_delegate = live_photo_view.register_delegate(|_event| true)?;
    drop(live_photo_delegate);

    Ok(())
}
