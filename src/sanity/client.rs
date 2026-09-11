// Sanity Content Lake HTTP client.
//
// Sanity exposes a REST API at:
//   GET https://{projectId}.api.sanity.io/v{version}/data/query/{dataset}?query={groq}
//
// This client wraps that endpoint with typed GROQ queries and deserialization.

use std::collections::HashMap;

use reqwest::Client;
use serde::Deserialize;

use super::models::*;

/// HTTP client for querying Sanity's Content Lake via GROQ.
#[derive(Clone)]
pub struct SanityClient {
    project_id: String,
    dataset: String,
    api_version: String,
    http: Client,
}

/// Sanity wraps all query responses in `{ "result": <data> }`.
#[derive(Deserialize)]
struct SanityResponse<T> {
    result: T,
}

impl SanityClient {
    pub fn new(project_id: String, dataset: String, api_version: String) -> Self {
        Self {
            project_id,
            dataset,
            api_version,
            http: Client::new(),
        }
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn dataset(&self) -> &str {
        &self.dataset
    }

    /// Execute a raw GROQ query string with optional parameters.
    ///
    /// Generic over `T` — any type that implements `Deserialize` (e.g., `Vec<Post>`, `Post`, `u64`).
    /// Returns `Err(String)` on network or deserialization failure.
    pub async fn query<T: for<'de> Deserialize<'de>>(
        &self,
        groq: &str,
        params: Option<HashMap<String, String>>,
    ) -> Result<T, String> {
        let url = format!(
            "https://{}.api.sanity.io/{}/data/query/{}",
            self.project_id, self.api_version, self.dataset,
        );

        let mut request = self.http.get(&url).query(&[("query", &groq)]);

        if let Some(params) = params {
            for (key, value) in &params {
                request = request.query(&[(&key, &value)]);
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(format!("HTTP {}: {}", status, body));
        }

        let sanity_response: SanityResponse<T> = serde_json::from_str(&body)
            .map_err(|e| format!("JSON parse failed: {} | body: {}", e, &body[..body.len().min(500)]))?;

        Ok(sanity_response.result)
    }

    /// Fetch paginated posts ordered by publication date (newest first).
    pub async fn get_posts(&self, page: u32, per_page: u32) -> Result<Vec<Post>, String> {
        let offset = (page - 1) * per_page;
        let end = page * per_page;

        let groq = format!(
            r#"*[_type == "post"] | order(publishedAt desc)[{}..{}] {{title, slug, excerpt, publishedAt, mainImage, tags[]->{{title, slug}}}}"#,
            offset, end,
        );

        self.query::<Vec<Post>>(&groq, None).await
    }

    /// Fetch a single post by its slug. Returns `None` if no match.
    pub async fn get_post(&self, slug: &str) -> Result<Option<Post>, String> {
        let groq = format!(
            r#"*[_type == "post" && slug.current == "{}"][0] {{title, slug, excerpt, body, publishedAt, mainImage, tags[]->{{title, slug}}}}"#,
            slug,
        );

        self.query::<Option<Post>>(&groq, None).await
    }

    /// Fetch all posts tagged with the given tag slug.
    pub async fn get_posts_by_tag(&self, tag: &str) -> Result<Vec<Post>, String> {
        let groq = format!(
            r#"*[_type == "post" && "{}" in tags[]->slug.current] | order(publishedAt desc) {{title, slug, excerpt, publishedAt, mainImage, tags[]->{{title, slug}}}}"#,
            tag,
        );

        self.query::<Vec<Post>>(&groq, None).await
    }

    /// Full-text search across post titles and excerpts.
    pub async fn search_posts(&self, query: &str) -> Result<Vec<Post>, String> {
        let groq = format!(
            r#"*[_type == "post" && (title match "{}*" || excerpt match "{}*")] | order(publishedAt desc) {{title, slug, excerpt, publishedAt, mainImage, tags[]->{{title, slug}}}}"#,
            query, query,
        );

        self.query::<Vec<Post>>(&groq, None).await
    }

    /// Fetch all tags ordered by title.
    pub async fn get_tags(&self) -> Result<Vec<Tag>, String> {
        let groq = r#"*[_type == "tag"] | order(title asc) {title, "slug": slug.current, description}"#;
        self.query::<Vec<Tag>>(groq, None).await
    }

    /// Count total published posts (used for pagination math).
    pub async fn count_posts(&self) -> Result<u64, String> {
        self.query(r#"count(*[_type == "post"])"#, None).await
    }

    /// Count posts for a specific tag.
    pub async fn count_posts_by_tag(&self, tag: &str) -> Result<u64, String> {
        let groq = format!(
            r#"count(*[_type == "post" && "{}" in tags[]->slug.current])"#,
            tag,
        );

        self.query::<u64>(&groq, None).await
    }
}
