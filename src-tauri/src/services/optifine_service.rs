use crate::app_error;
use crate::error::AppError;
use crate::utils::paths::get_cache_dir;
use chrono::{DateTime, Duration, Utc};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const OPTIFINE_DOWNLOADS_URL: &str = "https://optifine.net/downloads";
const CACHE_TTL_HOURS: i64 = 1;

/// Represents a single OptiFine version available for download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptifineVersion {
    /// Minecraft version (e.g., "1.20.1")
    pub mc_version: String,
    /// Full filename (e.g., "OptiFine_1.20.1_HD_U_I6.jar")
    pub filename: String,
    /// Direct download URL (mirror link)
    pub download_url: String,
    /// Whether this is a preview/pre-release version
    pub is_preview: bool,
    /// Compatible Forge version if listed (e.g., "Forge 47.2.18")
    pub forge_version: Option<String>,
}

/// Cache structure for OptiFine versions
#[derive(Debug, Serialize, Deserialize)]
struct OptifineCache {
    versions: Vec<OptifineVersion>,
    fetched_at: DateTime<Utc>,
}

/// Get the cache file path for OptiFine versions
fn get_optifine_cache_path() -> PathBuf {
    get_cache_dir().join("optifine_versions.json")
}

/// Load cached OptiFine versions if valid (not expired)
fn load_cache() -> Option<Vec<OptifineVersion>> {
    let cache_path = get_optifine_cache_path();

    if !cache_path.exists() {
        return None;
    }

    let content = fs::read_to_string(&cache_path).ok()?;
    let cache: OptifineCache = serde_json::from_str(&content).ok()?;

    // Check if cache is still valid (within TTL)
    let now = Utc::now();
    let cache_age = now.signed_duration_since(cache.fetched_at);

    if cache_age < Duration::hours(CACHE_TTL_HOURS) {
        Some(cache.versions)
    } else {
        None
    }
}

/// Save OptiFine versions to cache
fn save_cache(versions: &[OptifineVersion]) -> Result<(), AppError> {
    let cache_path = get_optifine_cache_path();

    // Ensure cache directory exists
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let cache = OptifineCache {
        versions: versions.to_vec(),
        fetched_at: Utc::now(),
    };

    let content = serde_json::to_string_pretty(&cache)?;
    fs::write(&cache_path, content)?;

    Ok(())
}

