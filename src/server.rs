use crate::storage::MemoryStorage;
use crate::v1;
use crate::{create_example_router, manager::ManagerType};
use anyhow::Result;
use axum::Router;
use did_key::{generate, Ed25519KeyPair};
use oid4vc_manager::{methods::key_method::KeySubject, servers::credential_issuer::Server};
use std::sync::Arc;

pub async fn start_server() -> Result<()> {
    // Create a key pair for the issuer
    let issuer_key = generate::<Ed25519KeyPair>(None);
    let issuer_subject = KeySubject::from_keypair(issuer_key, None);

    let listener = std::net::TcpListener::bind(std::net::SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 251)),
        3000,
    ))?;

    // Create a CredentialIssuerManager
    let credential_issuer_manager =
        ManagerType::new(Some(listener), MemoryStorage, Arc::new(issuer_subject))?;

    // Nest the API routes under "/api/v1"
    let app = Router::new()
        .nest("/api/v1", v1::create_router())
        .nest("/example", create_example_router());

    // Initialize the server with the app as the extension router
    let mut server = Server::setup(
        credential_issuer_manager,
        Some(app), // Pass the app without the `.with_state()` call
    )?;

    // Get the credential issuer URL
    let credential_issuer_url = server.credential_issuer_manager.credential_issuer_url()?;
    println!("Credential Issuer URL: {}", credential_issuer_url);

    // Start the server
    println!("Starting the Credential Issuer server...");
    server.start_server().await?;

    Ok(())
}
