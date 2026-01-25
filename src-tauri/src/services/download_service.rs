use crate::error::AppError;
use crate::models::instance::{DownloadProgress, LoaderType};
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
use backoff::{backoff::Backoff, ExponentialBackoff};
use futures::stream::{self, StreamExt};
use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::Manager;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use zip::ZipArchive;

const VERSION_MANIFEST_URL: &str =
    "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";
const RESOURCES_URL: &str = "https://resources.download.minecraft.net";
/// Default concurrent downloads if settings not available
const DEFAULT_CONCURRENT_DOWNLOADS: usize = 8;

/// Fetch the version manifest from Mojang
pub async fn fetch_version_manifest(
    client: &reqwest::Client,
    force_refresh: bool,
) -> Result<VersionManifest, AppError> {
    let cache_path = get_versions_cache_dir().join("version_manifest_v2.json");

    // Use cached version if available and not forcing refresh
    if !force_refresh && cache_path.exists() {
        let content = fs::read_to_string(&cache_path)?;
        if let Ok(manifest) = serde_json::from_str(&content) {
            return Ok(manifest);
        }
    }

    // Fetch from Mojang
    let response = client.get(VERSION_MANIFEST_URL).send().await?;
    let content = response.text().await?;

    // Parse and cache
    let manifest: VersionManifest = serde_json::from_str(&content)?;

    fs::create_dir_all(get_versions_cache_dir())?;
    fs::write(&cache_path, &content)?;

    Ok(manifest)
}

/// Get a specific version's info
pub async fn get_version_info(
    client: &reqwest::Client,
    version_id: &str,
) -> Result<VersionInfo, AppError> {
    let cache_path = get_versions_cache_dir().join(format!("{}.json", version_id));

    // Use cached version if available
    if cache_path.exists() {
        let content = fs::read_to_string(&cache_path)?;
        let info: VersionInfo = serde_json::from_str(&content)?;
        return Ok(info);
    }

    // Fetch version manifest to get the URL
    let manifest = fetch_version_manifest(client, false).await?;
    let version_entry = manifest
        .versions
        .iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| AppError::VersionNotFound(version_id.to_string()))?;

    // Fetch version info
    let response = client.get(&version_entry.url).send().await?;
    let content = response.text().await?;

    // Parse and cache
    let info: VersionInfo = serde_json::from_str(&content)?;

    fs::write(&cache_path, &content)?;

    Ok(info)
}

/// Get the loader version ID string for a given loader type
pub fn get_loader_version_id(
    loader_type: &LoaderType,
    loader_version: &str,
    mc_version: &str,
) -> Option<String> {
    match loader_type {
        LoaderType::Vanilla | LoaderType::Unknown | LoaderType::Datapack => None,
        LoaderType::Fabric => Some(format!("fabric-loader-{}-{}", loader_version, mc_version)),
        LoaderType::Quilt => Some(format!("quilt-loader-{}-{}", loader_version, mc_version)),
        LoaderType::Forge => Some(format!("{}-forge-{}", mc_version, loader_version)),
        LoaderType::NeoForge => Some(format!("neoforge-{}", loader_version)),
        LoaderType::LiteLoader => Some(format!("liteloader-{}", loader_version)),
    }
}

/// Load a loader's version JSON from the game directory
pub fn load_loader_version_info(
    game_dir: &std::path::Path,
    loader_version_id: &str,
) -> Result<VersionInfo, AppError> {
    let version_json_path = game_dir
        .join("versions")
        .join(loader_version_id)
        .join(format!("{}.json", loader_version_id));

    let resolved_path = if version_json_path.exists() {
        version_json_path
    } else {
        // Try to find a fallback matching this loader type and MC version (helps when installer picks a newer loader)
        find_matching_loader_json(game_dir, loader_version_id).ok_or_else(|| {
            AppError::LoaderNotInstalled(format!(
                "Loader version JSON not found: {:?}",
                version_json_path
            ))
        })?
    };

    let content = fs::read_to_string(&resolved_path)?;
    let info: VersionInfo = serde_json::from_str(&content)?;
    Ok(info)
}

