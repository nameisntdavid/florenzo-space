// Search page — full-text search across posts.

use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse};
use askama::Template;

use crate::sanity::models::Post;
use crate::AppState;

#[derive(serde::Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
    pub view: Option<String>,
}

#[derive(Template)]
#[template(path = "search.html")]
struct SearchTemplate {
    results: Vec<Post>,
    query: String,
    view_mode: String,
    sanity_project_id: String,
    sanity_dataset: String,
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

    let view_mode = params.view.unwrap_or_else(|| "list".to_string());
    let template = SearchTemplate {
        results,
        query,
        view_mode,
        sanity_project_id: state.sanity.project_id().to_string(),
        sanity_dataset: state.sanity.dataset().to_string(),
    };
    Html(template.render().unwrap())
}
