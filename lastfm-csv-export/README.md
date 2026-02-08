# Last.fm CSV Export

A command-line tool to export your Last.fm scrobbles to a CSV file.

## Features

- Export all your listening history from Last.fm
- Filter by date range (start and end timestamps)
- Limit the number of pages to fetch
- Output to a customizable CSV file

## Prerequisites

- A Last.fm account
- A Last.fm API key (get one from [Last.fm API](https://www.last.fm/api/account/create))

## Installation

```bash
cargo build --release
```

The executable will be available at `target/release/lastfm-csv-export`.

## Usage

```bash
lastfm-csv-export -u <USERNAME> -k <API_KEY> -o <OUTPUT_FILE>
```

### Options

- `-u, --username <USERNAME>`: Your Last.fm username (required)
- `-k, --api_key <API_KEY>`: Your Last.fm API key (required)
- `-o, --output <OUTPUT_FILE>`: Output CSV file path (default: "scrobbles.csv")
- `--from <FROM>`: Start timestamp (Unix timestamp)
- `--to <TO>`: End timestamp (Unix timestamp)
- `-l, --limit <LIMIT>`: Maximum number of pages to fetch (200 tracks per page)

### Example

```bash
# Export all scrobbles
lastfm-csv-export -u myusername -k myapikey

# Export with custom output file
lastfm-csv-export -u myusername -k myapikey -o my_scrobbles.csv

# Export with date range
lastfm-csv-export -u myusername -k myapikey --from 1609459200 --to 1640995200
```

## Output Format

The exported CSV file contains the following columns:
- `artist`: The artist name
- `album`: The album name
- `track`: The track name
- `date`: The scrobble date and time