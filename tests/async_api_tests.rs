//! Tests for `async_api` (feature = `async`). Run with:
//!   cargo test --all-features --test `async_api_tests`
#![cfg(feature = "async")]

mod common;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use photokit::async_api::{AsyncPHImageManager, AsyncPHPhotoLibrary};
use photokit::error::PHAuthorizationStatus;
use photokit::{PHAccessLevel, PHImageContentMode, PHImageRequest, PHPhotoLibrary};

fn authorization_is_determined(access_level: PHAccessLevel) -> bool {
    PHPhotoLibrary::authorization_status_for_access_level(access_level)
        != PHAuthorizationStatus::NotDetermined
}

/// Happy path: `request_authorization` resolves to a valid `PHAuthorizationStatus`.
#[test]
fn test_request_authorization_resolves() {
    if !authorization_is_determined(PHAccessLevel::ReadWrite) {
        return;
    }
    let status = pollster::block_on(AsyncPHPhotoLibrary::request_authorization(
        PHAccessLevel::ReadWrite,
    ));

    assert!(
        matches!(
            status,
            Ok(PHAuthorizationStatus::Authorized
                | PHAuthorizationStatus::Denied
                | PHAuthorizationStatus::NotDetermined
                | PHAuthorizationStatus::Limited
                | PHAuthorizationStatus::Restricted
                | PHAuthorizationStatus::Unknown(_))
        ),
        "unexpected result: {status:?}"
    );
}

/// Error path: calling with add-only access level should also resolve (may map to denied).
#[test]
fn test_request_authorization_add_only_resolves() {
    if !authorization_is_determined(PHAccessLevel::AddOnly) {
        return;
    }
    let status = pollster::block_on(AsyncPHPhotoLibrary::request_authorization(
        PHAccessLevel::AddOnly,
    ));

    assert!(status.is_ok(), "expected Ok but got {status:?}");
}

#[test]
fn image_futures_resolve_while_the_main_thread_is_blocked() {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let resolved = common::first_asset().map(|asset| {
            let manager = AsyncPHImageManager::shared().expect("image manager");
            let request = PHImageRequest::new(64.0, 64.0, PHImageContentMode::AspectFit);
            let image = manager
                .request_image(&asset, request)
                .map(pollster::block_on);
            let data = manager
                .request_image_data(&asset, &request)
                .map(pollster::block_on);
            (image.is_ok(), data.is_ok())
        });
        let _ = sender.send(resolved);
    });
    assert!(
        receiver.recv_timeout(Duration::from_secs(30)).is_ok(),
        "image futures did not resolve"
    );
}
