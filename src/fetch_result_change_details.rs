use serde::{Deserialize, Serialize};

use crate::fetch_result::PHFetchResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHFetchResultMove {
    pub from_index: usize,
    pub to_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PHFetchResultChangeDetails<T> {
    pub fetch_result_before_changes: PHFetchResult<T>,
    pub fetch_result_after_changes: PHFetchResult<T>,
    pub has_incremental_changes: bool,
    #[serde(default)]
    pub removed_indexes: Vec<usize>,
    #[serde(default)]
    pub removed_objects: Vec<T>,
    #[serde(default)]
    pub inserted_indexes: Vec<usize>,
    #[serde(default)]
    pub inserted_objects: Vec<T>,
    #[serde(default)]
    pub changed_indexes: Vec<usize>,
    #[serde(default)]
    pub changed_objects: Vec<T>,
    #[serde(default)]
    pub moves: Vec<PHFetchResultMove>,
}

impl<T> PHFetchResultChangeDetails<T> {
    pub fn has_moves(&self) -> bool {
        !self.moves.is_empty()
    }
}

impl<T: Clone + PartialEq> PHFetchResultChangeDetails<T> {
    pub fn change_details_from_fetch_result(
        from_result: &PHFetchResult<T>,
        to_result: &PHFetchResult<T>,
        changed_objects: &[T],
    ) -> Self {
        let removed_indexes: Vec<usize> = from_result
            .iter()
            .enumerate()
            .filter_map(|(index, object)| (!to_result.contains(object)).then_some(index))
            .collect();
        let removed_objects = from_result.objects_at_indexes(&removed_indexes);

        let inserted_indexes: Vec<usize> = to_result
            .iter()
            .enumerate()
            .filter_map(|(index, object)| (!from_result.contains(object)).then_some(index))
            .collect();
        let inserted_objects = to_result.objects_at_indexes(&inserted_indexes);

        let changed_indexes: Vec<usize> = to_result
            .iter()
            .enumerate()
            .filter_map(|(index, object)| changed_objects.contains(object).then_some(index))
            .collect();
        let changed_objects = to_result.objects_at_indexes(&changed_indexes);

        let moves = from_result
            .iter()
            .enumerate()
            .filter_map(|(from_index, object)| {
                let to_index = to_result.index_of(object)?;
                (from_index != to_index).then_some(PHFetchResultMove {
                    from_index,
                    to_index,
                })
            })
            .collect();

        Self {
            fetch_result_before_changes: from_result.clone(),
            fetch_result_after_changes: to_result.clone(),
            has_incremental_changes: true,
            removed_indexes,
            removed_objects,
            inserted_indexes,
            inserted_objects,
            changed_indexes,
            changed_objects,
            moves,
        }
    }
}
