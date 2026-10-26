mod common;

use photokit::prelude::*;

fn caught_exception<T: std::fmt::Debug>(result: Result<T, PhotoKitError>) -> String {
    match result {
        Err(PhotoKitError::Framework(info)) if info.domain == "PhotoKitObjCBridge" => info.message,
        other => panic!("expected a caught Objective-C exception, got {other:?}"),
    }
}

fn fetch_with_predicate(predicate: &str) -> String {
    caught_exception(PHAsset::fetch(
        &PHFetchOptions::default().with_predicate(predicate),
    ))
}

#[test]
fn predicate_format_specifiers_are_rejected_instead_of_reading_varargs() {
    for predicate in ["localIdentifier == %@", "%K == 1", "pixelWidth > %d"] {
        assert_ne!(fetch_with_predicate(predicate), "");
    }
}

#[test]
fn malformed_predicates_are_rejected() {
    assert_ne!(fetch_with_predicate("mediaType =="), "");
    assert_ne!(fetch_with_predicate("((mediaType == 1)"), "");
}

#[test]
fn unsupported_fetch_keys_are_rejected_when_the_library_is_available() {
    if common::authorized_library().is_none() {
        return;
    }
    assert_ne!(fetch_with_predicate("photokitNoSuchKey == 1"), "");

    let unsupported_sort = PHFetchOptions::default()
        .with_sort_descriptor(PHSortDescriptor::new("photokitNoSuchKey", true));
    assert_ne!(caught_exception(PHAsset::fetch(&unsupported_sort)), "");
    assert_ne!(
        caught_exception(PHAssetCollection::fetch(&unsupported_sort)),
        ""
    );
}
