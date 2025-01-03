use crate::manager::ManagerType;
use axum::routing::get_service;
use axum::Router;
use std::path::PathBuf;
use tower_http::services::ServeDir;

#[tracing::instrument]
pub fn create_router() -> Router<ManagerType> {
    let assets_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    log::debug!("Serving assets from: {:?} in /assets", assets_path);
    Router::new().nest_service("/assets", get_service(ServeDir::new(assets_path)))
}
