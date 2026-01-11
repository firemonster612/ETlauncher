use crate::error::AppError;
use crate::models::instance::LoaderType;
use crate::models::loader::{
    FabricInstallerMeta, FabricLoaderForVersion, ForgePromotions, LiteLoaderVersionsResponse,
    LoaderVersion, NeoForgeMavenResponse,
};
use crate::services::java_service;
use serde::Deserialize;
use std::path::Path;
use tokio::process::Command;

/// Fabric meta API base URL
const FABRIC_META_URL: &str = "https://meta.fabricmc.net/v2";

/// Quilt meta API base URL
const QUILT_META_URL: &str = "https://meta.quiltmc.org/v3";

/// Forge Maven base URL
const FORGE_MAVEN_URL: &str = "https://maven.minecraftforge.net/net/minecraftforge/forge";

/// Forge promotions API URL
const FORGE_PROMOTIONS_URL: &str =
    "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";

/// NeoForge Maven base URL
const NEOFORGE_MAVEN_URL: &str = "https://maven.neoforged.net/net/neoforged/neoforge";

/// NeoForge Maven API URL
const NEOFORGE_API_URL: &str =
    "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";

/// LiteLoader versions API URL
const LITELOADER_VERSIONS_URL: &str = "http://dl.liteloader.com/versions/versions.json";

/// Fabric Maven base URL
const FABRIC_MAVEN_URL: &str = "https://maven.fabricmc.net/net/fabricmc";

/// Quilt Maven base URL
const QUILT_MAVEN_URL: &str = "https://maven.quiltmc.org/repository/release";
const QUILT_INSTALLER_VERSION: &str = "0.12.1";
const QUILT_INSTALLER_COORD: &str = "org/quiltmc/quilt-installer";

/// Fetch available Fabric loader versions for a specific Minecraft version
pub async fn get_fabric_versions(mc_version: &str) -> Result<Vec<LoaderVersion>, AppError> {
    let url = format!("{}/versions/loader/{}", FABRIC_META_URL, mc_version);

    let response = reqwest::get(&url).await.map_err(AppError::HttpError)?;

    response.error_for_status_ref()?;

    let meta_versions: Vec<FabricLoaderForVersion> =
        response.json().await.map_err(AppError::HttpError)?;

    let versions: Vec<LoaderVersion> = meta_versions
        .into_iter()
        .map(|v| LoaderVersion {
            version: v.loader.version.clone(),
            maven: v.loader.maven,
            stable: v.loader.stable.unwrap_or_else(|| {
                !v.loader.version.contains("beta") && !v.loader.version.contains("alpha")
            }),
            build: v.loader.build,
            separator: v.loader.separator,
        })
        .collect();

    Ok(versions)
}

/// Fetch available Quilt loader versions for a specific Minecraft version
pub async fn get_quilt_versions(mc_version: &str) -> Result<Vec<LoaderVersion>, AppError> {
    let url = format!("{}/versions/loader/{}", QUILT_META_URL, mc_version);

    let response = reqwest::get(&url).await.map_err(AppError::HttpError)?;

    response.error_for_status_ref()?;

    let meta_versions: Vec<FabricLoaderForVersion> =
        response.json().await.map_err(AppError::HttpError)?;

    let versions: Vec<LoaderVersion> = meta_versions
        .into_iter()
        .map(|v| LoaderVersion {
            version: v.loader.version.clone(),
            maven: v.loader.maven,
            stable: v.loader.stable.unwrap_or_else(|| {
                !v.loader.version.contains("beta") && !v.loader.version.contains("alpha")
            }),
            build: v.loader.build,
            separator: v.loader.separator,
        })
        .collect();

    Ok(versions)
}

/// Check if a Minecraft version has Forge installer files available on Maven
/// Versions before 1.5.2 don't have installer jars on the Forge Maven
fn has_forge_installer_support(mc_version: &str) -> bool {
    let parts: Vec<&str> = mc_version.split('.').collect();
    if parts.len() < 2 {
        return false;
    }

    let major: u32 = parts[0].parse().unwrap_or(0);
    let minor: u32 = parts[1].parse().unwrap_or(0);
    let patch: u32 = parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0);

    // Only MC 1.5.2+ has Forge installers on Maven
    if major != 1 {
        return major > 1;
    }
    if minor > 5 {
        return true;
    }
    if minor == 5 {
        return patch >= 2;
    }
    false
}

