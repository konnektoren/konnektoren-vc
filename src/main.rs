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

    let repo: SharedStorage = new_shared_storage();

    let trace_layer = TraceLayer::new_for_http()
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::DEBUG));

    let app = Router::new()
        .nest("/api/v1", create_router())
        .nest("/example", create_example_router())
        .with_state(repo)
        .layer(trace_layer)
        .layer(CorsLayer::permissive())
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .fallback(handle_404);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    log::info!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
