use crate::error::AppError;
use crate::models::instance::DownloadProgress;
use crate::models::minecraft::{
    ArgumentValue, AssetIndex, Library, Rule, StringOrArray, VersionEntry, VersionInfo,
    VersionManifest,
};
use crate::utils::hash::verify_sha1;
use crate::utils::paths::{
    get_assets_dir, get_instance_dir_with_base, get_instance_natives_dir_with_base,
    get_libraries_dir, get_versions_cache_dir,
};
use crate::utils::platform::{get_arch, get_os_name};
use futures::stream::{self, StreamExt};
use tauri::Manager;
use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use zip::ZipArchive;

const VERSION_MANIFEST_URL: &str =
    "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";
const RESOURCES_URL: &str = "https://resources.download.minecraft.net";
const CONCURRENT_DOWNLOADS: usize = 8;

/// Fetch the version manifest from Mojang
pub async fn fetch_version_manifest(force_refresh: bool) -> Result<VersionManifest, AppError> {
    let cache_path = get_versions_cache_dir().join("version_manifest_v2.json");

    // Use cached version if available and not forcing refresh
    if !force_refresh && cache_path.exists() {
        let content = fs::read_to_string(&cache_path)?;
        if let Ok(manifest) = serde_json::from_str(&content) {
            return Ok(manifest);
        }
    }

    // Fetch from Mojang
    let client = reqwest::Client::new();
    let response = client.get(VERSION_MANIFEST_URL).send().await?;
    let content = response.text().await?;

    // Parse and cache
    let manifest: VersionManifest = serde_json::from_str(&content)?;

    fs::create_dir_all(get_versions_cache_dir())?;
    fs::write(&cache_path, &content)?;

    Ok(manifest)
}

/// Get a specific version's info
pub async fn get_version_info(version_id: &str) -> Result<VersionInfo, AppError> {
    let cache_path = get_versions_cache_dir().join(format!("{}.json", version_id));

    // Use cached version if available
    if cache_path.exists() {
        let content = fs::read_to_string(&cache_path)?;
        let info: VersionInfo = serde_json::from_str(&content)?;
        return Ok(info);
    }

    // Fetch version manifest to get the URL
    let manifest = fetch_version_manifest(false).await?;
    let version_entry = manifest
        .versions
        .iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| AppError::VersionNotFound(version_id.to_string()))?;

    // Fetch version info
    let client = reqwest::Client::new();
    let response = client.get(&version_entry.url).send().await?;
    let content = response.text().await?;

    // Parse and cache
    let info: VersionInfo = serde_json::from_str(&content)?;

    fs::write(&cache_path, &content)?;

    Ok(info)
}

