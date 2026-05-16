mod common;

#[test]
fn change_observer_registration_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let Some(library) = common::authorized_library() else {
        return Ok(());
    };

    let observer = library.register_detailed_change_observer(|_change| {})?;
    drop(observer);
    Ok(())
}
