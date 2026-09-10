// Home page — paginated post listing.

use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse};
use askama::Template;

use crate::cache;
use crate::sanity::models::Post;
use crate::AppState;

#[derive(serde::Deserialize)]
pub struct HomeParams {
    pub page: Option<u32>,
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    posts: Vec<Post>,
    page: u32,
    total_pages: u32,
    sanity_project_id: String,
    sanity_dataset: String,
}

/// GET / — renders paginated post list.
pub async fn handler(
    State(state): State<AppState>,
    Query(params): Query<HomeParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1);
    let per_page = 10u32;

    let cache_key = cache::keys::posts_page(page);
    let posts: Vec<Post> = if let Some(cached) = cache::get_json(&state.redis, &cache_key)
        .await
        .unwrap_or(None)
    {
        cached
    } else {
        let posts = state
            .sanity
            .get_posts(page, per_page)
            .await
            .map_err(|e| eprintln!("Sanity error: {}", e))
            .unwrap_or_default();
        let _ = cache::set_json(&state.redis, &cache_key, &posts, 300).await;
        posts
    };

    let total_count = state.sanity.count_posts().await.unwrap_or(0);
    let total_pages = ((total_count as f64) / (per_page as f64)).ceil() as u32;

    let template = HomeTemplate {
        posts,
        page,
        total_pages,
        sanity_project_id: state.sanity.project_id().to_string(),
        sanity_dataset: state.sanity.dataset().to_string(),
    };

    Html(template.render().unwrap())
}
