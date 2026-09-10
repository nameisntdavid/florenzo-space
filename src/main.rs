// Application entry point.
//
// Loads configuration, initializes shared state (Sanity client + Redis connection),
// defines routes, and starts the HTTP server.

mod cache;
mod portable_text;
mod routes;
mod sanity;

use axum::body::Body;
use axum::http::{header, Response, StatusCode};
use axum::routing::get;
use axum::Router;
use fred::clients::Client;
use fred::types::{Builder, config::Config};
use rust_embed::Embed;
use tower_http::compression::CompressionLayer;

/// Shared application state passed to every route handler via Axum's `State` extractor.
#[derive(Clone)]
pub struct AppState {
    pub sanity: sanity::SanityClient,
    pub redis: Client,
}

#[derive(Embed)]
#[folder = "static"]
struct Asset;

/// Serves files embedded in the binary at compile time.
/// Works regardless of working directory — no filesystem access needed.
async fn static_handler(path: axum::extract::Path<String>) -> Response<Body> {
    let path = format!("{}",
        path.trim_start_matches('/')
    );

    match Asset::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path)
                .first_or_octet_stream()
                .to_string();
            Response::builder()
                .header(header::CONTENT_TYPE, mime)
                .body(Body::from(content.data.to_vec()))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap(),
    }
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let sanity_project_id =
        std::env::var("SANITY_PROJECT_ID").expect("SANITY_PROJECT_ID must be set");
    let sanity_dataset =
        std::env::var("SANITY_DATASET").unwrap_or_else(|_| "production".to_string());
    let sanity_api_version = "v1".to_string();
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    let sanity_client = sanity::SanityClient::new(
        sanity_project_id,
        sanity_dataset,
        sanity_api_version,
    );

    // Create Redis client from URL and initialize connection.
    let redis_config = Config::from_url(&redis_url).expect("Failed to parse Redis URL");
    let redis_client = Builder::from_config(redis_config).build()
        .expect("Failed to build Redis client");

    // Import the trait needed for init()
    use fred::prelude::ClientLike as _;
    redis_client.init()
        .await
        .expect("Failed to connect to Redis");

    let state = AppState {
        sanity: sanity_client,
        redis: redis_client,
    };

    let app = Router::new()
        .route("/", get(routes::home::handler))
        .route("/post/:slug", get(routes::post::handler))
        .route("/tag/:slug", get(routes::tag::handler))
        .route("/search", get(routes::search::handler))
        .route("/api/react", axum::routing::post(routes::react::handler))
        .route("/static/*path", get(static_handler))
        .layer(CompressionLayer::new())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
