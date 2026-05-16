mod common;


#[test]
fn content_editing_input_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let Some(asset) = common::first_asset() else {
        return Ok(());
    };

    let Some(input) = common::request_content_editing_input(&asset)? else {
        return Ok(());
    };
    let _ = input.snapshot();
    Ok(())
}
