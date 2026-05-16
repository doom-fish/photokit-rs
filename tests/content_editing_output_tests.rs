mod common;


#[test]
fn content_editing_output_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let Some(asset) = common::first_asset() else {
        return Ok(());
    };

    let Some(input) = common::request_content_editing_input(&asset)? else {
        return Ok(());
    };
    let output = input.create_content_editing_output()?;
    assert!(!output.rendered_content_url.is_empty());
    Ok(())
}
