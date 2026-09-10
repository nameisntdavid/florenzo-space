// Single post page.

use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use askama::Template;

use crate::cache;
use crate::portable_text;
use crate::sanity::models::Post;
use crate::AppState;

#[derive(Template)]
#[template(path = "post.html")]
struct PostTemplate {
    post: Post,
    post_body_html: String,
    post_slug: String,
    reaction_counts: HashMap<String, i64>,
    sanity_project_id: String,
    sanity_dataset: String,
}

/// GET /post/:slug — renders a single post with Portable Text body and reaction counts.
pub async fn handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Response {
    let cache_key = cache::keys::post(&slug);
    let post: Option<Post> = cache::get_json(&state.redis, &cache_key)
        .await
        .unwrap_or(None);

    let post = match post {
        Some(p) => p,
        None => {
            match state.sanity.get_post(&slug).await {
                Ok(Some(p)) => {
                    let _ = cache::set_json(&state.redis, &cache_key, &p, 600).await;
                    p
                }
                Ok(None) => {
                    return (StatusCode::NOT_FOUND, Html("<h1>404 — Post not found</h1>".to_string())).into_response();
                }
                Err(e) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<h1>500 — Error fetching post</h1><pre>{}</pre>", e))).into_response();
                }
            }
        }
    };

    let post_body_html = post
        .body
        .as_deref()
        .map(|blocks| portable_text::render(blocks, state.sanity.project_id(), state.sanity.dataset()))
        .unwrap_or_default();

    let mut reaction_counts = HashMap::new();
    for emoji in ["fire", "heart", "rocket", "bulb", "eyes"] {
        let key = cache::keys::reaction(&slug, emoji);
        let count: i64 = cache::get(&state.redis, &key)
            .await
            .unwrap_or(None)
            .unwrap_or(0);
        reaction_counts.insert(emoji.to_string(), count);
    }

    let template = PostTemplate {
        post,
        post_body_html,
        post_slug: slug,
        reaction_counts,
        sanity_project_id: state.sanity.project_id().to_string(),
        sanity_dataset: state.sanity.dataset().to_string(),
    };

    (StatusCode::OK, Html(template.render().unwrap())).into_response()
}
