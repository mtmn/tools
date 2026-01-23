use crate::fetchers::discogs::DiscogsResult;
use std::collections::HashSet;

pub fn process(result: &DiscogsResult) -> Vec<String> {
    let mut subgenres = HashSet::<String>::new();

    if let Some(s) = &result.style {
        for style in s {
            subgenres.insert(style.clone());
        }
    }

    let mut sorted_subgenres: Vec<_> = subgenres.into_iter().collect();
    sorted_subgenres.sort();
    sorted_subgenres
}