/// Parse the OptiFine downloads page HTML to extract versions
fn parse_optifine_html(html: &str) -> Vec<OptifineVersion> {
    let mut versions = Vec::new();

    // Regex to extract the jar filename from mirror URLs
    // Matches: http://optifine.net/adloadx?f=OptiFine_1.20.1_HD_U_I6.jar or preview_OptiFine_...
    let mirror_url_re =
        Regex::new(r#"http://optifine\.net/adloadx\?f=((?:preview_)?OptiFine_([^_]+)_[^"]+\.jar)"#)
            .unwrap();

    // Find all mirror URLs and extract version info from the filename
    for cap in mirror_url_re.captures_iter(html) {
        let full_filename = cap[1].to_string();
        let mc_version = cap[2].to_string();
        let download_url = format!("http://optifine.net/adloadx?f={}", &full_filename);

        // Check if this is a preview version
        let is_preview = full_filename.starts_with("preview_") || full_filename.contains("_pre");

        // Create a cleaner display filename (remove preview_ prefix for display)
        let filename = if let Some(stripped) = full_filename.strip_prefix("preview_") {
            stripped.to_string()
        } else {
            full_filename.clone()
        };

        // Avoid duplicates (mirror URLs appear twice - once in ad link, once in mirror link)
        if !versions
            .iter()
            .any(|v: &OptifineVersion| v.download_url == download_url)
        {
            versions.push(OptifineVersion {
                mc_version,
                filename,
                download_url,
                is_preview,
                forge_version: None, // We don't parse forge version for simplicity
            });
        }
    }

    versions
}

/// Fetch OptiFine versions from the website or cache
pub async fn fetch_optifine_versions(
    client: &Client,
    force_refresh: bool,
) -> Result<Vec<OptifineVersion>, AppError> {
    // Try to use cached versions unless force refresh
    if !force_refresh {
        if let Some(cached) = load_cache() {
            return Ok(cached);
        }
    }

    // Fetch from OptiFine website
    let response = client
        .get(OPTIFINE_DOWNLOADS_URL)
        .header("User-Agent", "ETLauncher/1.0")
        .send()
        .await
        .map_err(|e| {
            AppError::ApiError(format!("Could not reach OptiFine download servers: {}", e))
        })?;

    if !response.status().is_success() {
        return Err(AppError::ApiError(
            "Could not reach OptiFine download servers. Please try again later.".to_string(),
        ));
    }

    let html = response
        .text()
        .await
        .map_err(|e| AppError::ApiError(format!("Failed to read OptiFine page: {}", e)))?;

    // Filter out preview versions - they're unstable and often incompatible with Forge
    let versions: Vec<OptifineVersion> = parse_optifine_html(&html)
        .into_iter()
        .filter(|v| !v.is_preview)
        .collect();

    if versions.is_empty() {
        return Err(AppError::ApiError(
            "No stable OptiFine versions available. Only preview versions exist, which are not supported due to compatibility issues.".to_string()
        ));
    }

    // Cache the results
    if let Err(e) = save_cache(&versions) {
        app_error!("Warning: Failed to cache OptiFine versions: {}", e);
    }

    Ok(versions)
}

/// Get the best OptiFine version for a specific Minecraft version
/// Only returns stable releases (preview versions are filtered out)
pub async fn get_optifine_for_mc_version(
    client: &Client,
    mc_version: &str,
) -> Result<Option<OptifineVersion>, AppError> {
    let versions = fetch_optifine_versions(client, false).await?;

    // Find the first stable version for this MC version
    // (preview versions are already filtered out at fetch time)
    Ok(versions.into_iter().find(|v| v.mc_version == mc_version))
}

/// Check if OptiFine is available for a specific Minecraft version
pub async fn check_optifine_available(client: &Client, mc_version: &str) -> Result<bool, AppError> {
    let version = get_optifine_for_mc_version(client, mc_version).await?;
    Ok(version.is_some())
}

/// Download OptiFine to the specified mods directory
///
/// OptiFine uses a two-step download process:
/// 1. Fetch the adloadx page (e.g., http://optifine.net/adloadx?f=OptiFine_1.20.1_HD_U_I6.jar)
/// 2. Parse out the downloadx URL with token from the page
/// 3. Download the actual JAR from the downloadx URL
pub async fn download_optifine(
    client: &Client,
    mc_version: &str,
    mods_dir: &PathBuf,
) -> Result<String, AppError> {
    let version = get_optifine_for_mc_version(client, mc_version)
        .await?
        .ok_or_else(|| {
            AppError::ContentNotFound(format!(
                "No OptiFine available for Minecraft {}",
                mc_version
            ))
        })?;

    // Ensure mods directory exists
    fs::create_dir_all(mods_dir)?;

    // Step 1: Fetch the adloadx page to get the download token
    let adloadx_response = client
        .get(&version.download_url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .send()
        .await
        .map_err(|e| {
            AppError::DownloadError(format!("Failed to access OptiFine download page: {}", e))
        })?;

    if !adloadx_response.status().is_success() {
        return Err(AppError::DownloadError(
            "Failed to access OptiFine download page. Please try again.".to_string(),
        ));
    }

    let adloadx_html = adloadx_response
        .text()
        .await
        .map_err(|e| AppError::DownloadError(format!("Failed to read OptiFine page: {}", e)))?;

    // Step 2: Extract the downloadx URL with token
    // Pattern: href='downloadx?f=OptiFine_1.20.1_HD_U_I6.jar&x=TOKEN'
    let downloadx_re = Regex::new(r#"href='(downloadx\?f=[^']+)'"#).unwrap();
    let downloadx_path = downloadx_re
        .captures(&adloadx_html)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str())
        .ok_or_else(|| {
            AppError::DownloadError(
                "Could not find OptiFine download link. The website structure may have changed."
                    .to_string(),
            )
        })?;

    // Build the full downloadx URL
    let downloadx_url = format!("https://optifine.net/{}", downloadx_path);

    // Step 3: Download the actual JAR file
    let response = client
        .get(&downloadx_url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .header("Referer", &version.download_url)
        .send()
        .await
        .map_err(|e| AppError::DownloadError(format!("Failed to download OptiFine: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::DownloadError(format!(
            "Failed to download OptiFine (HTTP {}). Please try again.",
            response.status()
        )));
    }

    // Verify we got a JAR file (should be application/java-archive or application/octet-stream)
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if content_type.contains("text/html") {
        return Err(AppError::DownloadError(
            "OptiFine download returned HTML instead of JAR. The download may have been blocked."
                .to_string(),
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| AppError::DownloadError(format!("Failed to read OptiFine download: {}", e)))?;

    // Basic sanity check - JAR files start with PK (ZIP magic number)
    if bytes.len() < 4 || &bytes[0..2] != b"PK" {
        return Err(AppError::DownloadError(
            "Downloaded file is not a valid JAR. The download may have failed.".to_string(),
        ));
    }

    // Use the filename from the parsed version (already includes .jar extension)
    let filename = version.filename.clone();

    let file_path = mods_dir.join(&filename);

    fs::write(&file_path, bytes)?;

    Ok(filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_optifine_html_basic() {
        let html = r#"
            <h2>Minecraft 1.20.1</h2>
            <table class='downloadTable mainTable'>
                <tr class='downloadLine downloadLineMain'>
                    <td class='colFile'>OptiFine HD U I6</td>
                    <td class='colDownload'><a href="http://adfoc.us/...">Download</a></td>
                    <td class='colMirror'><a href="http://optifine.net/adloadx?f=OptiFine_1.20.1_HD_U_I6.jar">(Mirror)</a></td>
                    <td class='colChangelog'><a href='...'>Changelog</a></td>
                    <td class='colForge'>Forge 47.2.18</td>
                </tr>
            </table>
        "#;

        let versions = parse_optifine_html(html);
        assert!(!versions.is_empty());
    }
}
