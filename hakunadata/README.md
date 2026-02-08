# hakunadata

`hakunadata` is an opinionated tool that fetches music metadata (genres and labels) from Discogs or MusicBrainz and writes them to audio files.

## Usage

### Fetching metadata for an artist and album

```bash
hakunadata --artist 'The Beatles' --album 'Abbey Road'
```

### Reading tags from a file

This will fetch metadata based on the artist and album tags in the file and show you what tags would be written, without actually changing the file.

```bash
hakunadata --read file.flac
```

### Writing tags to a file

This will fetch metadata and write it to the file's tags.

```bash
hakunadata --write file.flac
```

### Processing a directory recursively

You can also provide a directory path to process all supported audio files (`.flac`, `.mp3`, `.ogg`, `.m4a`) within that directory and its subdirectories.

```bash
# Show remote tags for all files in a directory
hakunadata --read /path/to/music

# Write tags for all files in a directory
hakunadata --write /path/to/music
```

## API Credentials

The tool can use either Discogs or MusicBrainz.

-   **Discogs (Default, recommended):** For better results, especially for genre and label information, a Discogs token is recommended. You can get one from your Discogs developer settings.

    ```bash
    export DISCOGS_TOKEN="your_discogs_token_here"
    ```

-   **MusicBrainz (Fallback):** If the `DISCOGS_TOKEN` environment variable is not set, the tool will fall back to using the MusicBrainz API, which does not require authentication. Results might be less detailed.
