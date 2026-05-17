//! Tests for `async_api` (feature = `async`). Run with:
//!   cargo test --all-features --test `async_api_tests`
#![cfg(feature = "async")]

use photokit::async_api::AsyncPHPhotoLibrary;
use photokit::error::PHAuthorizationStatus;
use photokit::PHAccessLevel;

/// Happy path: `request_authorization` resolves to a valid `PHAuthorizationStatus`.
#[test]
fn test_request_authorization_resolves() {
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
    let status = pollster::block_on(AsyncPHPhotoLibrary::request_authorization(
        PHAccessLevel::AddOnly,
    ));

    assert!(status.is_ok(), "expected Ok but got {status:?}");
}
