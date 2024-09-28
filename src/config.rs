use dotenv::dotenv;
use std::env;

pub fn load_config() -> (String, String) {
    dotenv().ok();

    let private_key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set in the .env file");
    let issuer_url = env::var("ISSUER_URL").expect("ISSUER_URL must be set in the .env file");

    (private_key, issuer_url)
}
