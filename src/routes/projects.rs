// Projects page — placeholder.

use axum::response::{Html, IntoResponse};
use askama::Template;

#[derive(Template)]
#[template(path = "projects.html")]
struct ProjectsTemplate;

/// GET /projects — renders placeholder projects page.
pub async fn handler() -> impl IntoResponse {
    Html(ProjectsTemplate.render().unwrap())
}
