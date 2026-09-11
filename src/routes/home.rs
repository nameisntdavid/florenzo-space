// Home page — landing with hero and recent posts grid.

use axum::extract::State;
use axum::response::{Html, IntoResponse};
use askama::Template;

use crate::cache;
use crate::sanity::models::Post;
use crate::AppState;

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    posts: Vec<Post>,
    sanity_project_id: String,
    sanity_dataset: String,
}

/// GET / — renders hero + recent posts grid.
pub async fn handler(State(state): State<AppState>) -> impl IntoResponse {
    let cache_key = cache::keys::posts_page(1);
    let posts: Vec<Post> = if let Some(cached) = cache::get_json(&state.redis, &cache_key)
        .await
        .unwrap_or(None)
    {
        cached
    } else {
        let posts = state
            .sanity
            .get_posts(1, 6)
            .await
            .map_err(|e| eprintln!("Sanity error: {}", e))
            .unwrap_or_default();
        let _ = cache::set_json(&state.redis, &cache_key, &posts, 300).await;
        posts
    };

    let template = HomeTemplate {
        posts,
        sanity_project_id: state.sanity.project_id().to_string(),
        sanity_dataset: state.sanity.dataset().to_string(),
    };

    Html(template.render().unwrap())
}
