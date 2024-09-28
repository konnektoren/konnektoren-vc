use crate::storage::{SharedStorage, Storage};
use chrono::Utc;
use qrcodegen::{QrCode, QrCodeEcc};

pub struct CertificateService {
    storage: SharedStorage,
}

impl CertificateService {
    pub fn new(storage: SharedStorage) -> Self {
        Self { storage }
    }

    pub fn generate_qr_code(&self, certificate_data: &CertificateData) -> Result<String, String> {
        let credential_offer_url = format!(
            "https://example.com/credential_offer?name={}&total_challenges={}&solved_challenges={}",
            certificate_data.profile_name,
            certificate_data.total_challenges,
            certificate_data.solved_challenges
        );

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
