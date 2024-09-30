use crate::certificate_data::CertificateData;
use crate::manager::ManagerType;
use anyhow::Result;
use qrcodegen::{QrCode, QrCodeEcc};
use url::Url;

pub struct CertificateService<'a> {
    manager: &'a ManagerType,
}

impl<'a> CertificateService<'a> {
    pub fn new(manager: &'a ManagerType) -> Self {
        Self { manager }
    }

    pub fn generate_qr_code(&self, certificate_data: &CertificateData) -> Result<String> {
        let offer_url = self.generate_offer_url(certificate_data)?;

        let qr = QrCode::encode_text(&offer_url, QrCodeEcc::Medium)?;

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

    pub fn generate_offer_url(&self, certificate_data: &CertificateData) -> Result<String> {
        let offer_url = self.manager.credential_offer_query(false)?;

        // Parse the URL and extract the credential_offer parameter
        let parsed_url = Url::parse(&offer_url)?;
        let credential_offer = parsed_url
            .query_pairs()
            .find(|(key, _)| key == "credential_offer")
            .map(|(_, value)| value.into_owned())
            .ok_or_else(|| anyhow::anyhow!("No credential offer found in URL"))?;

        // Decode and parse the credential offer JSON
        let decoded_offer = urlencoding::decode(&credential_offer)?;
        let offer_json: serde_json::Value = serde_json::from_str(&decoded_offer)?;

        // Extract the pre-authorized code
        let pre_authorized_code = offer_json["grants"]
            ["urn:ietf:params:oauth:grant-type:pre-authorized_code"]["pre-authorized_code"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No pre-authorized code found"))?;

        // Store the certificate data with the pre-authorized code
        self.manager
            .storage
            .store_certificate(pre_authorized_code.to_string(), certificate_data.clone());

        Ok(offer_url)
    }

    pub fn get_certificate(&self, offer_id: &str) -> Option<CertificateData> {
        self.manager.storage.get_certificate(offer_id)
    }
}
