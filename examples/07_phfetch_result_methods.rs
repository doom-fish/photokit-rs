use photokit::prelude::*;

fn main() {
    let result = PHFetchResult::from(vec![1, 2, 3]);
    println!(
        "len={} first={:?} last={:?}",
        result.len(),
        result.first(),
        result.last()
    );
    println!("subset={:?}", result.objects_at_indexes(&[0, 2]));
}
