mod common;

use photokit::prelude::*;

#[test]
fn collection_list_fetch_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let Some(collection_list) = common::first_collection_list() else {
        return Ok(());
    };

    let _ = collection_list.can_perform_edit_operation(PHCollectionEditOperation::Rename)?;
    let _ = collection_list.containing_collection_lists(&PHFetchOptions::default())?;
    Ok(())
}
