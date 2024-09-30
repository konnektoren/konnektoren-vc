use std::{collections::HashMap, fs::File};

use crate::config::load_config;
use did_key::{generate, DIDCore, Document, Ed25519KeyPair, PatchedKeyPair};
use futures::executor::block_on;
use jsonwebtoken::{Algorithm, Header};
use lazy_static::lazy_static;
use oid4vc_core::{authentication::subject::SigningSubject, generate_authorization_code, jwt};
use oid4vc_manager::storage::Storage;
use oid4vci::{
    authorization_response::AuthorizationResponse,
    credential_format_profiles::{CredentialFormatCollection, CredentialFormats, WithParameters},
    credential_issuer::credential_configurations_supported::CredentialConfigurationsSupportedObject,
    credential_offer::{AuthorizationCode, PreAuthorizedCode},
    credential_response::{CredentialResponse, CredentialResponseType},
    token_request::TokenRequest,
    token_response::TokenResponse,
    VerifiableCredentialJwt,
};
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde_json::json;

lazy_static! {
    pub static ref C_NONCE: String = "tZignsnFbp".to_string();
}

fn generate_pre_authorized_code() -> PreAuthorizedCode {
    PreAuthorizedCode {
        pre_authorized_code: generate_authorization_code(16),
        ..Default::default()
    }
}

use crate::certificate_data::CertificateData;
use std::sync::{Arc, Mutex};
use types_ob_v3::prelude::AchievementCredential;
use uuid::Uuid;

#[derive(Clone)]
pub struct MemoryStorage {
    certificates: Arc<Mutex<HashMap<String, CertificateData>>>,
    pre_authorized_codes: Arc<Mutex<HashMap<String, String>>>, // pre-authorized code -> certificate id
    access_tokens: Arc<Mutex<HashMap<String, String>>>,        // access token -> certificate id
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            certificates: Arc::new(Mutex::new(HashMap::new())),
            pre_authorized_codes: Arc::new(Mutex::new(HashMap::new())),
            access_tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn store_certificate(&self, certificate_id: String, certificate_data: CertificateData) {
        log::info!("Storing certificate with id: {}", certificate_id);
        let mut certificates = self.certificates.lock().unwrap();
        certificates.insert(certificate_id, certificate_data);
    }

    pub fn get_certificate(&self, certificate_id: &str) -> Option<CertificateData> {
        let certificates = self.certificates.lock().unwrap();
        certificates.get(certificate_id).cloned()
    }

    pub fn associate_pre_authorized_code(
        &self,
        pre_authorized_code: String,
        certificate_id: String,
    ) {
        log::info!(
            "Associating pre-authorized code {} with certificate id: {}",
            pre_authorized_code,
            certificate_id
        );
        let mut codes = self.pre_authorized_codes.lock().unwrap();
        codes.insert(pre_authorized_code, certificate_id);
    }

    pub fn associate_access_token(&self, access_token: String, certificate_id: String) {
        log::info!(
            "Associating access token {} with certificate id: {}",
            access_token,
            certificate_id
        );
        let mut tokens = self.access_tokens.lock().unwrap();
        tokens.insert(access_token, certificate_id);
    }

    pub fn get_certificate_id_by_pre_authorized_code(
        &self,
        pre_authorized_code: &str,
    ) -> Option<String> {
        let codes = self.pre_authorized_codes.lock().unwrap();
        codes.get(pre_authorized_code).cloned()
    }

    pub fn get_certificate_id_by_access_token(&self, access_token: &str) -> Option<String> {
        let tokens = self.access_tokens.lock().unwrap();
        tokens.get(access_token).cloned()
    }
}

impl<CFC: CredentialFormatCollection + DeserializeOwned> Storage<CFC> for MemoryStorage {
    fn get_credential_configurations_supported(
        &self,
    ) -> HashMap<String, CredentialConfigurationsSupportedObject<CFC>> {
        log::debug!("get_credential_configurations_supported");
        vec![(
            "KonnektorenCertificate".to_string(),
            serde_json::from_reader(
                File::open("./assets/konnektoren_certificate_config.json").unwrap(),
            )
            .unwrap(),
        )]
        .into_iter()
        .collect()
    }

    fn get_authorization_code(&self) -> Option<AuthorizationCode> {
        let state = Uuid::new_v4().to_string();
        log::debug!("get_authorization_code {}", state);
        Some(AuthorizationCode {
            issuer_state: Some(state),
            authorization_server: None,
        })
    }

    fn get_authorization_response(&self) -> Option<AuthorizationResponse> {
        let code = generate_authorization_code(16);

        Some(AuthorizationResponse { code, state: None })
    }

