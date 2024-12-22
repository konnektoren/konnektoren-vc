use crate::config::Config;
use anyhow::Result;
use oid4vc_core::Subject;
use oid4vc_manager::managers::credential_issuer::CredentialIssuerManager;
use oid4vc_manager::storage::Storage;
use oid4vci::credential_format_profiles::CredentialFormatCollection;
use oid4vci::credential_issuer::authorization_server_metadata::AuthorizationServerMetadata;
use oid4vci::credential_issuer::credential_issuer_metadata::CredentialIssuerMetadata;
use oid4vci::credential_issuer::CredentialIssuer;
use serde_json::json;
use std::net::TcpListener;
use std::sync::Arc;
use url::Url;

pub trait ConfigurableManager<S, CFC> {
    fn with_config(
        listener: Option<TcpListener>,
        storage: S,
        subject: Arc<dyn Subject>,
        config: Config,
    ) -> Result<Self>
    where
        Self: Sized;
}

impl<S, CFC> ConfigurableManager<S, CFC> for CredentialIssuerManager<S, CFC>
where
    S: Storage<CFC>,
    CFC: CredentialFormatCollection,
{
    fn with_config(
        listener: Option<TcpListener>,
        storage: S,
        subject: Arc<dyn Subject>,
        config: Config,
    ) -> Result<Self> {
        let listener = listener.unwrap_or_else(|| TcpListener::bind("127.0.0.1:0").unwrap());
        let issuer_url: Url = match config.issuer_url {
            Some(url) => url.parse()?,
            None => format!("http://{:?}", listener.local_addr()?).parse()?,
        };
        Ok(Self {
            credential_issuer: CredentialIssuer {
                subject: subject.clone(),
                metadata: CredentialIssuerMetadata {
                    credential_issuer: issuer_url.clone(),
                    authorization_servers: vec![issuer_url.clone()],
                    credential_endpoint: issuer_url.join("/credential")?,
                    batch_credential_endpoint: Some(issuer_url.join("/batch_credential")?),
                    deferred_credential_endpoint: None,
                    notification_endpoint: None,
                    credential_response_encryption: None,
                    credential_identifiers_supported: None,
                    signed_metadata: None,
                    display: Some(vec![json!({
                        "name": "Konnektoren Credential Issuer",
                        "locale": "en-US",
                        "logo": {
                            "uri": "https://konnektoren.help/favicon.png",
                            "alt_text": "konnektoren.help square logo"
                        },
                        "background_color": "#ff7e00",
                        "text_color": "#6200ea"
                    })]),
                    credential_configurations_supported: storage
                        .get_credential_configurations_supported(),
                },
                authorization_server_metadata: AuthorizationServerMetadata {
                    issuer: issuer_url.clone(),
                    authorization_endpoint: Some(issuer_url.join("/authorize")?),
                    token_endpoint: Some(issuer_url.join("/token")?),
                    pre_authorized_grant_anonymous_access_supported: Some(true),
                    ..Default::default()
                },
            },
            subject,
            storage,
            listener: Arc::new(listener),
        })
    }
}
