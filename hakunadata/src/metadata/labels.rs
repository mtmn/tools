use crate::fetchers::discogs::DiscogsResult;
use std::collections::HashSet;

pub fn process(result: &DiscogsResult) -> Vec<String> {
    let mut labels = HashSet::<String>::new();

    if let Some(l) = &result.label {
        for label in l {
            labels.insert(label.clone());
        }
    }

    let mut sorted_labels: Vec<_> = labels.into_iter().collect();
    sorted_labels.sort();
    sorted_labels
}
