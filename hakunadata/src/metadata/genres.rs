use crate::fetchers::discogs::DiscogsResult;
use std::collections::HashSet;

pub fn process(result: &DiscogsResult) -> Vec<String> {
    let mut genres = HashSet::<String>::new();

    if let Some(g) = &result.genre {
        for g_item in g {
            genres.insert(g_item.clone());
        }
    }

    let mut sorted_genres: Vec<_> = genres.into_iter().collect();
    sorted_genres.sort();
    sorted_genres
}