/// Find a loader version JSON for a given loader_version_id prefix (e.g., quilt-loader-x-y or fabric-loader-x-y)
fn find_matching_loader_json(
    game_dir: &std::path::Path,
    loader_version_id: &str,
) -> Option<std::path::PathBuf> {
    let versions_dir = game_dir.join("versions");
    if !versions_dir.exists() {
        return None;
    }

    // Extract mc_version by taking substring after last '-' (loader_version_id format: <loader>-<loader_ver>-<mc>)
    let mc_version = loader_version_id.rsplit('-').next()?;
    let loader_prefix = if let Some(idx) = loader_version_id.find('-') {
        &loader_version_id[..idx]
    } else {
        loader_version_id
    };

    let mut candidates = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&versions_dir) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.starts_with(loader_prefix) && name.ends_with(mc_version) {
                    let candidate = versions_dir.join(&name).join(format!("{}.json", name));
                    if candidate.exists() {
                        candidates.push((name, candidate));
                    }
                }
            }
        }
    }

    // Prefer the lexicographically highest loader version (newest)
    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    candidates.first().map(|(_, path)| path.clone())
}

/// Extract the artifact key (group:artifact:classifier) from a Maven coordinate
/// This preserves the classifier (if present) to avoid deduplicating native variants
/// e.g., "org.ow2.asm:asm:9.9" -> "org.ow2.asm:asm"
/// e.g., "org.lwjgl:lwjgl:3.3.3:natives-linux" -> "org.lwjgl:lwjgl:natives-linux"
fn get_library_artifact_key(name: &str) -> String {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() >= 4 {
        // Has classifier (group:artifact:version:classifier)
        format!("{}:{}:{}", parts[0], parts[1], parts[3])
    } else if parts.len() >= 2 {
        // No classifier (group:artifact:version)
        format!("{}:{}", parts[0], parts[1])
    } else {
        name.to_string()
    }
}

/// Deduplicate libraries by artifact key (group:artifact)
/// When duplicates exist:
/// 1. Prefer entries that apply to the current platform
/// 2. Prefer entries with natives/classifiers over those without
///
/// This ensures loader libraries take precedence, but native-providing entries aren't lost
fn deduplicate_libraries(
    libraries: Vec<crate::models::minecraft::Library>,
) -> Vec<crate::models::minecraft::Library> {
    use std::collections::HashMap;

    let mut seen: HashMap<String, crate::models::minecraft::Library> = HashMap::new();

    for lib in libraries {
        let key = get_library_artifact_key(&lib.name);

        // Check if this library applies to current platform
        let current_applies = should_use_library(&lib);

        if let Some(existing) = seen.get(&key) {
            let existing_applies = should_use_library(existing);

            // Check if libs have natives/classifiers
            let existing_has_natives = existing.natives.is_some()
                || existing
                    .downloads
                    .as_ref()
                    .map(|d| d.classifiers.is_some())
                    .unwrap_or(false);
            let current_has_natives = lib.natives.is_some()
                || lib
                    .downloads
                    .as_ref()
                    .map(|d| d.classifiers.is_some())
                    .unwrap_or(false);

            // Decide whether to replace:
            // 1. If current applies but existing doesn't, replace
            // 2. If both apply (or both don't), prefer the one with natives
            let should_replace = if current_applies && !existing_applies {
                true
            } else if !current_applies && existing_applies {
                false
            } else {
                // Both apply or both don't - prefer natives
                current_has_natives && !existing_has_natives
            };

            if should_replace {
                seen.insert(key, lib);
            }
        } else {
            seen.insert(key, lib);
        }
    }

    // Preserve original order as much as possible
    seen.into_values().collect()
}

/// Merge a loader version with its parent (vanilla) version
/// The loader version overrides main_class and adds its libraries
pub fn merge_version_info(loader_info: VersionInfo, parent_info: VersionInfo) -> VersionInfo {
    // Concatenate libraries with loader first, then deduplicate by artifact key
    // This ensures loader versions take precedence when there are conflicts
    let all_libraries = [loader_info.libraries, parent_info.libraries].concat();
    let deduplicated_libraries = deduplicate_libraries(all_libraries);

    VersionInfo {
        // Use loader's ID
        id: loader_info.id,
        // Use loader's main class (this is the key part!)
        main_class: loader_info.main_class,
        // Merge arguments - loader args come first, then parent
        minecraft_arguments: loader_info
            .minecraft_arguments
            .or(parent_info.minecraft_arguments),
        arguments: merge_arguments(loader_info.arguments, parent_info.arguments),
        // Deduplicated libraries - loader versions take precedence
        libraries: deduplicated_libraries,
        // Use parent's asset info (loader versions don't have these)
        asset_index: loader_info.asset_index.or(parent_info.asset_index),
        downloads: loader_info.downloads.or(parent_info.downloads),
        java_version: loader_info.java_version.or(parent_info.java_version),
        version_type: loader_info.version_type.or(parent_info.version_type),
        assets: loader_info.assets.or(parent_info.assets),
        inherits_from: None, // Merged version doesn't inherit from anything
    }
}

