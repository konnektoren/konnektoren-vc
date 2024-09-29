use crate::certificate_data::CertificateData;
use crate::config::load_config;
use crate::storage::MemoryStorage;
use qrcodegen::{QrCode, QrCodeEcc};
use uuid::Uuid;

pub struct CertificateService<'a> {
    issuer_url: String,
    storage: &'a MemoryStorage,
}

impl<'a> CertificateService<'a> {
    pub fn new(storage: &'a MemoryStorage) -> Self {
        let (_, issuer_url) = load_config();
        Self {
            issuer_url,
            storage,
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

        // Store the certificate data
        self.storage
            .store_certificate(offer_id.clone(), certificate_data.clone());

        Ok(format!(
            "openid-credential-offer://?credential_offer_uri={}",
            credential_offer_uri
        ))
    }
}
