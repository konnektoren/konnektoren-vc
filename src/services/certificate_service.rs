use crate::config::load_config;
use crate::storage::SharedStorage;
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use qrcodegen::{QrCode, QrCodeEcc};
use serde_json::json;
use uuid::Uuid;

pub struct CertificateService {
    _storage: SharedStorage,
    issuer_url: String,
}

impl CertificateService {
    pub fn new(_storage: SharedStorage) -> Self {
        let (_, issuer_url) = load_config();
        Self {
            _storage,
            issuer_url,
        }
    }

    pub fn generate_qr_code(&self, certificate_data: &CertificateData) -> Result<String, String> {
        let credential_offer_url = self.generate_offer_url(certificate_data)?;

        let qr = QrCode::encode_text(&credential_offer_url, QrCodeEcc::Medium)
            .map_err(|e| e.to_string())?;

        Ok(self.qr_to_string(&qr))
    }

    fn qr_to_string(&self, qr: &QrCode) -> String {
        let size = qr.size();
        let mut result = String::new();
        for y in 0..size {
            for x in 0..size {
                result.push(if qr.get_module(x, y) { '█' } else { ' ' });
            }
            result.push('\n');
        }
        result
    }

    pub fn get_issuer_url(&self) -> &str {
        &self.issuer_url
    }

    pub fn generate_offer_url(&self, certificate_data: &CertificateData) -> Result<String, String> {
        let offer_id = Uuid::new_v4().to_string();
        let credential_offer_uri = format!("{}/api/v1/offers/{}", self.issuer_url, offer_id);

        self.store_credential_offer(&offer_id, certificate_data)?;

        Ok(format!(
            "openid-credential-offer://?credential_offer_uri={}",
            credential_offer_uri
        ))
    }

    pub fn store_credential_offer(
        &self,
        offer_id: &str,
        certificate_data: &CertificateData,
    ) -> Result<(), String> {
        let offer_data = serde_json::to_string(certificate_data).map_err(|e| e.to_string())?;
        self._storage
            .lock()
            .unwrap()
            .store(offer_id.to_string(), offer_data)
    }

    pub fn retrieve_credential_offer(
        &self,
        offer_id: &str,
    ) -> Result<Option<CertificateData>, String> {
        match self._storage.lock().unwrap().retrieve(offer_id) {
            Some(offer_data) => {
                let certificate_data: CertificateData = serde_json::from_str(&offer_data)
                    .map_err(|e| format!("Failed to parse stored offer data: {}", e))?;
                Ok(Some(certificate_data))
            }
            None => Ok(None),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CertificateData {
    pub game_path_name: String,
    pub total_challenges: usize,
    pub solved_challenges: usize,
    pub performance_percentage: u8,
    pub profile_name: String,
    #[schema(value_type = String, format = DateTime)]
    pub date: chrono::DateTime<Utc>,
}
