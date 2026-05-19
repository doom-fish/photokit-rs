use std::process::Command;

#[test]
fn picker_and_live_photo_view_runtime_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned()))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["run", "--quiet", "--all-features", "--example", "17_photosui_runtime_smoke"])
        .output()?;

    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    Ok(())
}