/// Merge arguments from loader and parent versions
fn merge_arguments(
    loader_args: Option<crate::models::minecraft::Arguments>,
    parent_args: Option<crate::models::minecraft::Arguments>,
) -> Option<crate::models::minecraft::Arguments> {
    match (loader_args, parent_args) {
        (Some(loader), Some(parent)) => Some(crate::models::minecraft::Arguments {
            game: [loader.game, parent.game].concat(),
            jvm: [loader.jvm, parent.jvm].concat(),
        }),
        (Some(loader), None) => Some(loader),
        (None, Some(parent)) => Some(parent),
        (None, None) => None,
    }
}

/// Get version info with loader support - merges loader version if applicable
pub async fn get_version_info_with_loader(
    client: &reqwest::Client,
    mc_version: &str,
    loader_type: &LoaderType,
    loader_version: Option<&str>,
    game_dir: &std::path::Path,
) -> Result<VersionInfo, AppError> {
    // Always start with vanilla version
    let vanilla_info = get_version_info(client, mc_version).await?;

    // If no loader or vanilla, return vanilla version
    if *loader_type == LoaderType::Vanilla {
        return Ok(vanilla_info);
    }

    // Get loader version string
    let loader_ver = loader_version
        .ok_or_else(|| AppError::LoaderNotInstalled("Loader version not specified".to_string()))?;

    // Get the loader version ID
    let loader_version_id = get_loader_version_id(loader_type, loader_ver, mc_version)
        .ok_or_else(|| AppError::LoaderNotInstalled("Unknown loader type".to_string()))?;

    // Try to load the loader version JSON
    let loader_info = load_loader_version_info(game_dir, &loader_version_id)?;

    // Merge loader with vanilla
    Ok(merge_version_info(loader_info, vanilla_info))
}

/// Download all game files for an instance
pub async fn download_game_files(
    instance_id: &str,
    version_id: &str,
    app_handle: Option<&AppHandle>,
) -> Result<(), AppError> {
    // Get HTTP client from AppState, or create a new one if not available
    let client = app_handle
        .and_then(|handle| handle.try_state::<crate::state::AppState>())
        .map(|state| state.http_client.clone())
        .unwrap_or_else(|| {
            crate::utils::http::create_client().expect("Failed to create HTTP client")
        });

    let version_info = get_version_info(&client, version_id).await?;
    download_game_files_with_version(instance_id, version_id, &version_info, app_handle).await
}

