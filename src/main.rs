use dotenv::dotenv;
use konnektoren_vc::prelude::*;

#[tokio::main]
async fn main() {
    dotenv().ok();
    init_telemetry()
        .await
        .expect("Failed to initialize telemetry.");

    start_server().await.expect("Failed to start server.");
}
