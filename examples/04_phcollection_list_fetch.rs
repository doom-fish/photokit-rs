use photokit::prelude::*;

const ANY_SUBTYPE: i64 = i64::MAX;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !PHPhotoLibrary::authorization_status().is_authorized() {
        println!("photos access not granted; skipping PHCollectionList example");
        return Ok(());
    }

    let lists = PHCollectionList::fetch_with_type(
        PHCollectionListType::Folder,
        ANY_SUBTYPE,
        &PHFetchOptions::default(),
    )?;
    println!("folder lists: {}", lists.len());
    if let Some(list) = lists.first() {
        println!("first list: {:?}", list.localized_title);
    }
    Ok(())
}
