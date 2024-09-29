use dotenv::dotenv;
use std::env;

pub fn load_config() -> (String, String) {
    dotenv().ok();

    let private_key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set in the .env file");
    let issuer_url = env::var("ISSUER_URL").expect("ISSUER_URL must be set in the .env file");

    (private_key, issuer_url)
}

#[derive(Debug)]
pub struct Config {
    pub private_key: String,
    pub issuer_url: Option<String>,
}

impl Config {
    pub fn new(private_key: String, issuer_url: String) -> Self {
        Self {
            private_key,
            issuer_url: Some(issuer_url),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let (private_key, issuer_url) = load_config();
        Self::new(private_key, issuer_url)
    }
}