/// Download all game files for an instance using a pre-built VersionInfo
/// This is used when we have a merged version (vanilla + loader)
pub async fn download_game_files_with_version(
    instance_id: &str,
    version_id: &str,
    version_info: &VersionInfo,
    app_handle: Option<&AppHandle>,
) -> Result<(), AppError> {
    // Get HTTP client and settings from AppState
    let (http_client, instances_base_dir, concurrent_downloads) = app_handle
        .and_then(|handle| handle.try_state::<crate::state::AppState>())
        .map(|state| {
            let settings = state.settings.read();
            (
                state.http_client.clone(),
                settings.instances_path.clone(),
                settings.concurrent_downloads as usize,
            )
        })
        .unwrap_or_else(|| {
            (
                crate::utils::http::create_client().expect("Failed to create HTTP client"),
                crate::utils::paths::get_instances_dir()
                    .to_string_lossy()
                    .to_string(),
                DEFAULT_CONCURRENT_DOWNLOADS,
            )
        });

    let instance_dir = get_instance_dir_with_base(&instances_base_dir, instance_id);
    let game_dir = instance_dir.join(".minecraft");
    let game_libraries_dir = game_dir.join("libraries");
    let natives_dir = get_instance_natives_dir_with_base(&instances_base_dir, instance_id);

    // Collect all downloads needed
    let mut downloads: Vec<DownloadTask> = Vec::new();

    // Track all native JARs that need extraction (whether cached or downloaded)
    let mut all_natives: Vec<PathBuf> = Vec::new();

    // 1. Client JAR
    let client_jar_path = get_versions_cache_dir()
        .join(version_id)
        .join(format!("{}.jar", version_id));
    if let Some(ref dl) = version_info.downloads {
        if !file_valid(&client_jar_path, &dl.client.sha1) {
            downloads.push(DownloadTask {
                url: dl.client.url.clone(),
                path: client_jar_path,
                sha1: dl.client.sha1.clone(),
                size: dl.client.size,
                is_native: false,
            });
        }
    }

    // 2. Libraries
    for library in &version_info.libraries {
        if !should_use_library(library) {
            continue;
        }

        // Regular library artifact (Mojang-style)
        if let Some(ref lib_downloads) = library.downloads {
            // Handle native classifiers FIRST (before any continue statements)
            // This ensures natives are always collected even if the main artifact is cached
            if let Some(ref classifiers) = lib_downloads.classifiers {
                let natives_key = get_natives_key(library);
                if let Some(key) = natives_key {
                    if let Some(native_artifact) = classifiers.get(&key) {
                        let native_path = get_libraries_dir().join(&native_artifact.path);

                        // Always track this native for extraction
                        all_natives.push(native_path.clone());

                        // Only download if not already cached
                        if !file_valid(&native_path, &native_artifact.sha1) {
                            downloads.push(DownloadTask {
                                url: native_artifact.url.clone(),
                                path: native_path,
                                sha1: native_artifact.sha1.clone(),
                                size: native_artifact.size,
                                is_native: true,
                            });
                        }
                    }
                }
            }

            // Handle main artifact
            if let Some(ref artifact) = lib_downloads.artifact {
                let cache_lib_path = get_libraries_dir().join(&artifact.path);
                let game_lib_path = game_libraries_dir.join(&artifact.path);

                // Check if this is a modern-style native library (name contains classifier like :natives-linux)
                // Modern Minecraft versions have natives as separate library entries instead of classifiers
                let is_modern_native = is_native_library(&library.name);
                if is_modern_native {
                    // Add to natives for extraction (use whichever path exists, or cache path)
                    if game_lib_path.exists() {
                        all_natives.push(game_lib_path.clone());
                    } else {
                        all_natives.push(cache_lib_path.clone());
                    }
                }

                // Skip if library exists in either cache or game directory (Forge installs to game dir)
                if file_valid(&cache_lib_path, &artifact.sha1) || game_lib_path.exists() {
                    continue;
                }

                // If artifact URL is empty, try to construct from Maven repos
                let url = if artifact.url.is_empty() {
                    // Try Minecraft libraries, NeoForge Maven, Forge Maven, then Maven Central
                    maven_name_to_url(&library.name, "https://libraries.minecraft.net/")
                        .or_else(|| {
                            maven_name_to_url(
                                &library.name,
                                "https://maven.neoforged.net/releases/",
                            )
                        })
                        .or_else(|| {
                            maven_name_to_url(&library.name, "https://maven.minecraftforge.net/")
                        })
                        .or_else(|| {
                            maven_name_to_url(&library.name, "https://repo1.maven.org/maven2")
                        })
                        .unwrap_or_default()
                } else {
                    artifact.url.clone()
                };

                if !url.is_empty() {
                    downloads.push(DownloadTask {
                        url,
                        path: cache_lib_path,
                        sha1: artifact.sha1.clone(),
                        size: artifact.size,
                        is_native: false,
                    });
                } else {
                    eprintln!("WARN: No URL for library {}", library.name);
                }
            }
        } else if let Some(ref base_url) = library.url {
            // Maven-style library (used by Fabric, Quilt, Forge, etc.)
            if let Some(path) = maven_name_to_path(&library.name) {
                let cache_lib_path = get_libraries_dir().join(&path);
                let game_lib_path = game_libraries_dir.join(&path);

                // Check both cache and game directory (Forge extracts to game dir)
                if !cache_lib_path.exists() && !game_lib_path.exists() {
                    // For old Forge, try -universal suffix if regular name fails
                    let lib_name_with_universal = if library.name.contains("minecraftforge:forge:")
                        && !library.name.contains(":universal")
                    {
                        format!("{}:universal", library.name)
                    } else {
                        library.name.clone()
                    };

                    // Also check for universal variant in game dir
                    let universal_path = maven_name_to_path(&lib_name_with_universal);
                    let universal_exists = universal_path
                        .as_ref()
                        .map(|p| game_libraries_dir.join(p).exists())
                        .unwrap_or(false);

                    if !universal_exists {
                        // Try to download with universal suffix for old Forge
                        if let Some(url) = maven_name_to_url(&lib_name_with_universal, base_url) {
                            let target_path = if lib_name_with_universal != library.name {
                                // Use universal path if we modified the name
                                universal_path
                                    .map(|p| get_libraries_dir().join(p))
                                    .unwrap_or(cache_lib_path)
                            } else {
                                cache_lib_path
                            };
                            downloads.push(DownloadTask {
                                url,
                                path: target_path,
                                sha1: String::new(),
                                size: 0,
                                is_native: false,
                            });
                        }
                    }
                }
            }
        } else {
            // Library with just a name (no downloads, no url) - try multiple Maven repos
            if let Some(path) = maven_name_to_path(&library.name) {
                let lib_path = get_libraries_dir().join(&path);
                if !lib_path.exists() {
                    // Try Minecraft libraries first (for old MC libs), then Forge, then Maven Central
                    let url = maven_name_to_url(&library.name, "https://libraries.minecraft.net/")
                        .or_else(|| {
                            maven_name_to_url(&library.name, "https://maven.minecraftforge.net/")
                        })
                        .or_else(|| {
                            maven_name_to_url(&library.name, "https://repo1.maven.org/maven2")
                        });

                    if let Some(url) = url {
                        downloads.push(DownloadTask {
                            url,
                            path: lib_path,
                            sha1: String::new(),
                            size: 0,
                            is_native: false,
                        });
                    }
                }
            }
        }
    }

    // 3. Assets
    let asset_index = fetch_asset_index(&http_client, version_info).await?;
    for asset in asset_index.objects.values() {
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

    // Extract natives first (even if no downloads needed)
    // This ensures natives are always present in the instance's natives dir
    if !all_natives.is_empty() {
        fs::create_dir_all(&natives_dir)?;
        for native_path in &all_natives {
            if native_path.exists() {
                extract_natives(native_path, &natives_dir)?;
            }
        }
    }

    if total_files == 0 {
        return Ok(());
    }

    // Download with progress tracking
    let completed_files = Arc::new(AtomicU64::new(0));
    let downloaded_bytes = Arc::new(AtomicU64::new(0));
    let current_file = Arc::new(Mutex::new(String::new()));
    let start_time = std::time::Instant::now();

    let results = stream::iter(downloads)
        .map(|task| {
            let client = http_client.clone();
            let completed_files = completed_files.clone();
            let downloaded_bytes = downloaded_bytes.clone();
            let current_file = current_file.clone();
            let app_handle_clone = app_handle.cloned();

            async move {
                // Update current file
                {
                    let mut cf = current_file.lock().await;
                    *cf = task
                        .path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                }

                // Emit progress event with speed calculation
                if let Some(ref handle) = app_handle_clone {
                    let bytes_so_far = downloaded_bytes.load(Ordering::Relaxed);
                    let elapsed_secs = start_time.elapsed().as_secs_f64();
                    let speed = if elapsed_secs > 0.1 {
                        (bytes_so_far as f64 / elapsed_secs) as u64
                    } else {
                        0
                    };

                    let progress = DownloadProgress {
                        total_files,
                        completed_files: completed_files.load(Ordering::Relaxed) as u32,
                        current_file: task
                            .path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        total_bytes,
                        downloaded_bytes: bytes_so_far,
                        speed_bytes_per_sec: speed,
                    };
                    let _ = handle.emit("download_progress", progress);
                }

                // Download file
                let result = download_file(&client, &task.url, &task.path, &task.sha1).await;

                if result.is_ok() {
                    completed_files.fetch_add(1, Ordering::Relaxed);
                    downloaded_bytes.fetch_add(task.size, Ordering::Relaxed);
                }

                result
            }
        })
        .buffer_unordered(concurrent_downloads)
        .collect::<Vec<_>>()
        .await;

    // Check for errors
    for result in results {
        result?;
    }

    Ok(())
}

/// Download a single file with streaming, incremental hashing, and retry logic
///
/// Uses exponential backoff with jitter for transient failures (network issues, 5xx errors).
/// Streams data directly to disk while computing hash to minimize memory usage.
/// Will retry up to 3 times with delays of ~1s, ~2s, ~4s before giving up.
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

    // Use a temp file for atomic writes
    let temp_path = path.with_extension("part");

    // Configure exponential backoff: initial 1s, max 8s, max elapsed 30s
    let mut backoff = ExponentialBackoff {
        current_interval: Duration::from_secs(1),
        initial_interval: Duration::from_secs(1),
        max_interval: Duration::from_secs(8),
        max_elapsed_time: Some(Duration::from_secs(30)),
        multiplier: 2.0,
        randomization_factor: 0.5,
        ..Default::default()
    };
    backoff.reset();

    loop {
        match download_file_streaming(client, url, &temp_path, expected_sha1).await {
            Ok(()) => {
                // Rename temp file to final destination (atomic on most filesystems)
                tokio::fs::rename(&temp_path, path).await?;
                return Ok(());
            }
            Err(e) => {
                // Clean up temp file on failure
                let _ = tokio::fs::remove_file(&temp_path).await;

                // Check if error is retryable (network errors, 5xx, 429)
                let is_retryable = matches!(&e,
                    AppError::DownloadError(msg) if msg.contains("Failed to fetch") ||
                        msg.contains("HTTP 5") ||
                        msg.contains("HTTP 429")
                );

                if !is_retryable {
                    return Err(e);
                }

                // Get next backoff duration, or give up
                match backoff.next_backoff() {
                    Some(duration) => {
                        tokio::time::sleep(duration).await;
                    }
                    None => {
                        // Max retries exceeded
                        return Err(e);
                    }
                }
            }
        }
    }
}

