use crate::certificate_data::CertificateData;
use crate::manager::ManagerType;
use crate::services::CertificateService;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};

pub async fn send_certificate_and_get_qr(
    State(manager): State<ManagerType>,
    Json(certificate_data): Json<CertificateData>,
) -> Result<Json<String>, (StatusCode, String)> {
    let service = CertificateService::new(&manager);
    service
        .generate_qr_code(&certificate_data)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn post_certificate_and_get_offer(
    State(manager): State<ManagerType>,
    Json(certificate_data): Json<CertificateData>,
) -> Result<impl IntoResponse, StatusCode> {
    let service = CertificateService::new(&manager);
    service
        .generate_offer_url(&certificate_data)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn create_router() -> Router<ManagerType> {
    log::info!("Creating router for /api/v1");
    Router::new()
        .route("/certificates/qr", post(send_certificate_and_get_qr))
        .route("/certificates/offer", post(post_certificate_and_get_offer))
}