/// Download all game files for an instance
pub async fn download_game_files(
    instance_id: &str,
    version_id: &str,
    app_handle: Option<&AppHandle>,
) -> Result<(), AppError> {
    let version_info = get_version_info(version_id).await?;

    // Get instances base directory from settings
    let instances_base_dir = app_handle
        .and_then(|handle| {
            handle.try_state::<crate::state::AppState>()
                .map(|state| state.settings.read().instances_path.clone())
        })
        .unwrap_or_else(|| crate::utils::paths::get_instances_dir().to_string_lossy().to_string());

    let _instance_dir = get_instance_dir_with_base(&instances_base_dir, instance_id);
    let natives_dir = get_instance_natives_dir_with_base(&instances_base_dir, instance_id);

    // Collect all downloads needed
    let mut downloads: Vec<DownloadTask> = Vec::new();

    // 1. Client JAR
    let client_jar_path = get_versions_cache_dir()
        .join(version_id)
        .join(format!("{}.jar", version_id));
    if !file_valid(&client_jar_path, &version_info.downloads.client.sha1) {
        downloads.push(DownloadTask {
            url: version_info.downloads.client.url.clone(),
            path: client_jar_path,
            sha1: version_info.downloads.client.sha1.clone(),
            size: version_info.downloads.client.size,
            is_native: false,
        });
    }

    // 2. Libraries
    for library in &version_info.libraries {
        if !should_use_library(library) {
            continue;
        }

        // Regular library artifact
        if let Some(ref lib_downloads) = library.downloads {
            if let Some(ref artifact) = lib_downloads.artifact {
                let lib_path = get_libraries_dir().join(&artifact.path);
                if !file_valid(&lib_path, &artifact.sha1) {
                    downloads.push(DownloadTask {
                        url: artifact.url.clone(),
                        path: lib_path,
                        sha1: artifact.sha1.clone(),
                        size: artifact.size,
                        is_native: false,
                    });
                }
            }

            // Native classifiers
            if let Some(ref classifiers) = lib_downloads.classifiers {
                let natives_key = get_natives_key(library);
                if let Some(key) = natives_key {
                    if let Some(native_artifact) = classifiers.get(&key) {
                        let native_path = get_libraries_dir().join(&native_artifact.path);
                        if !file_valid(&native_path, &native_artifact.sha1) {
                            downloads.push(DownloadTask {
                                url: native_artifact.url.clone(),
                                path: native_path.clone(),
                                sha1: native_artifact.sha1.clone(),
                                size: native_artifact.size,
                                is_native: true,
                            });
                        }
                    }
                }
            }
        }
    }

    // 3. Assets
    let asset_index = fetch_asset_index(&version_info).await?;
    for (_name, asset) in &asset_index.objects {
        let hash_prefix = &asset.hash[..2];
        let asset_path = get_assets_dir()
            .join("objects")
            .join(hash_prefix)
            .join(&asset.hash);
        if !file_valid(&asset_path, &asset.hash) {
            downloads.push(DownloadTask {
                url: format!("{}/{}/{}", RESOURCES_URL, hash_prefix, asset.hash),
                path: asset_path,
                sha1: asset.hash.clone(),
                size: asset.size,
                is_native: false,
            });
        }
    }

    // Calculate totals
    let total_files = downloads.len() as u32;
    let total_bytes: u64 = downloads.iter().map(|d| d.size).sum();

    if total_files == 0 {
        return Ok(());
    }

    // Download with progress tracking
    let completed_files = Arc::new(AtomicU64::new(0));
    let downloaded_bytes = Arc::new(AtomicU64::new(0));
    let current_file = Arc::new(Mutex::new(String::new()));
    let natives_to_extract = Arc::new(Mutex::new(Vec::new()));

    let client = reqwest::Client::new();

    let results = stream::iter(downloads)
        .map(|task| {
            let client = client.clone();
            let completed_files = completed_files.clone();
            let downloaded_bytes = downloaded_bytes.clone();
            let current_file = current_file.clone();
            let natives_to_extract = natives_to_extract.clone();
            let app_handle_clone = app_handle.cloned();

            async move {
                // Update current file
                {
                    let mut cf = current_file.lock().await;
                    *cf = task.path.file_name().unwrap_or_default().to_string_lossy().to_string();
                }

                // Emit progress event
                if let Some(ref handle) = app_handle_clone {
                    let progress = DownloadProgress {
                        total_files,
                        completed_files: completed_files.load(Ordering::Relaxed) as u32,
                        current_file: task.path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                        total_bytes,
                        downloaded_bytes: downloaded_bytes.load(Ordering::Relaxed),
                        speed_bytes_per_sec: 0,
                    };
                    let _ = handle.emit("download_progress", progress);
                }

                // Download file
                let result = download_file(&client, &task.url, &task.path, &task.sha1).await;

                if result.is_ok() {
                    completed_files.fetch_add(1, Ordering::Relaxed);
                    downloaded_bytes.fetch_add(task.size, Ordering::Relaxed);

                    if task.is_native {
                        natives_to_extract.lock().await.push(task.path.clone());
                    }
                }

                result
            }
        })
        .buffer_unordered(CONCURRENT_DOWNLOADS)
        .collect::<Vec<_>>()
        .await;

    // Check for errors
    for result in results {
        result?;
    }

    // Extract natives
    let natives_list = natives_to_extract.lock().await;
    for native_path in natives_list.iter() {
        extract_natives(native_path, &natives_dir)?;
    }

    Ok(())
}

