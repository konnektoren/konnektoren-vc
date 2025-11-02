use crate::manager::ManagerType;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::Json;
use axum::Router;
use oid4vc_manager::storage::Storage;
use oid4vci::credential_format_profiles::{CredentialFormats, WithParameters}; // Add this import
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize)]
pub struct ReadinessResponse {
    pub status: String,
    pub checks: HealthChecks,
}

#[derive(Serialize, Deserialize)]
pub struct HealthChecks {
    pub storage: String,
    pub did_generation: String,
    pub credential_issuer: String,
}

/// Health check endpoint - always returns OK if the service is running
#[tracing::instrument]
pub async fn health_check() -> Result<Json<HealthResponse>, StatusCode> {
    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}

/// Readiness check endpoint - checks if the service is ready to serve traffic
#[tracing::instrument(skip(manager))]
pub async fn readiness_check(
    State(manager): State<ManagerType>,
) -> Result<Json<ReadinessResponse>, (StatusCode, Json<ReadinessResponse>)> {
    let mut checks = HealthChecks {
        storage: "unknown".to_string(),
        did_generation: "unknown".to_string(),
        credential_issuer: "unknown".to_string(),
    };

    let mut all_healthy = true;

    // Check storage
    match test_storage(&manager).await {
        Ok(_) => checks.storage = "healthy".to_string(),
        Err(e) => {
            log::error!("Storage check failed: {}", e);
            checks.storage = "unhealthy".to_string();
            all_healthy = false;
        }
    }

    // Check DID generation
    match test_did_generation().await {
        Ok(_) => checks.did_generation = "healthy".to_string(),
        Err(e) => {
            log::error!("DID generation check failed: {}", e);
            checks.did_generation = "unhealthy".to_string();
            all_healthy = false;
        }
    }

    // Check credential issuer
    match test_credential_issuer(&manager).await {
        Ok(_) => checks.credential_issuer = "healthy".to_string(),
        Err(e) => {
            log::error!("Credential issuer check failed: {}", e);
            checks.credential_issuer = "unhealthy".to_string();
            all_healthy = false;
        }
    }

    let response = ReadinessResponse {
        status: if all_healthy { "ready" } else { "not_ready" }.to_string(),
        checks,
    };

    if all_healthy {
        Ok(Json(response))
    } else {
        Err((StatusCode::SERVICE_UNAVAILABLE, Json(response)))
    }
}

async fn test_storage(
    manager: &ManagerType,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::certificate_data::CertificateData;
    use chrono::Utc;
    use uuid::Uuid;

    // Test basic storage operations
    let test_id = Uuid::new_v4().to_string();
    let test_certificate = CertificateData {
        game_path_name: "health_check".to_string(),
        total_challenges: 1,
        solved_challenges: 1,
        performance_percentage: 100,
        profile_name: "health_check".to_string(),
        date: Utc::now(),
    };

    // Test store and retrieve
    manager
        .storage
        .store_certificate(test_id.clone(), test_certificate);

    match manager.storage.get_certificate(&test_id) {
        Some(_) => Ok(()),
        None => Err("Failed to retrieve stored certificate".into()),
    }
}

async fn test_did_generation() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::config::load_config;
    use did_key::{generate, DIDCore, Ed25519KeyPair, PatchedKeyPair};

    let (priv_key, _) = load_config();
    let issuer_key: PatchedKeyPair = generate::<Ed25519KeyPair>(Some(priv_key.as_bytes()));
    let _document = issuer_key.get_did_document(did_key::Config::default());

    Ok(())
}

async fn test_credential_issuer(
    manager: &ManagerType,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use oid4vci::credential_issuer::credential_configurations_supported::CredentialConfigurationsSupportedObject;
    use std::collections::HashMap;

    // Test credential issuer URL generation
    let _url = manager.credential_issuer_url()?;

    // Test credential configurations with explicit type annotation
    let configs: HashMap<
        String,
        CredentialConfigurationsSupportedObject<CredentialFormats<WithParameters>>,
    > = manager.storage.get_credential_configurations_supported();

    if configs.is_empty() {
        return Err("No credential configurations available".into());
    }

    Ok(())
}

pub fn create_router() -> Router<ManagerType> {
    log::info!("Creating health check router");
    Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
}
