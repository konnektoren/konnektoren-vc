use crate::config::{load_config, Config};
use crate::manager::ConfigurableManager;
use crate::storage::MemoryStorage;
use crate::{assets, create_example_router, manager::ManagerType};
use crate::{v1, well_known};
use anyhow::Result;
use axum::Router;
use did_key::{generate, DIDCore, Ed25519KeyPair, PatchedKeyPair};
use oid4vc_manager::{methods::key_method::KeySubject, servers::credential_issuer::Server};
use std::sync::Arc;

pub async fn start_server() -> Result<()> {
    let (priv_key, _) = load_config();

    let issuer_key: PatchedKeyPair = generate::<Ed25519KeyPair>(Some(priv_key.as_bytes()));

    let document = issuer_key.get_did_document(did_key::Config::default());

    let issuer_subject = KeySubject::from_keypair(issuer_key, None);

    log::debug!("Issuer Subject: {:?}", document);

    let listener = std::net::TcpListener::bind(std::net::SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
        3000,
    ))?;

    let config = Config::default();

    // Create a CredentialIssuerManager
    let credential_issuer_manager = ManagerType::with_config(
        Some(listener),
        MemoryStorage::new(),
        Arc::new(issuer_subject),
        config,
    )?;

    // Nest the API routes under "/api/v1"
    let app = Router::new()
        .nest("/api/v1", v1::create_router())
        .nest("/example", create_example_router())
        .nest("/.well-known", well_known::create_router())
        .nest("/", assets::create_router());

    // Initialize the server with the app as the extension router
    let mut server = Server::setup(credential_issuer_manager, Some(app))?;

    // Get the credential issuer URL
    let credential_issuer_url = server.credential_issuer_manager.credential_issuer_url()?;
    println!("Credential Issuer URL: {}", credential_issuer_url);

    // Start the server
    println!("Starting the Credential Issuer server...");
    server.start_server().await?;

    Ok(())
}