/// Stream download directly to file with incremental SHA1 hashing
///
/// This function streams data chunk-by-chunk to disk while computing the hash,
/// avoiding loading the entire file into memory. Memory usage stays constant
/// regardless of file size (~64KB buffer).
async fn download_file_streaming(
    client: &reqwest::Client,
    url: &str,
    path: &PathBuf,
    expected_sha1: &str,
) -> Result<(), AppError> {
    // Validate URL is not empty
    if url.is_empty() {
        return Err(AppError::DownloadError(
            "Cannot download: URL is empty".to_string(),
        ));
    }

    // Start download
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::DownloadError(format!("Failed to fetch {}: {}", url, e)))?;

    if !response.status().is_success() {
        return Err(AppError::DownloadError(format!(
            "HTTP {} for {}",
            response.status(),
            url
        )));
    }

    // Create file for writing
    let mut file = tokio::fs::File::create(path).await?;

    // Initialize hasher if we need to verify
    let mut hasher = if expected_sha1.is_empty() {
        None
    } else {
        Some(Sha1::new())
    };

    // Stream chunks directly to file
    let mut stream = response.bytes_stream();
    while let Some(chunk_result) = stream.next().await {
        let chunk =
            chunk_result.map_err(|e| AppError::DownloadError(format!("Stream error: {}", e)))?;

        // Update hash
        if let Some(ref mut h) = hasher {
            h.update(&chunk);
        }

        // Write chunk to file
        file.write_all(&chunk).await?;
    }

    // Ensure all data is flushed
    file.flush().await?;

    // Verify hash if required
    if let Some(h) = hasher {
        let hash = format!("{:x}", h.finalize());
        if hash != expected_sha1 {
            return Err(AppError::HashMismatch(url.to_string()));
        }
    }

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
async fn fetch_asset_index(
    client: &reqwest::Client,
    version_info: &VersionInfo,
) -> Result<AssetIndex, AppError> {
    let asset_index = version_info
        .asset_index
        .as_ref()
        .ok_or_else(|| AppError::AssetNotFound("Version has no asset index".to_string()))?;

    let index_path = get_assets_dir()
        .join("indexes")
        .join(format!("{}.json", asset_index.id));

    // Use cached version if valid
    if file_valid(&index_path, &asset_index.sha1) {
        let content = fs::read_to_string(&index_path)?;
        let index: AssetIndex = serde_json::from_str(&content)?;
        return Ok(index);
    }

    // Fetch from URL
    let response = client.get(&asset_index.url).send().await?;
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

/// Check if a library name indicates it's a native library for the current platform
/// Modern Minecraft versions use separate library entries with names like:
/// "org.lwjgl:lwjgl:3.3.3:natives-linux" instead of the old classifiers format
fn is_native_library(library_name: &str) -> bool {
    let os = get_os_name();
    let arch = get_arch();

    // Check for platform-specific native classifiers in the library name
    match os {
        "linux" => library_name.contains(":natives-linux"),
        "windows" => {
            library_name.contains(":natives-windows")
                || (arch == "aarch64" && library_name.contains(":natives-windows-arm64"))
                || (arch == "x86" && library_name.contains(":natives-windows-x86"))
        }
        "osx" => {
            library_name.contains(":natives-macos")
                || library_name.contains(":natives-osx")
                || (arch == "aarch64" && library_name.contains(":natives-macos-arm64"))
        }
        _ => false,
    }
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
/// game_dir is the instance's .minecraft directory (for Forge-installed libraries)
pub fn get_classpath(
    version_info: &VersionInfo,
    version_id: &str,
    game_dir: Option<&std::path::Path>,
) -> Vec<PathBuf> {
    let mut classpath = Vec::new();
    let game_libraries_dir = game_dir.map(|d| d.join("libraries"));

    // Add libraries
    for library in &version_info.libraries {
        if !should_use_library(library) {
            continue;
        }

        // Try standard downloads.artifact first
        if let Some(ref downloads) = library.downloads {
            if let Some(ref artifact) = downloads.artifact {
                let cache_path = get_libraries_dir().join(&artifact.path);

                // Check game directory first (Forge installs here), then cache
                if let Some(ref game_libs) = game_libraries_dir {
                    let game_path = game_libs.join(&artifact.path);
                    if game_path.exists() {
                        classpath.push(game_path);
                        continue;
                    }
                }

                classpath.push(cache_path);
                continue;
            }
        }

        // Fall back to Maven-style library (used by Fabric, Quilt, etc.)
        // Convert Maven coordinates (group:artifact:version) to path
        if let Some(path) = maven_name_to_path(&library.name) {
            let cache_path = get_libraries_dir().join(&path);

            // Check game directory first (for loader-installed libs), then cache
            if let Some(ref game_libs) = game_libraries_dir {
                let game_path = game_libs.join(&path);
                if game_path.exists() {
                    classpath.push(game_path);
                    continue;
                }
            }

            classpath.push(cache_path);
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

/// Convert Maven coordinates (group:artifact:version) to file path
/// e.g., "net.fabricmc:fabric-loader:0.15.0" -> "net/fabricmc/fabric-loader/0.15.0/fabric-loader-0.15.0.jar"
fn maven_name_to_path(name: &str) -> Option<String> {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return None;
    }

    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];

    // Handle classifier if present (e.g., "group:artifact:version:classifier")
    let classifier = if parts.len() > 3 {
        Some(parts[3])
    } else {
        None
    };

    let filename = if let Some(c) = classifier {
        format!("{}-{}-{}.jar", artifact, version, c)
    } else {
        format!("{}-{}.jar", artifact, version)
    };

    Some(format!("{}/{}/{}/{}", group, artifact, version, filename))
}

/// Convert Maven coordinates to download URL
fn maven_name_to_url(name: &str, base_url: &str) -> Option<String> {
    maven_name_to_path(name).map(|path| {
        let base = base_url.trim_end_matches('/');
        // URL-encode special characters in path components (like + in versions)
        let encoded_path = path
            .split('/')
            .map(|segment| segment.replace('%', "%25").replace(' ', "%20"))
            .collect::<Vec<_>>()
            .join("/");
        format!("{}/{}", base, encoded_path)
    })
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
        .filter(|v| match v.version_type.as_str() {
            "release" => true,
            "snapshot" => show_snapshots,
            "old_beta" | "old_alpha" => show_old_versions,
            _ => false,
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
fn replace_placeholders(
    s: &str,
    replacements: &std::collections::HashMap<String, String>,
) -> String {
    let mut result = s.to_string();
    for (key, value) in replacements {
        result = result.replace(&format!("${{{}}}", key), value);
    }
    result
}

#[allow(dead_code)]
struct DownloadTask {
    url: String,
    path: PathBuf,
    sha1: String,
    size: u64,
    is_native: bool,
}