/// Download a single file with SHA1 verification
async fn download_file(
    client: &reqwest::Client,
    url: &str,
    path: &PathBuf,
    expected_sha1: &str,
) -> Result<(), AppError> {
    // Create parent directories
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Download
    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        return Err(AppError::DownloadError(format!(
            "HTTP {} for {}",
            response.status(),
            url
        )));
    }

    let bytes = response.bytes().await?;

    // Verify SHA1
    let mut hasher = Sha1::new();
    hasher.update(&bytes);
    let hash = format!("{:x}", hasher.finalize());

    if hash != expected_sha1 {
        return Err(AppError::HashMismatch(path.display().to_string()));
    }

    // Write to file
    let mut file = File::create(path)?;
    file.write_all(&bytes)?;

    Ok(())
}

/// Check if a file exists and has the correct SHA1 hash
fn file_valid(path: &PathBuf, expected_sha1: &str) -> bool {
    if !path.exists() {
        return false;
    }

    verify_sha1(path, expected_sha1).unwrap_or(false)
}

/// Fetch and cache the asset index
async fn fetch_asset_index(version_info: &VersionInfo) -> Result<AssetIndex, AppError> {
    let index_path = get_assets_dir()
        .join("indexes")
        .join(format!("{}.json", version_info.asset_index.id));

    // Use cached version if valid
    if file_valid(&index_path, &version_info.asset_index.sha1) {
        let content = fs::read_to_string(&index_path)?;
        let index: AssetIndex = serde_json::from_str(&content)?;
        return Ok(index);
    }

    // Fetch from URL
    let client = reqwest::Client::new();
    let response = client.get(&version_info.asset_index.url).send().await?;
    let content = response.text().await?;

    // Parse and cache
    let index: AssetIndex = serde_json::from_str(&content)?;

    fs::create_dir_all(index_path.parent().unwrap())?;
    fs::write(&index_path, &content)?;

    Ok(index)
}

/// Check if a library should be used based on rules
fn should_use_library(library: &Library) -> bool {
    if library.rules.is_none() {
        return true;
    }

    let rules = library.rules.as_ref().unwrap();
    let mut allow = false;

    for rule in rules {
        let matches = rule_matches(rule);

        if rule.action == "allow" && matches {
            allow = true;
        } else if rule.action == "disallow" && matches {
            allow = false;
        }
    }

    allow
}

/// Check if a rule matches the current platform
fn rule_matches(rule: &Rule) -> bool {
    if let Some(ref os_rule) = rule.os {
        let current_os = get_os_name();
        let current_arch = get_arch();

        if let Some(ref name) = os_rule.name {
            if name != current_os {
                return false;
            }
        }

        if let Some(ref arch) = os_rule.arch {
            if arch != current_arch {
                return false;
            }
        }
    }

    true
}

/// Get the natives key for the current platform
fn get_natives_key(library: &Library) -> Option<String> {
    let natives = library.natives.as_ref()?;
    let os = get_os_name();

    let key = natives.get(os)?;

    // Replace arch placeholder
    let arch = get_arch();
    let bits = if arch == "x86_64" { "64" } else { "32" };
    Some(key.replace("${arch}", bits))
}

/// Extract native libraries from a JAR file
fn extract_natives(jar_path: &PathBuf, natives_dir: &PathBuf) -> Result<(), AppError> {
    fs::create_dir_all(natives_dir)?;

    let file = File::open(jar_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        // Skip directories and META-INF
        if name.ends_with('/') || name.starts_with("META-INF") {
            continue;
        }

        // Only extract native libraries
        let is_native = name.ends_with(".so")
            || name.ends_with(".dylib")
            || name.ends_with(".dll")
            || name.ends_with(".jnilib");

        if is_native {
            let out_path = natives_dir.join(&name);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut out_file = File::create(&out_path)?;
            let mut buffer = Vec::new();
            entry.read_to_end(&mut buffer)?;
            out_file.write_all(&buffer)?;
        }
    }

    Ok(())
}

