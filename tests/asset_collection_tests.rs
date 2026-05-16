mod common;

use photokit::prelude::*;

#[test]
fn asset_collection_fetch_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let Some(library) = common::authorized_library() else {
        return Ok(());
    };

    let collections = library.fetch_asset_collections(&PHFetchOptions::default())?;
    if let Some(collection) = collections.first() {
        let _ = collection.can_perform_edit_operation(PHCollectionEditOperation::Rename)?;
        let _ = collection.assets(&PHFetchOptions::default())?;
    }
    Ok(())
}
