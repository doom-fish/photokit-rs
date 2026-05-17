//! Async API smoke test — runs with `cargo run --example 16_async_api --features async`
//!
//! Uses `pollster::block_on` for executor-agnostic async.
//! Permission-gated operations (requestAuthorization that need UI) gracefully exit.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    use photokit::async_api::AsyncPHPhotoLibrary;
    use photokit::PHAccessLevel;

    println!("requesting authorization...");
    match AsyncPHPhotoLibrary::request_authorization(PHAccessLevel::ReadWrite).await {
        Ok(status) => println!("authorization status: {status:?}"),
        Err(error) => {
            println!("authorization not available in headless env: {error}");
            return Ok(());
        }
    }

    Ok(())
}
