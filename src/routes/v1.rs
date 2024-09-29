use crate::certificate_data::CertificateData;
use crate::manager::ManagerType;
use crate::services::CertificateService;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};

/// Generate QR Code for Certificate issuance.
/// The user sends certificate data and receives a QR code to scan with the UniMe wallet.
#[utoipa::path(
    post,
    operation_id = "generate_certificate_qr",
    tag = "certificate_v1",
    path = "/api/v1/certificates/qr",
    request_body = CertificateData,
    responses(
        (status = 200, description = "QR Code generated successfully", body = String),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn send_certificate_and_get_qr(
    State(manager): State<ManagerType>,
    Json(certificate_data): Json<CertificateData>,
) -> Result<Json<String>, (StatusCode, String)> {
    let storage = manager.storage;
    let service = CertificateService::new(&storage);
    service
        .generate_qr_code(&certificate_data)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub fn create_router() -> Router<ManagerType> {
    log::info!("Creating router for /api/v1");
    Router::new().route("/certificates/qr", post(send_certificate_and_get_qr))
}
