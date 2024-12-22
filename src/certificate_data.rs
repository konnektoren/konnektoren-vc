use chrono::Utc;
use types_ob_v3::prelude::{
    Achievement, AchievementBuilder, AchievementCredential, AchievementCredentialBuilder,
    AchievementCredentialType, AchievementSubject, AchievementSubjectBuilder, Criteria,
    CriteriaBuilder, ImageBuilder, Profile, ProfileBuilder, ResultBuilder,
};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CertificateData {
    pub game_path_name: String,
    pub total_challenges: usize,
    pub solved_challenges: usize,
    pub performance_percentage: u8,
    pub profile_name: String,
    pub date: chrono::DateTime<Utc>,
}

impl From<CertificateData> for AchievementCredential {
    fn from(cert_data: CertificateData) -> Self {
        let criteria: Criteria = CriteriaBuilder::default()
            .narrative(format!(
                "Completed {} out of {} challenges with a performance of {}%",
                cert_data.solved_challenges,
                cert_data.total_challenges,
                cert_data.performance_percentage
            ))
            .try_into()
            .unwrap();

        let achievement: Achievement = AchievementBuilder::default()
            .id("https://example.com/achievements/certificate".to_string())
            .type_("Achievement")
            .criteria(criteria)
            .name(cert_data.game_path_name.clone())
            .description(format!(
                "This certificate recognizes the achievement of {} in completing the course.",
                cert_data.profile_name
            ))
            .image(
                ImageBuilder::default()
                    .id(konnektoren_image())
                    .type_("Image")
                    .caption("Konnektoren Logo".to_string()),
            )
            .credits_available(cert_data.total_challenges as f64)
            .try_into()
            .unwrap();

        let achievement_subject: AchievementSubject = AchievementSubjectBuilder::default()
            .id("did:example:ebfeb1f712ebc6f1c276e12ec21".to_string())
            .type_("AchievementSubject")
            .achievement(achievement)
            .image(
                ImageBuilder::default()
                    .id(konnektoren_image())
                    .type_("Image")
                    .caption("Konnektoren Logo".to_string()),
            )
            .credits_earned(cert_data.solved_challenges as f64)
            .result(vec![ResultBuilder::default()
                .achieved_level(cert_data.game_path_name.clone())
                .type_("Result")
                .value(Some(format!("{}%", cert_data.performance_percentage)))])
            .try_into()
            .unwrap();

        // Building issuer profile
        let issuer: Profile = ProfileBuilder::default()
            .id("vc.konnektoren.help")
            .type_("Profile")
            .name("Konnektoren".to_string())
            .image(
                ImageBuilder::default()
                    .id(konnektoren_image())
                    .type_("Image")
                    .caption("Konnektoren Logo".to_string()),
            )
            .try_into()
            .unwrap();

        // Building the achievement credential
        AchievementCredentialBuilder::default()
            .context(vec![
                "https://www.w3.org/2018/credentials/v1",
                "https://purl.imsglobal.org/spec/ob/v3p0/context-3.0.2.json",
            ])
            .credential_subject(&achievement_subject)
            .id("http://example.com/credentials/3527".to_string())
            .name(cert_data.game_path_name.clone())
            .type_(AchievementCredentialType::from(vec![
                "VerifiableCredential",
                "OpenBadgeCredential",
            ]))
            .issuance_date(cert_data.date.to_rfc3339())
            .issuer(issuer)
            .awarded_date(cert_data.date.to_rfc3339())
            .description(format!(
                "This certificate recognizes the achievement of {} in completing the {} course.",
                cert_data.profile_name, cert_data.game_path_name
            ))
            .image(
                ImageBuilder::default()
                    .id(konnektoren_image())
                    .type_("Image")
                    .caption("Konnektoren Logo".to_string()),
            )
            .try_into()
            .unwrap()
    }
}

fn konnektoren_image() -> String {
    let data = include_bytes!("../assets/favicon.png");
    let encoded = base64::encode(data);
    format!("data:image/png;base64,{}", encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDateTime, TimeZone, Utc};
    use std::fs::File;
    use std::io::BufReader;
    use types_ob_v3::prelude::AchievementCredential;

    #[test]
    fn test_conversion() {
        let naive_date = NaiveDateTime::parse_from_str("2024-09-30 08:00:00", "%Y-%m-%d %H:%M:%S")
            .expect("Failed to parse date");
        let utc_date = Utc.from_utc_datetime(&naive_date);

        let cert_data = CertificateData {
            game_path_name: "Introduction to Rust".to_string(),
            total_challenges: 10,
            solved_challenges: 8,
            performance_percentage: 80,
            profile_name: "Alice Rustacean".to_string(),
            date: utc_date, // Use Utc DateTime here
        };

        // Convert CertificateData into AchievementCredential
        let achievement_credential: AchievementCredential = cert_data.into();

        // Write achievement_credential to file
        let file =
            File::create("assets/konnektoren_certificate.json").expect("Failed to create file");
        serde_json::to_writer_pretty(file, &achievement_credential)
            .expect("Couldn't write to file");

        // Load JSON from file to test the conversion
        let file = File::open("assets/konnektoren_certificate.json")
            .expect("Failed to open file for reading");
        let reader = BufReader::new(file);

        let json_value_from_file: serde_json::Value =
            serde_json::from_reader(reader).expect("Couldn't read from file");

        // Ensure the generated credential matches the expected JSON structure
        assert_eq!(
            serde_json::to_value(&achievement_credential).unwrap(),
            json_value_from_file
        );
    }
}
