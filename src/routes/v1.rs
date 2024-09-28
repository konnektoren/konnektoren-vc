use crate::services::{CertificateData, CertificateService};
use crate::storage::SharedStorage;
use axum::routing::post;
use axum::{extract::State, http::StatusCode, Json, Router};

/// Generate QR Code for Certificate issuance.
/// The user sends certificate data and receives a QR code to scan with the UniMe wallet.
#[utoipa::path(
    post,
    operation_id = "generate_certificate_qr",
    tag = "certificate_v1",
    path = "/api/v1/certificates/qr", // Update this line
    request_body = CertificateData,
    responses(
        (status = 200, description = "QR Code generated successfully", body = String),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn send_certificate_and_get_qr(
    State(storage): State<SharedStorage>,
    Json(certificate_data): Json<CertificateData>,
) -> Result<Json<String>, (StatusCode, String)> {
    let service = CertificateService::new(storage);
    service
        .generate_qr_code(&certificate_data)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub fn create_router() -> Router<SharedStorage> {
    log::info!("Creating router for /api/v1");
    Router::new().route("/certificates/qr", post(send_certificate_and_get_qr))
}
