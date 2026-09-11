// Tag page — lists posts filtered by tag.

use axum::extract::{Path, Query, State};
use axum::response::{Html, IntoResponse};
use askama::Template;

use crate::cache;
use crate::sanity::models::Post;
use crate::AppState;

#[derive(serde::Deserialize)]
pub struct ViewParams {
    pub view: Option<String>,
}

#[derive(Template)]
#[template(path = "tag.html")]
struct TagTemplate {
    posts: Vec<Post>,
    tag_name: String,
    view_mode: String,
    sanity_project_id: String,
    sanity_dataset: String,
}

/// GET /tag/:slug — posts matching the given tag.
pub async fn handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(params): Query<ViewParams>,
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

    let view_mode = params.view.unwrap_or_else(|| "list".to_string());
    let template = TagTemplate {
        posts,
        tag_name: slug,
        view_mode,
        sanity_project_id: state.sanity.project_id().to_string(),
        sanity_dataset: state.sanity.dataset().to_string(),
    };

    Html(template.render().unwrap())
}
