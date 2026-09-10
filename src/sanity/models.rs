// Data models for Sanity content.
//
// Each struct maps to a Sanity document type or nested structure.
// `#[serde(rename_all = "camelCase")]` handles the JSON key conversion since
// Sanity uses camelCase while Rust convention is snake_case.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A published blog post.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub title: String,
    pub slug: Slug,
    pub excerpt: Option<String>,
    /// Portable Text blocks — Sanity's structured rich text format.
    pub body: Option<Vec<PtBlock>>,
    pub tags: Option<Vec<TagRef>>,
    pub published_at: Option<DateTime<Utc>>,
    pub main_image: Option<MainImage>,
}

impl Post {
    /// Build a Sanity CDN URL from the main image's asset reference.
    /// Format: https://cdn.sanity.io/images/{project}/{dataset}/{ref}-{width}x{height}.{ext}
    pub fn main_image_url(&self, project_id: &str, dataset: &str) -> Option<String> {
        let img = self.main_image.as_ref()?;
        let asset = img.asset.as_ref()?;
        let r#ref = asset._ref.as_ref()?;
        // ref format: "image-{hash}-{dims}-{ext}" — replace last dash with dot for extension
        let path = &r#ref[6..]; // skip "image-" prefix
        let url_path = match path.rfind('-') {
            Some(pos) => format!("{}.{}", &path[..pos], &path[pos + 1..]),
            None => path.to_string(),
        };
        Some(format!(
            "https://cdn.sanity.io/images/{}/{}/{}",
            project_id, dataset, url_path
        ))
    }
}

/// Sanity main image — an object with an asset reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainImage {
    pub asset: Option<PtAsset>,
}

/// Sanity slug — an object with a `current` field containing the URL-safe string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slug {
    pub current: String,
}

impl std::fmt::Display for Slug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.current)
    }
}

/// Inline reference to a tag (resolved via GROQ `->` dereference).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagRef {
    pub title: String,
    pub slug: Slug,
}

/// Standalone tag document.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
}

/// Portable Text block — a single node in Sanity's rich text tree.
///
/// Block types: "block" (text), "image", or custom types.
/// Text blocks contain inline `children` with optional formatting `marks`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PtBlock {
    #[serde(rename = "_type")]
    pub _type: String,
    pub style: Option<String>,
    pub children: Option<Vec<PtChild>>,
    #[serde(rename = "markDefs")]
    pub mark_defs: Option<Vec<PtMarkDef>>,
    pub asset: Option<PtAsset>,
    pub alt: Option<String>,
}

/// Inline text node within a Portable Text block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PtChild {
    #[serde(rename = "_type")]
    pub _type: String,
    pub text: Option<String>,
    /// Formatting marks: "strong", "em", "code", "underline", "link-{key}".
    pub marks: Option<Vec<String>>,
}

/// Mark definition — associates a mark key with data (e.g., a link's URL).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PtMarkDef {
    #[serde(rename = "_type")]
    pub _type: String,
    #[serde(rename = "_key")]
    pub _key: Option<String>,
    pub href: Option<String>,
}

/// Reference to a Sanity-hosted asset (image, file).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtAsset {
    #[serde(rename = "_ref")]
    pub _ref: Option<String>,
}

/// Pagination parameters for post listing queries.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PostQueryParams {
    pub page: u32,
    pub per_page: u32,
}

impl Default for PostQueryParams {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 10,
        }
    }
}
