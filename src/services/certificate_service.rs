use crate::config::load_config;
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use qrcodegen::{QrCode, QrCodeEcc};
use serde_json::json;
use uuid::Uuid;

pub struct CertificateService {
    issuer_url: String,
}

impl CertificateService {
    pub fn new() -> Self {
        let (_, issuer_url) = load_config();
        Self { issuer_url }
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

        Ok(format!(
            "openid-credential-offer://?credential_offer_uri={}",
            credential_offer_uri
        ))
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
