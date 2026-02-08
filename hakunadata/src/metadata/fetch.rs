use crate::context::AppContext;
use crate::metadata::{genres, labels, subgenres};
use anyhow::Result;

pub struct FetchedMetadata {
    pub genres: Vec<String>,
    pub subgenres: Vec<String>,
    pub labels: Vec<String>,
}

pub async fn process_query(ctx: &AppContext, artist: &str, album: &str) -> Result<FetchedMetadata> {
    // Run fetchers concurrently (or rather, run whichever is enabled)
    let discogs_future = async {
        if let Some(client) = &ctx.discogs_client {
            client.fetch_metadata(artist, album).await
        } else {
            Ok(None)
        }
    };

    let mb_future = async {
        if let Some(client) = &ctx.mb_client {
            client.fetch_genres(artist, album).await
        } else {
            Ok(vec![])
        }
    };

    let (discogs_res, mb_res) = tokio::join!(discogs_future, mb_future);

    let mut genres = std::collections::HashSet::new();
    let mut subgenres = std::collections::HashSet::new();
    let mut labels = std::collections::HashSet::new();

    // Process Discogs
    if let Ok(Some(data)) = discogs_res {
        let g = genres::process(&data);
        for item in g {
            genres.insert(item);
        }

        let s = subgenres::process(&data);
        for item in s {
            subgenres.insert(item);
        }

        let l = labels::process(&data);
        for item in l {
            labels.insert(item);
        }
    }

    // Process MusicBrainz
    if let Ok(mb_genres) = mb_res {
        for g in mb_genres {
            genres.insert(g);
        }
    }

    let mut sorted_genres: Vec<_> = genres.into_iter().collect();
    sorted_genres.sort();

    let mut sorted_subgenres: Vec<_> = subgenres.into_iter().collect();
    sorted_subgenres.sort();

    let mut sorted_labels: Vec<_> = labels.into_iter().collect();
    sorted_labels.sort();

    Ok(FetchedMetadata {
        genres: sorted_genres,
        subgenres: sorted_subgenres,
        labels: sorted_labels,
    })
}
