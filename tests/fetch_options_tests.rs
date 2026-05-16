use photokit::prelude::*;

#[test]
fn fetch_options_builder_captures_every_field() {
    let options = PHFetchOptions::default()
        .with_predicate("mediaType == 1")
        .with_sort_descriptor(PHSortDescriptor::new("creationDate", false))
        .with_include_hidden_assets(true)
        .with_include_all_burst_assets(true)
        .with_include_asset_source_types(3)
        .with_fetch_limit(5)
        .with_wants_incremental_change_details(false);

    assert_eq!(options.predicate.as_deref(), Some("mediaType == 1"));
    assert_eq!(options.sort_descriptors.len(), 1);
    assert!(options.include_hidden_assets);
    assert!(options.include_all_burst_assets);
    assert_eq!(options.include_asset_source_types, Some(3));
    assert_eq!(options.fetch_limit, Some(5));
    assert!(!options.wants_incremental_change_details);
}