/// Get the classpath for launching
pub fn get_classpath(version_info: &VersionInfo, version_id: &str) -> Vec<PathBuf> {
    let mut classpath = Vec::new();

    // Add libraries
    for library in &version_info.libraries {
        if !should_use_library(library) {
            continue;
        }

        if let Some(ref downloads) = library.downloads {
            if let Some(ref artifact) = downloads.artifact {
                classpath.push(get_libraries_dir().join(&artifact.path));
            }
        }
    }

    // Add client JAR
    classpath.push(
        get_versions_cache_dir()
            .join(version_id)
            .join(format!("{}.jar", version_id)),
    );

    classpath
}

/// Get version entries filtered by type
pub fn filter_versions(
    manifest: &VersionManifest,
    show_snapshots: bool,
    show_old_versions: bool,
) -> Vec<&VersionEntry> {
    manifest
        .versions
        .iter()
        .filter(|v| {
            match v.version_type.as_str() {
                "release" => true,
                "snapshot" => show_snapshots,
                "old_beta" | "old_alpha" => show_old_versions,
                _ => false,
            }
        })
        .collect()
}

/// Build game arguments from version info
pub fn build_game_arguments(
    version_info: &VersionInfo,
    replacements: &std::collections::HashMap<String, String>,
) -> Vec<String> {
    let mut args = Vec::new();

    // Modern format (1.13+)
    if let Some(ref arguments) = version_info.arguments {
        for arg in &arguments.game {
            match arg {
                ArgumentValue::Simple(s) => {
                    args.push(replace_placeholders(s, replacements));
                }
                ArgumentValue::Conditional { rules, value } => {
                    // Check if rules match (skip for now, add feature detection later)
                    let matches = rules.iter().all(|r| {
                        // Skip feature-based rules
                        r.features.is_none() && rule_matches(r)
                    });

                    if matches {
                        match value {
                            StringOrArray::Single(s) => {
                                args.push(replace_placeholders(s, replacements));
                            }
                            StringOrArray::Multiple(arr) => {
                                for s in arr {
                                    args.push(replace_placeholders(s, replacements));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // Legacy format (pre-1.13)
    else if let Some(ref mc_args) = version_info.minecraft_arguments {
        for arg in mc_args.split_whitespace() {
            args.push(replace_placeholders(arg, replacements));
        }
    }

    args
}

/// Build JVM arguments from version info
pub fn build_jvm_arguments(
    version_info: &VersionInfo,
    replacements: &std::collections::HashMap<String, String>,
) -> Vec<String> {
    let mut args = Vec::new();

    // Modern format (1.13+)
    if let Some(ref arguments) = version_info.arguments {
        for arg in &arguments.jvm {
            match arg {
                ArgumentValue::Simple(s) => {
                    args.push(replace_placeholders(s, replacements));
                }
                ArgumentValue::Conditional { rules, value } => {
                    let matches = rules.iter().all(rule_matches);

                    if matches {
                        match value {
                            StringOrArray::Single(s) => {
                                args.push(replace_placeholders(s, replacements));
                            }
                            StringOrArray::Multiple(arr) => {
                                for s in arr {
                                    args.push(replace_placeholders(s, replacements));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    args
}

/// Replace placeholders in argument strings
fn replace_placeholders(s: &str, replacements: &std::collections::HashMap<String, String>) -> String {
    let mut result = s.to_string();
    for (key, value) in replacements {
        result = result.replace(&format!("${{{}}}", key), value);
    }
    result
}

struct DownloadTask {
    url: String,
    path: PathBuf,
    sha1: String,
    size: u64,
    is_native: bool,
}
