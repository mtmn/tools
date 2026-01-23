use anyhow::{Context, Result};
use reqwest::header;
use serde::Deserialize;

const USER_AGENT: &str = "hakunadata/0.1.0 ( miro@haravara.org )";

pub struct MusicBrainzClient {
    client: reqwest::Client,
}

impl MusicBrainzClient {
    pub fn new() -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(USER_AGENT),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self { client })
    }

    pub async fn fetch_genres(&self, artist: &str, release: &str) -> Result<Vec<String>> {
        // First search for the release group to get a broader set of tags, or specific release.
        // Let's try searching for "release" first as it is more specific, but release-group often has the tags.
        // Actually, searching for release-group is usually better for genres as they adhere to the abstract album.

        let query = format!("artist:\"{artist}\" AND release:\"{release}\"");
        let url = "https://musicbrainz.org/ws/2/release";

        let response = self
            .client
            .get(url)
            .query(&[
                ("query", query.as_str()),
                ("fmt", "json"),
                ("limit", "1"), // Just take the best match
            ])
            .send()
            .await
            .context("Failed to send MusicBrainz request")?;

        if !response.status().is_success() {
            // It's okay if we don't find it, but we should log it?
            // For now just return empty.
            return Ok(vec![]);
        }

        let search_result: MbSearchResponse = response.json().await?;

        // If we found a release, let's fetch its group to get tags, OR if the release itself has tags (usually not in search result).
        // The search result usually contains some info. But to get tags we might need to look up.
        // Wait, the search result for 'release' DOES return a list of releases.
        // If we want tags, we often need to include 'tags' in an 'inc' parameter for a direct lookup, but for search we get what we get.
        // Actually, MB search API returns tags if present? Usually not.
        // Better flow:
        // 1. Search for release-group (or release).
        // 2. Get ID.
        // 3. Lookup release-group with inc=tags.

        if let Some(release_match) = search_result.releases.first() {
            // If we have a release-group ID, use that.
            if let Some(rg) = &release_match.release_group {
                return self.lookup_release_group_tags(&rg.id).await;
            }
        }

        Ok(vec![])
    }

    async fn lookup_release_group_tags(&self, id: &str) -> Result<Vec<String>> {
        let url = format!("https://musicbrainz.org/ws/2/release-group/{id}");
        let response = self
            .client
            .get(&url)
            .query(&[("fmt", "json"), ("inc", "tags+genres")])
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let rg: MbReleaseGroup = response.json().await?;
        let mut genres = Vec::new();

        if let Some(tags) = rg.tags {
            for tag in tags {
                // Filter out garbage tags? maybe later.
                genres.push(tag.name);
            }
        }
        if let Some(genres_list) = rg.genres {
            for g in genres_list {
                genres.push(g.name);
            }
        }

        Ok(genres)
    }
}

// --- Serde Structs ---

#[derive(Deserialize, Debug)]
struct MbSearchResponse {
    #[serde(default)]
    releases: Vec<MbRelease>,
}

#[derive(Deserialize, Debug)]
struct MbRelease {
    #[serde(rename = "release-group")]
    release_group: Option<MbReleaseGroupRef>,
}

#[derive(Deserialize, Debug)]
struct MbReleaseGroupRef {
    id: String,
}

#[derive(Deserialize, Debug)]
struct MbReleaseGroup {
    tags: Option<Vec<MbTag>>,
    genres: Option<Vec<MbTag>>,
}

#[derive(Deserialize, Debug)]
struct MbTag {
    name: String,
}
