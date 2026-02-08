use anyhow::{Context, Result};
use reqwest::header;
use serde::Deserialize;

const USER_AGENT: &str = "hakunadata/0.1.0 ( miro@haravara.org )";

pub struct DiscogsClient {
    client: reqwest::Client,
    token: Option<String>,
}

impl DiscogsClient {
    pub fn new() -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(USER_AGENT),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        let token = std::env::var("DISCOGS_TOKEN").ok();

        Ok(Self { client, token })
    }

    pub async fn fetch_metadata(
        &self,
        artist: &str,
        release: &str,
    ) -> Result<Option<DiscogsResult>> {
        let url = "https://api.discogs.com/database/search";
        let mut query = vec![
            ("type", "release"),
            ("artist", artist),
            ("release_title", release),
        ];

        let token_string;
        if let Some(t) = &self.token {
            token_string = t.clone();
            query.push(("token", &token_string));
        }

        let response = self
            .client
            .get(url)
            .query(&query)
            .send()
            .await
            .context("Failed to send Discogs request")?;

        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::UNAUTHORIZED {
                use std::sync::atomic::{AtomicBool, Ordering};
                static WARNED: AtomicBool = AtomicBool::new(false);
                if !WARNED.swap(true, Ordering::Relaxed) {
                    eprintln!("DISCOGS_TOKEN is required");
                }
            }
            return Ok(None);
        }

        let search_result: DiscogsSearchResponse = response.json().await?;

        Ok(search_result.results.into_iter().next())
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct DiscogsSearchResponse {
    pub results: Vec<DiscogsResult>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DiscogsResult {
    pub genre: Option<Vec<String>>,
    pub style: Option<Vec<String>>,
    pub label: Option<Vec<String>>,
}
