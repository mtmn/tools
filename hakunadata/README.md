# hakunadata
hakunadata is a simple command-line tool that fetches music metadata (genres and labels) from Discogs or MusicBrainz.

```bash
DISCOGS_TOKEN="your_token" hakunadata "artist" "album"
```

> [!NOTE]
> In case `DISCOGS_TOKEN` is not set it falls back to MusicBrainz which is sufficient in _most_ cases.
