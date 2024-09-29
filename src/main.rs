use axum::body::Body;
use axum::extract::OriginalUri;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use axum::Router;
use dotenv::dotenv;
use konnektoren_vc::prelude::*;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

async fn handle_404(OriginalUri(uri): OriginalUri, req: Request<Body>) -> impl IntoResponse {
    // Log the details of the 404 request
    log::info!("404 Not Found: {} {}", req.method(), uri);

    // Return a 404 response
    (StatusCode::NOT_FOUND, "404 Not Found")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    dotenv().ok();

    let trace_layer = TraceLayer::new_for_http()
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::DEBUG));

    start_server().await.expect("Failed to start server.");
}