/// Fetch available Forge versions for a specific Minecraft version
pub async fn get_forge_versions(mc_version: &str) -> Result<Vec<LoaderVersion>, AppError> {
    // Forge installers before MC 1.5.2 are not available on Maven
    if !has_forge_installer_support(mc_version) {
        return Ok(Vec::new());
    }

    // Fetch promotions to identify recommended/latest versions
    let promos_response = reqwest::get(FORGE_PROMOTIONS_URL)
        .await
        .map_err(AppError::HttpError)?;

    promos_response.error_for_status_ref()?;

    let promotions: ForgePromotions = promos_response.json().await.map_err(AppError::HttpError)?;

    // Collect versions for this MC version from promos
    let mut versions: Vec<LoaderVersion> = Vec::new();
    let mut seen_versions = std::collections::HashSet::new();

    // Check for recommended version (stable)
    let recommended_key = format!("{}-recommended", mc_version);
    if let Some(forge_ver) = promotions.promos.get(&recommended_key) {
        if seen_versions.insert(forge_ver.clone()) {
            versions.push(LoaderVersion {
                version: forge_ver.clone(),
                maven: format!("net.minecraftforge:forge:{}-{}", mc_version, forge_ver),
                stable: true,
                build: 0,
                separator: "-".to_string(),
            });
        }
    }

    // Check for latest version
    let latest_key = format!("{}-latest", mc_version);
    if let Some(forge_ver) = promotions.promos.get(&latest_key) {
        if seen_versions.insert(forge_ver.clone()) {
            versions.push(LoaderVersion {
                version: forge_ver.clone(),
                maven: format!("net.minecraftforge:forge:{}-{}", mc_version, forge_ver),
                stable: false,
                build: 0,
                separator: "-".to_string(),
            });
        }
    }

    // Also fetch from Maven metadata for more versions
    let maven_metadata_url = format!("{}/maven-metadata.xml", FORGE_MAVEN_URL);
    if let Ok(response) = reqwest::get(&maven_metadata_url).await {
        if let Ok(text) = response.text().await {
            // Parse XML to extract versions for this MC version
            // Format is: <version>mcversion-forgeversion</version>
            let prefix = format!("{}-", mc_version);
            for line in text.lines() {
                if let Some(start) = line.find("<version>") {
                    if let Some(end) = line.find("</version>") {
                        let version_str = &line[start + 9..end];
                        if version_str.starts_with(&prefix) {
                            let forge_ver =
                                version_str.strip_prefix(&prefix).unwrap_or(version_str);
                            if seen_versions.insert(forge_ver.to_string()) {
                                let is_stable = promotions
                                    .promos
                                    .get(&recommended_key)
                                    .map(|v| v == forge_ver)
                                    .unwrap_or(false);
                                versions.push(LoaderVersion {
                                    version: forge_ver.to_string(),
                                    maven: format!("net.minecraftforge:forge:{}", version_str),
                                    stable: is_stable,
                                    build: 0,
                                    separator: "-".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort versions (newer first) - simple string sort works for semver-like versions
    versions.sort_by(|a, b| b.version.cmp(&a.version));

    Ok(versions)
}

/// Fetch available NeoForge versions for a specific Minecraft version
pub async fn get_neoforge_versions(mc_version: &str) -> Result<Vec<LoaderVersion>, AppError> {
    let response = reqwest::get(NEOFORGE_API_URL)
        .await
        .map_err(AppError::HttpError)?;

    response.error_for_status_ref()?;

    let maven_response: NeoForgeMavenResponse =
        response.json().await.map_err(AppError::HttpError)?;

    // NeoForge versions are formatted as: mcversion.forgeversion (e.g., "21.4.50-beta" for MC 1.21.4)
    // For MC 1.20.x, versions look like "20.4.xxx"
    // For MC 1.21.x, versions look like "21.x.xxx"
    // We need to filter by MC version

    // Extract major.minor from MC version (e.g., "1.21.4" -> "21.4")
    let mc_parts: Vec<&str> = mc_version.split('.').collect();
    let neoforge_prefix = if mc_parts.len() >= 2 {
        // Remove the leading "1." and use major.minor
        if mc_parts[0] == "1" && mc_parts.len() >= 3 {
            format!("{}.{}", mc_parts[1], mc_parts[2])
        } else if mc_parts[0] == "1" && mc_parts.len() == 2 {
            format!("{}.", mc_parts[1])
        } else {
            mc_version.to_string()
        }
    } else {
        mc_version.to_string()
    };

    let versions: Vec<LoaderVersion> = maven_response
        .versions
        .into_iter()
        .filter(|v| v.starts_with(&neoforge_prefix))
        .map(|v| {
            let is_stable = !v.contains("beta") && !v.contains("alpha");
            LoaderVersion {
                version: v.clone(),
                maven: format!("net.neoforged:neoforge:{}", v),
                stable: is_stable,
                build: 0,
                separator: "-".to_string(),
            }
        })
        .collect();

    // Sort versions (newer first)
    let mut sorted_versions = versions;
    sorted_versions.sort_by(|a, b| {
        // Parse version numbers for proper sorting
        let a_nums: Vec<u32> = a
            .version
            .split(['.', '-'])
            .filter_map(|s| s.parse().ok())
            .collect();
        let b_nums: Vec<u32> = b
            .version
            .split(['.', '-'])
            .filter_map(|s| s.parse().ok())
            .collect();
        b_nums.cmp(&a_nums)
    });

    Ok(sorted_versions)
}

/// Fetch available LiteLoader versions (legacy loader)
pub async fn get_liteloader_versions(mc_version: &str) -> Result<Vec<LoaderVersion>, AppError> {
    let response = reqwest::get(LITELOADER_VERSIONS_URL)
        .await
        .map_err(AppError::HttpError)?;

    response.error_for_status_ref()?;

    let versions_response: LiteLoaderVersionsResponse =
        response.json().await.map_err(AppError::HttpError)?;

    // Get the versions for the specified MC version
    let mut versions: Vec<LoaderVersion> = Vec::new();

    if let Some(mc_versions) = versions_response.versions.get(mc_version) {
        if let Some(artefacts) = &mc_versions.artefacts {
            if let Some(liteloader_versions) = &artefacts.liteloader {
                for (key, artefact) in liteloader_versions {
                    if let Some(version) = &artefact.version {
                        // Determine stability - "RELEASE" streams are stable
                        let is_stable = key.to_uppercase().contains("RELEASE");

                        versions.push(LoaderVersion {
                            version: version.clone(),
                            maven: format!("com.mumfrey:liteloader:{}", version),
                            stable: is_stable,
                            build: 0,
                            separator: "-".to_string(),
                        });
                    }
                }
            }
        }
    }

    // Sort versions (newer first based on version string)
    versions.sort_by(|a, b| b.version.cmp(&a.version));

    Ok(versions)
}

/// Get loader versions for a specific loader type
pub async fn get_loader_versions(
    loader_type: LoaderType,
    mc_version: &str,
) -> Result<Vec<LoaderVersion>, AppError> {
    match loader_type {
        LoaderType::Fabric => get_fabric_versions(mc_version).await,
        LoaderType::Quilt => get_quilt_versions(mc_version).await,
        LoaderType::Forge => get_forge_versions(mc_version).await,
        LoaderType::NeoForge => get_neoforge_versions(mc_version).await,
        LoaderType::LiteLoader => get_liteloader_versions(mc_version).await,
        LoaderType::Vanilla | LoaderType::Unknown => Ok(vec![]),
    }
}

#[derive(Debug, Deserialize)]
struct QuiltInstallerMeta {
    url: String,
    version: String,
}

/// Fetch latest Quilt installer (sorted newest first by the API)
async fn fetch_latest_quilt_installer() -> Result<(String, String), AppError> {
    let url = format!("{}/versions/installer", QUILT_META_URL);

    let response = reqwest::get(&url).await.map_err(AppError::HttpError)?;

    response.error_for_status_ref()?;

    let installers: Vec<QuiltInstallerMeta> = response.json().await.map_err(AppError::HttpError)?;

    let latest = installers
        .into_iter()
        .next()
        .ok_or_else(|| AppError::InstallationError("No Quilt installers available".to_string()))?;

    Ok((latest.version, latest.url))
}

/// Download a file from URL to destination with progress callback
async fn download_file_with_progress(
    url: &str,
    destination: &Path,
    progress: impl Fn(u32),
) -> Result<(), AppError> {
    let response = reqwest::get(url).await.map_err(AppError::HttpError)?;

    let response = response
        .error_for_status()
        .map_err(|e| AppError::DownloadError(format!("Failed to download {}: {}", url, e)))?;

    let total_bytes = response.content_length();

    let bytes = response.bytes().await.map_err(AppError::HttpError)?;

    // Calculate progress if we have content-length, otherwise just report 100%
    let percent = if let Some(total) = total_bytes {
        ((bytes.len() as f64 / total as f64) * 100.0) as u32
    } else {
        100
    };
    progress(percent);

    tokio::fs::write(destination, bytes)
        .await
        .map_err(AppError::IoError)?;

    Ok(())
}

/// Create a minimal launcher_profiles.json for mod loader installers
async fn ensure_launcher_profiles(game_dir: &Path) -> Result<(), AppError> {
    let profiles_path = game_dir.join("launcher_profiles.json");

    if !profiles_path.exists() {
        // Create minimal launcher_profiles.json that Fabric/Quilt/Forge installers expect
        let minimal_profile = serde_json::json!({
            "profiles": {},
            "selectedProfile": null,
            "clientToken": "",
            "authenticationDatabase": {},
            "launcherVersion": {
                "name": "ETLauncher",
                "format": 21
            }
        });

        let content = serde_json::to_string_pretty(&minimal_profile)
            .map_err(|e| AppError::IoError(std::io::Error::other(e.to_string())))?;

        tokio::fs::write(&profiles_path, content)
            .await
            .map_err(AppError::IoError)?;
    }

    Ok(())
}

/// Install Fabric loader to a game directory
pub async fn install_fabric(
    game_dir: &Path,
    mc_version: &str,
    loader_version: &str,
    progress: impl Fn(String, u32),
) -> Result<(), AppError> {
    progress("Preparing installation...".to_string(), 0);

    // Ensure game directory exists
    tokio::fs::create_dir_all(game_dir)
        .await
        .map_err(AppError::IoError)?;

    // Create launcher_profiles.json if it doesn't exist (required by Fabric installer)
    ensure_launcher_profiles(game_dir).await?;

    progress("Downloading Fabric installer...".to_string(), 5);

    // Get latest stable installer version
    let installer_meta_url = format!("{}/versions/installer", FABRIC_META_URL);
    let response = reqwest::get(&installer_meta_url)
        .await
        .map_err(AppError::HttpError)?;

    let installers: Vec<FabricInstallerMeta> =
        response.json().await.map_err(AppError::HttpError)?;

    let installer_version = installers
        .first()
        .ok_or_else(|| AppError::DownloadError("No Fabric installer available".to_string()))?
        .version
        .clone();

    let installer_url = format!(
        "{}/fabric-installer/{}/fabric-installer-{}.jar",
        FABRIC_MAVEN_URL, installer_version, installer_version
    );

    let installer_path =
        std::env::temp_dir().join(format!("fabric-installer-{}.jar", installer_version));

    download_file_with_progress(&installer_url, &installer_path, |p| {
        progress("Downloading Fabric installer...".to_string(), 5 + (p / 3));
    })
    .await?;

    progress("Running Fabric installer...".to_string(), 40);

    let output = Command::new("java")
        .arg("-jar")
        .arg(&installer_path)
        .arg("client")
        .arg("-dir")
        .arg(game_dir)
        .arg("-mcversion")
        .arg(mc_version)
        .arg("-loader")
        .arg(loader_version)
        .arg("-noprofile") // Don't modify launcher profiles
        .current_dir(game_dir)
        .output()
        .await
        .map_err(|e| AppError::ProcessError(format!("Failed to run Fabric installer: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(AppError::ProcessError(format!(
            "Fabric installer failed: {} {}",
            stderr, stdout
        )));
    }

    // Cleanup
    let _ = tokio::fs::remove_file(&installer_path).await;

    progress("Verifying installation...".to_string(), 90);

    // Verify installation by checking for the version JSON
    let version_id = format!("fabric-loader-{}-{}", loader_version, mc_version);
    let version_json = game_dir
        .join("versions")
        .join(&version_id)
        .join(format!("{}.json", version_id));

    if !version_json.exists() {
        return Err(AppError::InstallationError(format!(
            "Fabric installation verification failed - version JSON not found at {:?}",
            version_json
        )));
    }

    progress("Installation complete".to_string(), 100);

    Ok(())
}

/// Install Quilt loader to a game directory
pub async fn install_quilt(
    game_dir: &Path,
    mc_version: &str,
    loader_version: &str,
    progress: impl Fn(String, u32),
) -> Result<(), AppError> {
    progress("Preparing installation...".to_string(), 0);

    // Resolve to latest available loader version for this MC version
    let resolved_loader_version = match get_quilt_versions(mc_version).await {
        Ok(mut versions) if !versions.is_empty() => versions.remove(0).version,
        _ => loader_version.to_string(),
    };

    // Ensure game directory exists
    tokio::fs::create_dir_all(game_dir)
        .await
        .map_err(AppError::IoError)?;

    // Create launcher_profiles.json if it doesn't exist (required by Quilt installer)
    ensure_launcher_profiles(game_dir).await?;

    progress("Downloading Quilt installer...".to_string(), 5);

    // Try to resolve the latest installer from Quilt meta; fall back to pinned version
    let (installer_version, installer_url) = match fetch_latest_quilt_installer().await {
        Ok((version, url)) => (version, url),
        Err(_) => (
            QUILT_INSTALLER_VERSION.to_string(),
            format!(
                "{}/{}/{}/quilt-installer-{}.jar",
                QUILT_MAVEN_URL,
                QUILT_INSTALLER_COORD,
                QUILT_INSTALLER_VERSION,
                QUILT_INSTALLER_VERSION
            ),
        ),
    };

    let installer_path =
        std::env::temp_dir().join(format!("quilt-installer-{}.jar", installer_version));

    // Ensure we don't reuse a bad download
    if installer_path.exists() {
        let _ = tokio::fs::remove_file(&installer_path).await;
    }

    download_file_with_progress(&installer_url, &installer_path, |p| {
        progress("Downloading Quilt installer...".to_string(), 5 + (p / 3));
    })
    .await?;

    // Validate installer archive before running it
    let installer_path_clone = installer_path.clone();
    tokio::task::spawn_blocking(move || -> Result<(), AppError> {
        let file = std::fs::File::open(&installer_path_clone)?;
        let archive = zip::ZipArchive::new(file)?;
        if archive.is_empty() {
            return Err(AppError::InstallationError(
                "Quilt installer archive is empty".to_string(),
            ));
        }
        Ok(())
    })
    .await
    .map_err(|e| {
        AppError::InstallationError(format!("Installer validation task failed: {}", e))
    })??;

    progress("Running Quilt installer...".to_string(), 40);

    let output = Command::new("java")
        .arg("-jar")
        .arg(&installer_path)
        .arg("install")
        .arg("client")
        .arg(mc_version)
        .arg(&resolved_loader_version)
        .arg(format!("--install-dir={}", game_dir.display()))
        .arg("--no-profile")
        .current_dir(game_dir)
        .output()
        .await
        .map_err(|e| AppError::ProcessError(format!("Failed to run Quilt installer: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(AppError::ProcessError(format!(
            "Quilt installer failed: {} {}",
            stderr, stdout
        )));
    }

    // Cleanup
    let _ = tokio::fs::remove_file(&installer_path).await;

    progress("Verifying installation...".to_string(), 90);

    // Verify installation by checking for the version JSON
    let versions_dir = game_dir.join("versions");
    let expected_id = format!("quilt-loader-{}-{}", resolved_loader_version, mc_version);
    let mut version_json = versions_dir
        .join(&expected_id)
        .join(format!("{}.json", expected_id));

    // If expected file isn't there, look for any quilt loader matching this MC version
    if !version_json.exists() && versions_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&versions_dir) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.starts_with("quilt-loader-") && name.ends_with(mc_version) {
                        version_json = versions_dir.join(&name).join(format!("{}.json", name));
                        break;
                    }
                }
            }
        }
    }

    if !version_json.exists() {
        return Err(AppError::InstallationError(format!(
            "Quilt installation verification failed - version JSON not found at {:?}",
            version_json
        )));
    }

    progress("Installation complete".to_string(), 100);

    Ok(())
}

/// Check if a Minecraft version is legacy (1.12.2 or earlier)
/// Legacy versions use different Forge installer arguments
fn is_legacy_mc_version(mc_version: &str) -> bool {
    // Parse major.minor version
    let parts: Vec<&str> = mc_version.split('.').collect();
    if parts.len() < 2 {
        return true; // Assume legacy if can't parse
    }

    let major: u32 = parts[0].parse().unwrap_or(1);
    let minor: u32 = parts[1].parse().unwrap_or(0);

    // 1.12 and earlier are legacy
    major == 1 && minor <= 12
}

/// Check if a Minecraft version is very old (before 1.15)
/// Very old Forge installers don't support --installClient at all
/// This includes 1.7.x through 1.14.x
fn is_very_old_mc_version(mc_version: &str) -> bool {
    let parts: Vec<&str> = mc_version.split('.').collect();
    if parts.len() < 2 {
        return true;
    }

    let major: u32 = parts[0].parse().unwrap_or(1);
    let minor: u32 = parts[1].parse().unwrap_or(0);

    // Before 1.15 don't support --installClient
    major == 1 && minor < 15
}

/// Extract old Forge installer manually (for pre-1.15 versions)
/// These installers don't support --installClient, so we extract the contents directly
async fn extract_old_forge_installer(
    game_dir: &Path,
    mc_version: &str,
    loader_version: &str,
    installer_path: &Path,
) -> Result<(), AppError> {
    let game_dir = game_dir.to_path_buf();
    let mc_version = mc_version.to_string();
    let loader_version = loader_version.to_string();
    let installer_path = installer_path.to_path_buf();

    tokio::task::spawn_blocking(move || {
        extract_old_forge_installer_sync(&game_dir, &mc_version, &loader_version, &installer_path)
    })
    .await
    .map_err(|e| AppError::InstallationError(format!("Task join error: {}", e)))?
}

/// Synchronous extraction of old Forge installer
fn extract_old_forge_installer_sync(
    game_dir: &Path,
    mc_version: &str,
    loader_version: &str,
    installer_path: &Path,
) -> Result<(), AppError> {
    use std::io::Read;

    let file = std::fs::File::open(installer_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Create necessary directories
    let versions_dir = game_dir.join("versions");
    let libraries_dir = game_dir.join("libraries");
    std::fs::create_dir_all(&versions_dir)?;
    std::fs::create_dir_all(&libraries_dir)?;

    // The version ID for old Forge - matches the format expected by the launcher
    let version_id = format!("{}-forge-{}", mc_version, loader_version);
    let version_dir = versions_dir.join(&version_id);
    std::fs::create_dir_all(&version_dir)?;

    // Look for install_profile.json to get the version info
    let mut version_json_content = None;
    let mut universal_jar_data = None;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        // Skip directory entries
        if name.ends_with('/') {
            continue;
        }

        if name == "install_profile.json" {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            eprintln!(
                "[forge] Found install_profile.json, length: {}",
                content.len()
            );

            // Parse the install profile to extract version info
            if let Ok(profile) = serde_json::from_str::<serde_json::Value>(&content) {
                // Old Forge install_profile.json has a "versionInfo" field
                if let Some(version_info) = profile.get("versionInfo") {
                    eprintln!("[forge] Found versionInfo in install_profile.json");
                    version_json_content = Some(serde_json::to_string_pretty(version_info)?);
                } else {
                    eprintln!(
                        "[forge] No versionInfo field, keys: {:?}",
                        profile.as_object().map(|o| o.keys().collect::<Vec<_>>())
                    );
                    // Try alternative: some installers have version.json separately or use different structure
                    if let Some(_install) = profile.get("install") {
                        eprintln!("[forge] Found 'install' field - this is a newer Forge installer format");
                    }
                }
            } else {
                eprintln!("[forge] Failed to parse install_profile.json as JSON");
            }
        } else if name == "version.json" {
            // Some Forge installers have version.json directly
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            eprintln!(
                "[forge] Found version.json directly, length: {}",
                content.len()
            );
            if version_json_content.is_none() {
                version_json_content = Some(content);
            }
        } else if name.contains("universal") && name.ends_with(".jar") {
            // Read the universal jar
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            universal_jar_data = Some((name, data));
        } else if name.starts_with("maven/") || (name.contains('/') && name.ends_with(".jar")) {
            // Extract library files
            let lib_path = if let Some(stripped) = name.strip_prefix("maven/") {
                libraries_dir.join(stripped)
            } else {
                libraries_dir.join(&name)
            };

            // Skip directories and handle conflicts
            if lib_path.is_dir() {
                eprintln!(
                    "[forge] Skipping {:?} - already exists as directory",
                    lib_path
                );
                continue;
            }

            if let Some(parent) = lib_path.parent() {
                // Check if parent path conflicts with a file
                if parent.is_file() {
                    eprintln!("[forge] Removing file at {:?} to create directory", parent);
                    std::fs::remove_file(parent)?;
                }
                std::fs::create_dir_all(parent)?;
            }

            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            std::fs::write(&lib_path, &data).map_err(|e| {
                AppError::InstallationError(format!("Failed to write {:?}: {}", lib_path, e))
            })?;
        }
    }

    // Write the version JSON
    if let Some(json_content) = version_json_content {
        // Update the id field in the JSON to match our expected version_id format
        let mut json_value: serde_json::Value = serde_json::from_str(&json_content)?;
        if let Some(obj) = json_value.as_object_mut() {
            obj.insert(
                "id".to_string(),
                serde_json::Value::String(version_id.clone()),
            );
        }
        let updated_json = serde_json::to_string_pretty(&json_value)?;

        let json_path = version_dir.join(format!("{}.json", version_id));
        if json_path.is_dir() {
            eprintln!(
                "[forge] Warning: {:?} is a directory, removing it",
                json_path
            );
            std::fs::remove_dir_all(&json_path)?;
        }
        std::fs::write(&json_path, &updated_json).map_err(|e| {
            AppError::InstallationError(format!("Failed to write {:?}: {}", json_path, e))
        })?;
        eprintln!("[forge] Wrote version JSON to {:?}", json_path);
    } else {
        return Err(AppError::InstallationError(
            "Could not find versionInfo in Forge installer".to_string(),
        ));
    }

    // Write the universal jar to the libraries directory
    if let Some((_jar_name, jar_data)) = universal_jar_data {
        // The jar should go to libraries/net/minecraftforge/forge/{mc}-{forge}/forge-{mc}-{forge}-universal.jar
        // Old Forge references libraries with :universal classifier
        let forge_lib_dir = libraries_dir
            .join("net/minecraftforge/forge")
            .join(format!("{}-{}", mc_version, loader_version));
        std::fs::create_dir_all(&forge_lib_dir)?;

        // Write with -universal suffix (what the version.json expects)
        let jar_path = forge_lib_dir.join(format!(
            "forge-{}-{}-universal.jar",
            mc_version, loader_version
        ));
        if jar_path.is_dir() {
            eprintln!(
                "[forge] Warning: {:?} is a directory, removing it",
                jar_path
            );
            std::fs::remove_dir_all(&jar_path)?;
        }
        std::fs::write(&jar_path, &jar_data).map_err(|e| {
            AppError::InstallationError(format!("Failed to write {:?}: {}", jar_path, e))
        })?;
        eprintln!("[forge] Wrote Forge universal jar to {:?}", jar_path);

        // Also write without suffix as fallback (some version.json variants reference it this way)
        let jar_path_alt =
            forge_lib_dir.join(format!("forge-{}-{}.jar", mc_version, loader_version));
        if jar_path_alt.is_dir() {
            eprintln!(
                "[forge] Warning: {:?} is a directory, removing it",
                jar_path_alt
            );
            std::fs::remove_dir_all(&jar_path_alt)?;
        }
        std::fs::write(&jar_path_alt, &jar_data).map_err(|e| {
            AppError::InstallationError(format!("Failed to write {:?}: {}", jar_path_alt, e))
        })?;
    }

    eprintln!(
        "[forge] Manual extraction complete for {} Forge {}",
        mc_version, loader_version
    );
    Ok(())
}

/// Install Forge loader to a game directory
pub async fn install_forge(
    game_dir: &Path,
    mc_version: &str,
    loader_version: &str,
    progress: impl Fn(String, u32),
) -> Result<(), AppError> {
    progress("Preparing installation...".to_string(), 0);

    // Ensure game directory exists
    tokio::fs::create_dir_all(game_dir)
        .await
        .map_err(AppError::IoError)?;

    // Create launcher_profiles.json if it doesn't exist (required by Forge installer)
    ensure_launcher_profiles(game_dir).await?;

    progress("Downloading Forge installer...".to_string(), 5);

    let installer_url = format!(
        "{}/{}-{}/forge-{}-{}-installer.jar",
        FORGE_MAVEN_URL, mc_version, loader_version, mc_version, loader_version
    );

    let installer_path = std::env::temp_dir().join(format!(
        "forge-installer-{}-{}.jar",
        mc_version, loader_version
    ));

    download_file_with_progress(&installer_url, &installer_path, |p| {
        progress("Downloading Forge installer...".to_string(), 5 + (p / 3));
    })
    .await?;

    progress("Running Forge installer...".to_string(), 40);

    let is_legacy = is_legacy_mc_version(mc_version);
    let is_very_old = is_very_old_mc_version(mc_version);

    // Get the correct Java for this MC version (old Forge needs Java 8)
    let required_java = java_service::get_required_java_version(mc_version);
    let java_path =
        java_service::get_installed_java(required_java).unwrap_or_else(|| "java".to_string());

    eprintln!(
        "[forge] Using Java {} at {} for MC {}",
        required_java, java_path, mc_version
    );

    let output = if is_very_old {
        // Very old Forge (pre-1.15) - these installers don't support --installClient
        // We need to extract the installer manually.
        eprintln!(
            "[forge] Using manual extraction for old Forge MC {}",
            mc_version
        );

        extract_old_forge_installer(game_dir, mc_version, loader_version, &installer_path).await?;

        // Return a fake successful output since we handled it manually
        std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: b"Manually extracted".to_vec(),
            stderr: Vec::new(),
        }
    } else if is_legacy {
        // Legacy Forge (1.8 - 1.12.2) - run headless with --installClient
        eprintln!("[forge] Using legacy installer mode for MC {}", mc_version);

        let dot_minecraft = game_dir.join(".minecraft");
        tokio::fs::create_dir_all(&dot_minecraft).await.ok();

        // Copy launcher_profiles.json to the .minecraft subfolder if needed
        let profiles_src = game_dir.join("launcher_profiles.json");
        let profiles_dst = dot_minecraft.join("launcher_profiles.json");
        if profiles_src.exists() && !profiles_dst.exists() {
            tokio::fs::copy(&profiles_src, &profiles_dst).await.ok();
        }

        Command::new(&java_path)
            .arg("-Djava.awt.headless=true")
            .arg("-jar")
            .arg(&installer_path)
            .arg("--installClient")
            .arg(&dot_minecraft)
            .current_dir(&dot_minecraft)
            .output()
            .await
            .map_err(|e| AppError::ProcessError(format!("Failed to run Forge installer: {}", e)))?
    } else {
        // Modern Forge (1.13+) - use --installClient
        Command::new(&java_path)
            .arg("-Djava.awt.headless=true")
            .arg("-jar")
            .arg(&installer_path)
            .arg("--installClient")
            .arg(game_dir)
            .current_dir(game_dir)
            .output()
            .await
            .map_err(|e| AppError::ProcessError(format!("Failed to run Forge installer: {}", e)))?
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(AppError::ProcessError(format!(
            "Forge installer failed: {} {}",
            stderr, stdout
        )));
    }

    // Cleanup installer
    let _ = tokio::fs::remove_file(&installer_path).await;

    progress("Verifying installation...".to_string(), 90);

    // For legacy installs, move files from .minecraft subfolder to game_dir if needed
    if is_legacy {
        let dot_minecraft = game_dir.join(".minecraft");
        if dot_minecraft.exists() {
            // Move versions folder
            let src_versions = dot_minecraft.join("versions");
            let dst_versions = game_dir.join("versions");
            if src_versions.exists() {
                move_dir_contents(&src_versions, &dst_versions).await?;
            }

            // Move libraries folder
            let src_libraries = dot_minecraft.join("libraries");
            let dst_libraries = game_dir.join("libraries");
            if src_libraries.exists() {
                move_dir_contents(&src_libraries, &dst_libraries).await?;
            }

            // Clean up .minecraft folder
            let _ = tokio::fs::remove_dir_all(&dot_minecraft).await;
        }
    }

    // Verify installation by checking for Forge version JSON
    // Try multiple possible version ID formats
    let possible_version_ids = vec![
        format!("{}-forge-{}", mc_version, loader_version),
        format!("{}-forge{}-{}", mc_version, mc_version, loader_version),
        format!("{}-Forge{}-{}", mc_version, mc_version, loader_version),
    ];

    let versions_dir = game_dir.join("versions");
    let mut found = false;

    for version_id in &possible_version_ids {
        let version_json = versions_dir
            .join(version_id)
            .join(format!("{}.json", version_id));
        if version_json.exists() {
            found = true;
            break;
        }
    }

    // Also check if any forge-related version exists
    if !found && versions_dir.exists() {
        if let Ok(mut entries) = tokio::fs::read_dir(&versions_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.to_lowercase().contains("forge") && name.contains(mc_version) {
                    found = true;
                    break;
                }
            }
        }
    }

    if !found {
        return Err(AppError::InstallationError(format!(
            "Forge installation verification failed - no Forge version found in {:?}",
            versions_dir
        )));
    }

    progress("Installation complete".to_string(), 100);

    Ok(())
}

