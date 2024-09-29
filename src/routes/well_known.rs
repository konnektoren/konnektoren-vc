use crate::config::load_config;
use crate::manager::ManagerType;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use did_key::{generate, DIDCore, Document, Ed25519KeyPair, PatchedKeyPair};

pub async fn get_did_document() -> Result<Json<Document>, (StatusCode, String)> {
    let (priv_key, _) = load_config();

    let issuer_key: PatchedKeyPair = generate::<Ed25519KeyPair>(Some(priv_key.as_bytes()));

    let document = issuer_key.get_did_document(did_key::Config::default());

    Ok(Json(document))
}

pub fn create_router() -> Router<ManagerType> {
    log::info!("Creating router for /.well-known/did.json");
    Router::new().route("/did.json", get(get_did_document))
}
