mod context;
mod fetchers;
mod metadata;
mod tagging;

use anyhow::{Context, Result};
use clap::Parser;
use context::AppContext;
use fetchers::discogs::DiscogsClient;
use fetchers::musicbrainz::MusicBrainzClient;
use metadata::fetch::process_query;
use std::path::Path;
use tagging::{print_metadata, process_file};
use walkdir::WalkDir;

static EXAMPLES: &str = r"EXAMPLES:
    Fetch metadata for an artist and album:
    hakunadata --artist 'Djrum' --album 'Under Tangled Silence'

    Preview tags for a single file:
    hakunadata --read file.mp3

    Write tags to a single file:
    hakunadata --write file.mp3

    Show proposed tags for all files in a directory:
    hakunadata --read /path/to/music

    Write tags to all files in a directory:
    hakunadata --write /path/to/music";

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
    #[arg(long)]
    artist: Option<String>,

    /// Album name
    #[arg(long)]
    album: Option<String>,

    /// File(s) to process
    #[arg(required_unless_present_any = ["artist", "album"])]
    files: Vec<String>,

    /// Write tags to file
    #[arg(short, long, default_value_t = false)]
    write: bool,

    /// Read tags from file and show what would be written
    #[arg(short, long, default_value_t = true)]
    read: bool,
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

    if !args.files.is_empty() {
        for path_str in args.files {
            let path = Path::new(&path_str);
            if !path.exists() {
                eprintln!("Path does not exist: {}", path.display());
                continue;
            }

            let mut files_to_process = Vec::new();
            if path.is_dir() {
                // Fix redundant_closure: |e| e.ok() -> Result::ok
                for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
                    if entry.file_type().is_file()
                        && entry
                            .path()
                            .extension()
                            .and_then(|s| s.to_str())
                            .is_some_and(|ext| matches!(ext, "flac" | "mp3" | "ogg" | "m4a"))
                    {
                        files_to_process.push(entry.path().to_path_buf());
                    }
                }
            } else {
                files_to_process.push(path.to_path_buf());
            }

            for file_path in files_to_process {
                if let Err(e) = process_file(&ctx, &file_path, args.read, args.write).await {
                    eprintln!("Failed to process file {}: {e:?}", file_path.display());
                }
            }
        }
    } else if let (Some(artist), Some(album)) = (args.artist, args.album) {
        let result = process_query(&ctx, &artist, &album).await?;
        print_metadata(&result);
    }

    Ok(())
}
