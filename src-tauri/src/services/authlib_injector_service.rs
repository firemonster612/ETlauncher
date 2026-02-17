use crate::app_error;
use crate::error::AppError;
use crate::utils::paths::get_app_data_dir;
use serde::Deserialize;
use std::path::PathBuf;

/// Get the directory for authlib-injector storage
fn get_authlib_injector_dir() -> PathBuf {
    get_app_data_dir().join("authlib-injector")
}

/// Get the path to the version file that tracks the current version
fn get_version_file_path() -> PathBuf {
    get_authlib_injector_dir().join("version.txt")
}

/// API response from authlib-injector latest artifact endpoint
#[derive(Deserialize)]
struct LatestArtifact {
    #[allow(dead_code)]
    build_number: u32,
    version: String,
    download_url: String,
    #[allow(dead_code)]
    checksums: ArtifactChecksums,
}

#[derive(Deserialize)]
struct ArtifactChecksums {
    sha256: String,
}

/// Get the path to the cached authlib-injector jar, downloading if needed
pub async fn get_authlib_injector_path(client: &reqwest::Client) -> Result<PathBuf, AppError> {
    let dir = get_authlib_injector_dir();
    std::fs::create_dir_all(&dir)?;

    // Check if we have a cached version
    let version_file = get_version_file_path();
    let cached_version = if version_file.exists() {
        std::fs::read_to_string(&version_file).ok()
    } else {
        None
    };

    // Check if the jar exists for the cached version
    if let Some(ref version) = cached_version {
        let jar_path = dir.join(format!("authlib-injector-{}.jar", version));
        if jar_path.exists() {
            app_error!("[authlib-injector] Using cached version {}", version);
            return Ok(jar_path);
        }
    }

    // Need to download - fetch latest version info
    app_error!("[authlib-injector] Checking for latest version...");
    let latest = fetch_latest_artifact(client).await?;
    let jar_path = dir.join(format!("authlib-injector-{}.jar", latest.version));

    // Check if we already have this version
    if jar_path.exists() {
        app_error!("[authlib-injector] Already have version {}", latest.version);
        // Update version file in case it was missing
        let _ = std::fs::write(&version_file, &latest.version);
        return Ok(jar_path);
    }

    // Download the jar
    app_error!(
        "[authlib-injector] Downloading version {} from {}",
        latest.version,
        latest.download_url
    );

    let response = client.get(&latest.download_url).send().await?;

    if !response.status().is_success() {
        return Err(AppError::DownloadError(format!(
            "Failed to download authlib-injector: HTTP {}",
            response.status()
        )));
    }

    let bytes = response.bytes().await?;

    // Verify SHA-256 checksum
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = format!("{:x}", hasher.finalize());

    if hash != latest.checksums.sha256 {
        return Err(AppError::HashMismatch(format!(
            "authlib-injector checksum mismatch: expected {}, got {}",
            latest.checksums.sha256, hash
        )));
    }

    // Write the jar
    std::fs::write(&jar_path, &bytes)?;

    // Update version file
    std::fs::write(&version_file, &latest.version)?;

    // Clean up old versions
    cleanup_old_versions(&dir, &latest.version);

    app_error!(
        "[authlib-injector] Downloaded and cached version {}",
        latest.version
    );

    Ok(jar_path)
}

/// Fetch the latest artifact info from the authlib-injector API
async fn fetch_latest_artifact(client: &reqwest::Client) -> Result<LatestArtifact, AppError> {
    let response = client
        .get("https://authlib-injector.yushi.moe/artifact/latest.json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(AppError::ApiError(format!(
            "Failed to fetch authlib-injector info: HTTP {}",
            response.status()
        )));
    }

    let artifact: LatestArtifact = response.json().await.map_err(|e| {
        AppError::ApiError(format!("Failed to parse authlib-injector response: {}", e))
    })?;

    Ok(artifact)
}

/// Remove old versions of authlib-injector, keeping only the current one
fn cleanup_old_versions(dir: &PathBuf, current_version: &str) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("authlib-injector-")
                    && name.ends_with(".jar")
                    && !name.contains(current_version)
                {
                    app_error!("[authlib-injector] Removing old version: {}", name);
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }
}

/// Check if authlib-injector is available (already cached)
pub fn is_cached() -> bool {
    let version_file = get_version_file_path();
    if let Ok(version) = std::fs::read_to_string(&version_file) {
        let jar_path =
            get_authlib_injector_dir().join(format!("authlib-injector-{}.jar", version.trim()));
        jar_path.exists()
    } else {
        false
    }
}
