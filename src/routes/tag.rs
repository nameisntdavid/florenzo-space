// Tag page — lists posts filtered by tag.

use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse};
use askama::Template;

use crate::cache;
use crate::sanity::models::Post;
use crate::AppState;

#[derive(Template)]
#[template(path = "tag.html")]
struct TagTemplate {
    posts: Vec<Post>,
    tag_name: String,
}

/// GET /tag/:slug — posts matching the given tag.
pub async fn handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let cache_key = cache::keys::tag(&slug);
    let posts: Vec<Post> = if let Some(cached) = cache::get_json(&state.redis, &cache_key)
        .await
        .unwrap_or(None)
    {
        cached
    } else {
        let posts = state
            .sanity
            .get_posts_by_tag(&slug)
            .await
            .unwrap_or_default();
        let _ = cache::set_json(&state.redis, &cache_key, &posts, 300).await;
        posts
    };

    let template = TagTemplate {
        posts,
        tag_name: slug,
    };

    Html(template.render().unwrap())
}
