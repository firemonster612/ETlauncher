use reqwest::Client;
use std::time::Duration;

/// Create a configured HTTP client
pub fn create_client() -> reqwest::Result<Client> {
    Client::builder()
        .user_agent("ETLauncher/1.0")
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()
}

/// Create a client with a custom timeout for large downloads
pub fn create_download_client() -> reqwest::Result<Client> {
    Client::builder()
        .user_agent("ETLauncher/1.0")
        .timeout(Duration::from_secs(300)) // 5 minutes for large files
        .connect_timeout(Duration::from_secs(10))
        .build()
}
