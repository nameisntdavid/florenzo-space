// Emoji reaction endpoint.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Html;
use serde::Deserialize;

use crate::cache;
use crate::AppState;

#[derive(Deserialize)]
pub struct ReactParams {
    pub slug: String,
    pub emoji: String,
}

/// POST /api/react — increments reaction count, returns updated button HTML.
pub async fn handler(
    State(state): State<AppState>,
    axum::Form(params): axum::Form<ReactParams>,
) -> Result<Html<String>, StatusCode> {
    let key = cache::keys::reaction(&params.slug, &params.emoji);

    let new_count = cache::increment(&state.redis, &key, 1)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let display_emoji = match params.emoji.as_str() {
        "fire" => "🔥",
        "heart" => "❤️",
        "rocket" => "🚀",
        "bulb" => "💡",
        "eyes" => "👀",
        _ => "👍",
    };

    Ok(Html(format!(
        r#"<button class="reaction-btn" hx-post="/api/react" hx-vals='{{"slug":"{}","emoji":"{}"}}' hx-target="this" hx-swap="outerHTML"><span class="reaction-emoji">{}</span> <span class="reaction-count">{}</span></button>"#,
        params.slug, params.emoji, display_emoji, new_count
    )))
}
