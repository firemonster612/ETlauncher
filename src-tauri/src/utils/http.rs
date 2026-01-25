use reqwest::Client;
use std::time::Duration;

/// Create a configured HTTP client with optimized connection pooling
///
/// This client is designed to be created once and reused throughout the application.
/// It includes:
/// - Connection pooling for HTTP keep-alive benefits
/// - TCP keepalive to prevent idle connection resets
/// - TCP nodelay for reduced latency
pub fn create_client() -> reqwest::Result<Client> {
    Client::builder()
        .user_agent("ETLauncher/1.0")
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        // Connection pool settings for concurrent downloads
        .pool_max_idle_per_host(20)
        .pool_idle_timeout(Duration::from_secs(90))
        // TCP optimizations
        .tcp_keepalive(Duration::from_secs(30))
        .tcp_nodelay(true)
        .build()
}

/// Create a client with a custom timeout for large downloads
///
/// Uses the same optimized settings as `create_client()` but with a longer timeout
/// suitable for downloading large files like modpacks.
pub fn create_download_client() -> reqwest::Result<Client> {
    Client::builder()
        .user_agent("ETLauncher/1.0")
        .timeout(Duration::from_secs(300)) // 5 minutes for large files
        .connect_timeout(Duration::from_secs(10))
        // Connection pool settings for concurrent downloads
        .pool_max_idle_per_host(20)
        .pool_idle_timeout(Duration::from_secs(90))
        // TCP optimizations
        .tcp_keepalive(Duration::from_secs(30))
        .tcp_nodelay(true)
        .build()
}