    fn get_pre_authorized_code(&self) -> Option<PreAuthorizedCode> {
        Some(generate_pre_authorized_code())
    }

    fn get_token_response(&self, token_request: TokenRequest) -> Option<TokenResponse> {
        log::debug!("get_token_response: {:?}", token_request);
        let (is_valid, pre_authorized_code) = match token_request {
            TokenRequest::AuthorizationCode { code, .. } => (
                self.get_certificate_id_by_access_token(&code).is_some(),
                None,
            ),
            TokenRequest::PreAuthorizedCode {
                pre_authorized_code,
                ..
            } => (
                self.get_certificate_id_by_pre_authorized_code(&pre_authorized_code)
                    .is_some(),
                Some(pre_authorized_code),
            ),
        };

        if is_valid {
            let access_token = generate_authorization_code(16); // Generate a new access token
            if let Some(pre_authorized_code) = pre_authorized_code {
                if let Some(certificate_id) =
                    self.get_certificate_id_by_pre_authorized_code(&pre_authorized_code)
                {
                    self.associate_access_token(access_token.clone(), certificate_id);
                }
            }

            Some(TokenResponse {
                access_token,
                token_type: "bearer".to_string(),
                expires_in: Some(86400),
                refresh_token: None,
                scope: None,
                c_nonce: Some(C_NONCE.clone()),
                c_nonce_expires_in: Some(86400),
            })
        } else {
            None
        }
    }

    fn get_credential_response(
        &self,
        access_token: String,
        subject_did: Url,
        issuer_did: Url,
        credential_format: CFC,
        signer: SigningSubject,
    ) -> Option<CredentialResponse> {
        log::debug!("Getting credential response for {}", access_token);

        log::debug!("access_token: {}", access_token);
        log::debug!("credential_format: {:?}", credential_format);
        log::debug!("subject did: {}", subject_did);
        log::debug!("issuer did: {}", issuer_did);

        let issuer_did = get_issuer_did();
        log::debug!("updated issuer did {}", issuer_did);

        let type_ = match serde_json::from_value::<CredentialFormats<WithParameters>>(
            serde_json::to_value(credential_format).unwrap(),
        )
        .unwrap()
        {
            CredentialFormats::JwtVcJson(credential) => {
                credential.parameters.credential_definition.type_
            }
            _ => unreachable!("Credential format not supported"),
        };

        let certificate_id = self.get_certificate_id_by_access_token(&access_token);
        log::debug!("certificate_id: {:?}", certificate_id);

        let certificate = certificate_id
            .clone()
            .and_then(|id| self.get_certificate(&id));

        let credential_json: serde_json::Value = match certificate {
            Some(certificate) => {
                let achievement_credential: AchievementCredential = certificate.into();
                serde_json::to_value(achievement_credential).unwrap()
            }
            None => {
                log::error!(
                    "Certificate not found for certificate id: {:?}",
                    certificate_id
                );
                return None;
            }
        };

        log::debug!("Credential JSON: {:?}", credential_json);

        let mut verifiable_credential: serde_json::Value = credential_json;
        verifiable_credential["issuer"] = json!(issuer_did);
        verifiable_credential["credentialSubject"]["id"] = json!(subject_did);

        log::debug!("Verifiable Credential: {:?}", verifiable_credential);

        Some(CredentialResponse {
            credential: CredentialResponseType::Immediate {
                credential: serde_json::to_value(block_on(async {
                    jwt::encode(
                        signer.clone(),
                        Header::new(Algorithm::EdDSA),
                        VerifiableCredentialJwt::builder()
                            .sub(subject_did.clone())
                            .iss(issuer_did.clone())
                            .iat(0)
                            .exp(9999999999i64)
                            .verifiable_credential(verifiable_credential)
                            .build()
                            .ok(),
                        "did:key",
                    )
                    .await
                    .ok()
                }))
                .unwrap(),
                notification_id: None,
            },
            c_nonce: Some(C_NONCE.clone()),
            c_nonce_expires_in: Some(86400),
        })
    }

    fn get_state(&self) -> Option<String> {
        log::debug!("Getting state in memory storage");
        None
    }

    fn set_state(&mut self, _state: String) {
        log::debug!("Setting state in memory storage: {}", _state)
    }
}

pub fn get_issuer_did() -> String {
    let (priv_key, _) = load_config();

    let issuer_key: PatchedKeyPair = generate::<Ed25519KeyPair>(Some(priv_key.as_bytes()));

    let document: Document = issuer_key.get_did_document(did_key::Config::default());
    document.id
}
