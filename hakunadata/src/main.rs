mod fetchers;
mod metadata;

use anyhow::{Context, Result};
use clap::Parser;
use fetchers::discogs::DiscogsClient;
use fetchers::musicbrainz::MusicBrainzClient;

const EXAMPLES: &str = r"EXAMPLES:
    Fetch metadata for an artist and album:
    hakunadata 'Nirvana' 'Nevermind'";

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = None,
    after_help = EXAMPLES
)]
struct Args {
    /// Artist name
    artist: String,

    /// Album name
    album: String,
}

struct AppContext {
    mb_client: Option<MusicBrainzClient>,
    discogs_client: Option<DiscogsClient>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let (discogs_client, mb_client) = if std::env::var("DISCOGS_TOKEN").is_ok() {
        (
            Some(DiscogsClient::new().context("Failed to init Discogs client")?),
            None,
        )
    } else {
        (
            None,
            Some(MusicBrainzClient::new().context("Failed to init MusicBrainz client")?),
        )
    };

    let ctx = AppContext {
        mb_client,
        discogs_client,
    };

    let result = process_query(&ctx, &args.artist, &args.album).await?;

    if result.genres.is_empty() {
        println!("Genres: (none)");
    } else {
        println!("Genres:");
        for genre in &result.genres {
            println!("  {genre}");
        }
    }

    if result.subgenres.is_empty() {
        println!("Subgenres: (none)");
    } else {
        println!("Subgenres:");
        for subgenre in &result.subgenres {
            println!("  {subgenre}");
        }
    }

    if result.labels.is_empty() {
        println!("Label: (none)");
    } else {
        println!("Label:");
        for label in &result.labels {
            println!("  {label}");
        }
    }

    Ok(())
}

struct FetchedMetadata {
    genres: Vec<String>,
    subgenres: Vec<String>,
    labels: Vec<String>,
}

async fn process_query(ctx: &AppContext, artist: &str, album: &str) -> Result<FetchedMetadata> {
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
        let g = metadata::genres::process(&data);
        for item in g {
            genres.insert(item);
        }

        let s = metadata::subgenres::process(&data);
        for item in s {
            subgenres.insert(item);
        }

        let l = metadata::labels::process(&data);
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
