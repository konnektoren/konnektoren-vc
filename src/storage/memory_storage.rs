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
    pub static ref CODE: String = generate_authorization_code(16);
    pub static ref PRE_AUTHORIZED_CODE: PreAuthorizedCode = PreAuthorizedCode {
        pre_authorized_code: generate_authorization_code(16),
        ..Default::default()
    };
    pub static ref ACCESS_TOKEN: String = "czZCaGRSa3F0MzpnWDFmQmF0M2JW".to_string();
    pub static ref C_NONCE: String = "tZignsnFbp".to_string();
}

use crate::certificate_data::CertificateData;
use std::sync::{Arc, Mutex};
use types_ob_v3::prelude::AchievementCredential;
use uuid::Uuid;

#[derive(Clone)]
pub struct MemoryStorage {
    certificates: Arc<Mutex<HashMap<String, CertificateData>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            certificates: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn store_certificate(&self, offer_id: String, certificate_data: CertificateData) {
        let mut certificates = self.certificates.lock().unwrap();
        certificates.insert(offer_id, certificate_data);
    }

    pub fn get_certificate(&self, offer_id: &str) -> Option<CertificateData> {
        let certificates = self.certificates.lock().unwrap();
        certificates
            .get(offer_id)
            .cloned()
            .or_else(|| certificates.values().last().cloned())
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
        log::debug!("get_authorization_code");
        let state = Uuid::new_v4().to_string();

        Some(AuthorizationCode {
            issuer_state: Some(state),
            authorization_server: None,
        })
    }

    fn get_authorization_response(&self) -> Option<AuthorizationResponse> {
        Some(AuthorizationResponse {
            code: CODE.clone(),
            state: None,
        })
    }

    fn get_pre_authorized_code(&self) -> Option<PreAuthorizedCode> {
        log::debug!("get_pre_authorized_code");
        Some(PRE_AUTHORIZED_CODE.clone())
    }

    fn get_token_response(&self, token_request: TokenRequest) -> Option<TokenResponse> {
        log::debug!("get_token_response: {:?}", token_request);
        match token_request {
            TokenRequest::AuthorizationCode { code, .. } => code == CODE.clone(),
            TokenRequest::PreAuthorizedCode {
                pre_authorized_code,
                ..
            } => pre_authorized_code == PRE_AUTHORIZED_CODE.pre_authorized_code,
        }
        .then_some(TokenResponse {
            // TODO: dynamically create this.
            access_token: ACCESS_TOKEN.clone(),
            token_type: "bearer".to_string(),
            expires_in: Some(86400),
            refresh_token: None,
            scope: None,
            c_nonce: Some(C_NONCE.clone()),
            c_nonce_expires_in: Some(86400),
        })
    }

    fn get_credential_response(
        &self,
        access_token: String,
        subject_did: Url,
        issuer_did: Url,
        credential_format: CFC,
        signer: SigningSubject,
    ) -> Option<CredentialResponse> {
        log::debug!(
            "Getting credential response in memory storage {}",
            access_token
        );

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

        let certificate = self.get_certificate(&access_token);

        let credential_json: serde_json::Value = match &type_[..] {
            [_, b] if b == "KonnektorenCertificate" => match certificate {
                Some(certificate) => {
                    let achievement_credential: AchievementCredential = certificate.into();
                    serde_json::to_value(achievement_credential).unwrap()
                }
                None => serde_json::from_reader(
                    File::open("./assets/konnektoren_certificate.json").unwrap(),
                )
                .unwrap(),
            },
            _ => unreachable!(),
        };

        log::debug!("Credential JSON: {:?}", credential_json);

        let mut verifiable_credential: serde_json::Value = credential_json;
        verifiable_credential["issuer"] = json!(issuer_did);
        verifiable_credential["credentialSubject"]["id"] = json!(subject_did);

        log::debug!("Verifiable Credential: {:?}", verifiable_credential);

        (access_token == ACCESS_TOKEN.clone()).then_some(CredentialResponse {
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