/// Move contents of one directory to another
fn move_dir_contents<'a>(
    src: &'a Path,
    dst: &'a Path,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AppError>> + Send + 'a>> {
    Box::pin(async move {
        tokio::fs::create_dir_all(dst)
            .await
            .map_err(AppError::IoError)?;

        let mut entries = tokio::fs::read_dir(src).await.map_err(AppError::IoError)?;
        while let Some(entry) = entries.next_entry().await.map_err(AppError::IoError)? {
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                // Recursively move directory
                move_dir_contents(&src_path, &dst_path).await?;
                let _ = tokio::fs::remove_dir(&src_path).await;
            } else {
                // Move file (copy + delete since rename may fail across filesystems)
                if !dst_path.exists() {
                    tokio::fs::copy(&src_path, &dst_path)
                        .await
                        .map_err(AppError::IoError)?;
                }
                let _ = tokio::fs::remove_file(&src_path).await;
            }
        }

        Ok(())
    })
}

/// Install NeoForge loader to a game directory
pub async fn install_neoforge(
    game_dir: &Path,
    _mc_version: &str, // NeoForge versions are standalone, don't need MC version
    loader_version: &str,
    progress: impl Fn(String, u32),
) -> Result<(), AppError> {
    progress("Preparing installation...".to_string(), 0);

    // Ensure game directory exists
    tokio::fs::create_dir_all(game_dir)
        .await
        .map_err(AppError::IoError)?;

    // Create launcher_profiles.json if it doesn't exist (required by NeoForge installer)
    ensure_launcher_profiles(game_dir).await?;

    progress("Downloading NeoForge installer...".to_string(), 5);

    // NeoForge installer URL format: neoforge-{version}-installer.jar
    let installer_url = format!(
        "{}/{}/neoforge-{}-installer.jar",
        NEOFORGE_MAVEN_URL, loader_version, loader_version
    );

    let installer_path =
        std::env::temp_dir().join(format!("neoforge-installer-{}.jar", loader_version));

    download_file_with_progress(&installer_url, &installer_path, |p| {
        progress("Downloading NeoForge installer...".to_string(), 5 + (p / 3));
    })
    .await?;

    progress("Running NeoForge installer...".to_string(), 40);

    // NeoForge is for modern MC (1.20.1+), needs Java 21
    let java_path = java_service::get_installed_java(21).unwrap_or_else(|| "java".to_string());

    let output = Command::new(&java_path)
        .arg("-jar")
        .arg(&installer_path)
        .arg("--installClient")
        .arg(game_dir)
        .current_dir(game_dir)
        .output()
        .await
        .map_err(|e| AppError::ProcessError(format!("Failed to run NeoForge installer: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(AppError::ProcessError(format!(
            "NeoForge installer failed: {} {}",
            stderr, stdout
        )));
    }

    // Cleanup
    let _ = tokio::fs::remove_file(&installer_path).await;

    progress("Verifying installation...".to_string(), 90);

    // Verify installation - NeoForge version format: neoforge-{version}
    let version_id = format!("neoforge-{}", loader_version);
    let version_json = game_dir
        .join("versions")
        .join(&version_id)
        .join(format!("{}.json", version_id));

    if !version_json.exists() {
        return Err(AppError::InstallationError(format!(
            "NeoForge installation verification failed - version JSON not found at {:?}",
            version_json
        )));
    }

    progress("Installation complete".to_string(), 100);

    Ok(())
}

