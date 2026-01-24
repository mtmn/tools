use clap::Parser;
use csv::Writer;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

const TRACKS_PER_PAGE: u32 = 200;
const API_BASE_URL: &str = "https://ws.audioscrobbler.com/2.0/";

#[derive(Parser, Debug)]
#[command(name = "lastfm-csv-export")]
#[command(about = "Export Last.fm scrobbles to a .csv file", long_about = None)]
struct Args {
    /// Last.fm username
    #[arg(short, long)]
    username: String,

    /// Last.fm API key
    #[arg(short, long)]
    api_key: String,

    /// Output CSV file path
    #[arg(short, long, default_value = "scrobbles.csv")]
    output: String,

    /// Start timestamp (Unix timestamp)
    #[arg(long)]
    from: Option<u64>,

    /// End timestamp (Unix timestamp)
    #[arg(long)]
    to: Option<u64>,

    /// Maximum number of pages to fetch (200 tracks per page)
    #[arg(short, long)]
    limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct LastFmResponse {
    recenttracks: RecentTracks,
}

#[derive(Debug, Deserialize)]
struct RecentTracks {
    track: Vec<Track>,
    #[serde(rename = "@attr")]
    attr: TrackAttributes,
}

#[derive(Debug, Deserialize)]
struct TrackAttributes {
    total: String,
    #[serde(rename = "totalPages")]
    total_pages: String,
}

#[derive(Debug, Deserialize)]
struct Track {
    artist: ArtistInfo,
    album: AlbumInfo,
    name: String,
    date: Option<DateInfo>,
}

#[derive(Debug, Deserialize)]
struct ArtistInfo {
    #[serde(rename = "#text")]
    text: String,
}

#[derive(Debug, Deserialize)]
struct AlbumInfo {
    #[serde(rename = "#text")]
    text: String,
}

#[derive(Debug, Deserialize)]
struct DateInfo {
    #[serde(rename = "#text")]
    text: String,
}

#[derive(Debug, Serialize)]
struct CsvRecord {
    artist: String,
    album: String,
    track: String,
    date: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    println!("Fetching scrobbles for user: {}", args.username);

    let client = Client::new();
    let tracks = fetch_all_tracks(&client, &args)?;

    println!("Writing {} tracks to {}", tracks.len(), args.output);
    write_csv(&args.output, &tracks)?;

    println!("Done!");
    Ok(())
}

fn fetch_all_tracks(client: &Client, args: &Args) -> Result<Vec<Track>, Box<dyn Error>> {
    let mut all_tracks = Vec::new();

    // First request to get total pages
    let first_response = fetch_page(client, args, 1)?;
    let total_pages: u32 = first_response.recenttracks.attr.total_pages.parse()?;
    let total_tracks: u32 = first_response.recenttracks.attr.total.parse()?;

    println!("Total tracks: {}", total_tracks);
    println!("Total pages: {}", total_pages);

    // Add tracks from first page
    all_tracks.extend(first_response.recenttracks.track);

    // Determine how many pages to fetch
    let max_page = args
        .limit
        .map_or(total_pages, |limit| limit.min(total_pages));

    // Fetch remaining pages
    for page in 2..=max_page {
        println!("Fetching page {}/{}", page, max_page);
        let response = fetch_page(client, args, page)?;
        all_tracks.extend(response.recenttracks.track);
    }

    Ok(all_tracks)
}

fn fetch_page(client: &Client, args: &Args, page: u32) -> Result<LastFmResponse, Box<dyn Error>> {
    let mut url = format!(
        "{}?method=user.getrecenttracks&user={}&api_key={}&format=json&limit={}&page={}",
        API_BASE_URL, args.username, args.api_key, TRACKS_PER_PAGE, page
    );

    if let Some(from) = args.from {
        url.push_str(&format!("&from={}", from));
    }

    if let Some(to) = args.to {
        url.push_str(&format!("&to={}", to));
    }

    let response = client.get(&url).send()?;

    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }

    Ok(response.json()?)
}

fn write_csv(path: &str, tracks: &[Track]) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut writer = Writer::from_writer(file);

    // Write empty header row as Maloja doesn't expect it
    writer.write_record(["", "", "", ""])?;

    for track in tracks {
        let date_str = track
            .date
            .as_ref()
            .map(|date| {
                // Input format: "29 Sep 2025, 15:32"
                date.text.replace(", ", " ")
            })
            .unwrap_or_default();

        let record = CsvRecord {
            artist: track.artist.text.clone(),
            album: track.album.text.clone(),
            track: track.name.clone(),
            date: date_str,
        };

        writer.serialize(&record)?;
    }

    writer.flush()?;
    Ok(())
}

