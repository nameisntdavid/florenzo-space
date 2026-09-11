// Blogs page — full post listing with search and tag filters.

use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse};
use askama::Template;

use crate::cache;
use crate::sanity::models::{Post, Tag};
use crate::AppState;

#[derive(serde::Deserialize)]
pub struct BlogsParams {
    pub page: Option<u32>,
    pub q: Option<String>,
    pub tag: Option<String>,
}

#[derive(Template)]
#[template(path = "blogs.html")]
struct BlogsTemplate {
    posts: Vec<Post>,
    all_tags: Vec<Tag>,
    query: String,
    active_tag: Option<String>,
    sanity_project_id: String,
    sanity_dataset: String,
}

/// GET /blogs — renders post listing with optional search and tag filter.
pub async fn handler(
    State(state): State<AppState>,
    Query(params): Query<BlogsParams>,
) -> impl IntoResponse {
    let _page = params.page.unwrap_or(1);
    let query = params.q.unwrap_or_default();
    let active_tag = params.tag;

    // Fetch all posts (we'll filter client-side for now)
    let cache_key = cache::keys::posts_page(1);
    let all_posts: Vec<Post> = if let Some(cached) = cache::get_json(&state.redis, &cache_key)
        .await
        .unwrap_or(None)
    {
        cached
    } else {
        let posts = state
            .sanity
            .get_posts(1, 100)
            .await
            .map_err(|e| eprintln!("Sanity error: {}", e))
            .unwrap_or_default();
        let _ = cache::set_json(&state.redis, &cache_key, &posts, 300).await;
        posts
    };

    // Filter by search query
    let posts: Vec<Post> = if !query.is_empty() {
        let q = query.to_lowercase();
        all_posts
            .into_iter()
            .filter(|p| p.title.to_lowercase().contains(&q))
            .collect()
    } else {
        all_posts
    };

    // Filter by tag
    let posts: Vec<Post> = if let Some(ref tag_slug) = active_tag {
        posts
            .into_iter()
            .filter(|p| {
                p.tags
                    .as_ref()
                    .map(|tags| tags.iter().any(|t| t.slug.current == **tag_slug))
                    .unwrap_or(false)
            })
            .collect()
    } else {
        posts
    };

    // Fetch all tags for filter chips
    let tags_cache_key = cache::keys::tags_all();
    let all_tags: Vec<Tag> = if let Some(cached) =
        cache::get_json(&state.redis, &tags_cache_key)
            .await
            .unwrap_or(None)
    {
        cached
    } else {
        let tags = state
            .sanity
            .get_tags()
            .await
            .map_err(|e| eprintln!("Sanity error: {}", e))
            .unwrap_or_default();
        let _ = cache::set_json(&state.redis, &tags_cache_key, &tags, 300).await;
        tags
    };

    let template = BlogsTemplate {
        posts,
        all_tags,
        query,
        active_tag,
        sanity_project_id: state.sanity.project_id().to_string(),
        sanity_dataset: state.sanity.dataset().to_string(),
    };

    Html(template.render().unwrap())
}