/// Install LiteLoader (legacy loader)
pub async fn install_liteloader(
    game_dir: &Path,
    mc_version: &str,
    loader_version: &str,
    progress: impl Fn(String, u32),
) -> Result<(), AppError> {
    progress("Downloading LiteLoader...".to_string(), 0);

    // LiteLoader is a simple JAR that goes into mods/ directory
    let liteloader_url = format!(
        "http://dl.liteloader.com/versions/{}-{}.jar",
        mc_version, loader_version
    );

    let mods_dir = game_dir.join("mods");
    let liteloader_path = mods_dir.join(format!("liteloader-{}.jar", loader_version));

    download_file_with_progress(&liteloader_url, &liteloader_path, |p| {
        progress("Downloading LiteLoader...".to_string(), p);
    })
    .await?;

    progress("Installation complete".to_string(), 100);

    Ok(())
}

/// Install a mod loader to a game directory
pub async fn install_loader(
    game_dir: &Path,
    loader_type: LoaderType,
    mc_version: &str,
    loader_version: &str,
    progress: impl Fn(String, u32),
) -> Result<(), AppError> {
    match loader_type {
        LoaderType::Fabric => install_fabric(game_dir, mc_version, loader_version, progress).await,
        LoaderType::Quilt => install_quilt(game_dir, mc_version, loader_version, progress).await,
        LoaderType::Forge => install_forge(game_dir, mc_version, loader_version, progress).await,
        LoaderType::NeoForge => {
            install_neoforge(game_dir, mc_version, loader_version, progress).await
        }
        LoaderType::LiteLoader => {
            install_liteloader(game_dir, mc_version, loader_version, progress).await
        }
        LoaderType::Vanilla | LoaderType::Unknown => Err(AppError::InvalidInput(
            "Cannot install Vanilla/Unknown loader".to_string(),
        )),
    }
}

/// Check if a loader is installed for a game directory
pub fn check_loader_installed(
    game_dir: &Path,
    loader_type: LoaderType,
    mc_version: &str,
    loader_version: &str,
) -> Result<bool, AppError> {
    match loader_type {
        LoaderType::Fabric => Ok(game_dir.join(".fabric").exists()),
        LoaderType::Quilt => Ok(game_dir.join(".quilt").exists()),
        LoaderType::Forge => {
            let version_dir = format!("{}-{}", mc_version, loader_version);
            let version_json = game_dir
                .join("versions")
                .join(&version_dir)
                .join(format!("{}.json", mc_version));
            Ok(version_json.exists())
        }
        LoaderType::NeoForge => {
            let version_dir = format!("{}-{}", mc_version, loader_version);
            let version_json = game_dir
                .join("versions")
                .join(&version_dir)
                .join(format!("{}.json", mc_version));
            Ok(version_json.exists())
        }
        LoaderType::LiteLoader => {
            let liteloader_jar = game_dir
                .join("mods")
                .join(format!("liteloader-{}.jar", loader_version));
            Ok(liteloader_jar.exists())
        }
        LoaderType::Vanilla | LoaderType::Unknown => Ok(true),
    }
}
