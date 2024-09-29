use crate::certificate_data::CertificateData;
use crate::manager::ManagerType;
use crate::services::CertificateService;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::Value;
use url::Url;

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

pub async fn post_certificate_and_get_offer(
    State(manager): State<ManagerType>,
    Json(certificate_data): Json<CertificateData>,
) -> Result<impl IntoResponse, StatusCode> {
    log::info!("Certificate Data {:?}", certificate_data);
    let qr_url = manager
        .credential_offer_query(false)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Parse the URL and extract the credential_offer parameter
    let parsed_url = Url::parse(&qr_url).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let credential_offer = parsed_url
        .query_pairs()
        .find(|(key, _)| key == "credential_offer")
        .map(|(_, value)| value.into_owned())
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Decode and parse the credential offer JSON
    let decoded_offer =
        urlencoding::decode(&credential_offer).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let offer_json: Value =
        serde_json::from_str(&decoded_offer).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Extract the pre-authorized code
    let pre_authorized_code = offer_json["grants"]
        ["urn:ietf:params:oauth:grant-type:pre-authorized_code"]["pre-authorized_code"]
        .as_str()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Store the certificate data with the pre-authorized code
    manager
        .storage
        .store_certificate(pre_authorized_code.to_string(), certificate_data);

    Ok(Json(qr_url))
}

pub fn create_router() -> Router<ManagerType> {
    log::info!("Creating router for /api/v1");
    Router::new()
        .route("/certificates/qr", post(send_certificate_and_get_qr))
        .route("/certificates/offer", post(post_certificate_and_get_offer))
}
