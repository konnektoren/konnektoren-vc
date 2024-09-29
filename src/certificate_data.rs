use chrono::Utc;

#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema, Clone, Debug)]
pub struct CertificateData {
    pub game_path_name: String,
    pub total_challenges: usize,
    pub solved_challenges: usize,
    pub performance_percentage: u8,
    pub profile_name: String,
    #[schema(value_type = String, format = DateTime)]
    pub date: chrono::DateTime<Utc>,
}
