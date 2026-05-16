use photokit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = PHFetchOptions::default()
        .with_predicate("mediaType == 1")
        .with_fetch_limit(3)
        .with_sort_descriptor(PHSortDescriptor::new("creationDate", false));
    println!("fetch limit: {:?}", options.fetch_limit);

    if PHPhotoLibrary::authorization_status().is_authorized() {
        let assets = PHAsset::fetch(&options)?;
        println!("filtered assets: {}", assets.len());
    }

    Ok(())
}
