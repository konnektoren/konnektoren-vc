use crate::services::{CertificateData, CertificateService};
use crate::storage::SharedStorage;
use axum::extract::Path;
use axum::routing::{get, post};
use axum::{extract::State, http::StatusCode, Json, Router};
use serde_json::json;

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

/// Retrieve a specific credential offer.
#[utoipa::path(
    get,
    operation_id = "get_credential_offer",
    tag = "certificate_v1",
    path = "/api/v1/offers/{offer_id}",
    params(
        ("offer_id" = String, Path, description = "The unique identifier for the credential offer")
    ),
    responses(
        (status = 200, description = "Credential offer retrieved successfully", body = String),
        (status = 404, description = "Credential offer not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_credential_offer(
    State(storage): State<SharedStorage>,
    Path(offer_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let service = CertificateService::new(storage);

    match service.retrieve_credential_offer(&offer_id) {
        Ok(Some(offer_data)) => {
            let offer = json!({
                "credential_issuer": service.get_issuer_url(),
                "credentials": ["PersonalInformation_JWT"],
                "grants": {
                    "authorization_code": {
                        "issuer_state": offer_id
                    }
                }
            });
            Ok(Json(offer))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            "Credential offer not found".to_string(),
        )),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub fn create_router() -> Router<SharedStorage> {
    log::info!("Creating router for /api/v1");
    Router::new()
        .route("/certificates/qr", post(send_certificate_and_get_qr))
        .route("/offers/:offer_id", get(get_credential_offer))
}
