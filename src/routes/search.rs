// Search page — full-text search across posts.

use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse};
use askama::Template;

use crate::sanity::models::Post;
use crate::AppState;

#[derive(serde::Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
}

#[derive(Template)]
#[template(path = "search.html")]
struct SearchTemplate {
    results: Vec<Post>,
    query: String,
}

/// GET /search?q=... — searches posts by title and excerpt.
pub async fn handler(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let query = params.q.unwrap_or_default();
    let results = if query.is_empty() {
        Vec::new()
    } else {
        state
            .sanity
            .search_posts(&query)
            .await
            .unwrap_or_default()
    };

    let template = SearchTemplate { results, query };
    Html(template.render().unwrap())
}
