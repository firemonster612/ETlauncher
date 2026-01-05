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

/// Get the loader version ID string for a given loader type
pub fn get_loader_version_id(
    loader_type: &LoaderType,
    loader_version: &str,
    mc_version: &str,
) -> Option<String> {
    match loader_type {
        LoaderType::Vanilla => None,
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

    if !version_json_path.exists() {
        return Err(AppError::LoaderNotInstalled(format!(
            "Loader version JSON not found: {:?}",
            version_json_path
        )));
    }

    let content = fs::read_to_string(&version_json_path)?;
    let info: VersionInfo = serde_json::from_str(&content)?;
    Ok(info)
}

/// Merge a loader version with its parent (vanilla) version
/// The loader version overrides main_class and adds its libraries
pub fn merge_version_info(loader_info: VersionInfo, parent_info: VersionInfo) -> VersionInfo {
    VersionInfo {
        // Use loader's ID
        id: loader_info.id,
        // Use loader's main class (this is the key part!)
        main_class: loader_info.main_class,
        // Merge arguments - loader args come first, then parent
        minecraft_arguments: loader_info.minecraft_arguments.or(parent_info.minecraft_arguments),
        arguments: merge_arguments(loader_info.arguments, parent_info.arguments),
        // Loader libraries come first (they take precedence), then parent libraries
        libraries: [loader_info.libraries, parent_info.libraries].concat(),
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
    mc_version: &str,
    loader_type: &LoaderType,
    loader_version: Option<&str>,
    game_dir: &std::path::Path,
) -> Result<VersionInfo, AppError> {
    // Always start with vanilla version
    let vanilla_info = get_version_info(mc_version).await?;

    // If no loader or vanilla, return vanilla version
    if *loader_type == LoaderType::Vanilla {
        return Ok(vanilla_info);
    }

    // Get loader version string
    let loader_ver = loader_version.ok_or_else(|| {
        AppError::LoaderNotInstalled("Loader version not specified".to_string())
    })?;

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
    let version_info = get_version_info(version_id).await?;
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
    // Get instances base directory from settings
    let instances_base_dir = app_handle
        .and_then(|handle| {
            handle.try_state::<crate::state::AppState>()
                .map(|state| state.settings.read().instances_path.clone())
        })
        .unwrap_or_else(|| crate::utils::paths::get_instances_dir().to_string_lossy().to_string());

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

                // Skip if library exists in either cache or game directory (Forge installs to game dir)
                if file_valid(&cache_lib_path, &artifact.sha1) || game_lib_path.exists() {
                    continue;
                }

                // If artifact URL is empty, try to construct from Maven repos
                let url = if artifact.url.is_empty() {
                    // Try NeoForge Maven, Forge Maven, then Maven Central
                    maven_name_to_url(&library.name, "https://maven.neoforged.net/releases/")
                        .or_else(|| maven_name_to_url(&library.name, "https://maven.minecraftforge.net/"))
                        .or_else(|| maven_name_to_url(&library.name, "https://repo1.maven.org/maven2"))
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
                let lib_path = get_libraries_dir().join(&path);
                if !lib_path.exists() {
                    if let Some(url) = maven_name_to_url(&library.name, base_url) {
                        downloads.push(DownloadTask {
                            url,
                            path: lib_path,
                            sha1: String::new(), // Maven libs often don't have SHA1 in version JSON
                            size: 0,
                            is_native: false,
                        });
                    }
                }
            }
        } else {
            // Library with just a name (no downloads, no url) - try Maven Central
            if let Some(path) = maven_name_to_path(&library.name) {
                let lib_path = get_libraries_dir().join(&path);
                if !lib_path.exists() {
                    if let Some(url) = maven_name_to_url(&library.name, "https://repo1.maven.org/maven2") {
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
    let asset_index = fetch_asset_index(version_info).await?;
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

    let client = reqwest::Client::new();

    let results = stream::iter(downloads)
        .map(|task| {
            let client = client.clone();
            let completed_files = completed_files.clone();
            let downloaded_bytes = downloaded_bytes.clone();
            let current_file = current_file.clone();
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

    Ok(())
}

/// Download a single file with optional SHA1 verification
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
    let response = client.get(url).send().await.map_err(|e| {
        AppError::DownloadError(format!("Failed to fetch {}: {}", url, e))
    })?;
    if !response.status().is_success() {
        return Err(AppError::DownloadError(format!(
            "HTTP {} for {}",
            response.status(),
            url
        )));
    }

    let bytes = response.bytes().await?;

    // Verify SHA1 (skip if empty - used for Maven libraries)
    if !expected_sha1.is_empty() {
        let mut hasher = Sha1::new();
        hasher.update(&bytes);
        let hash = format!("{:x}", hasher.finalize());

        if hash != expected_sha1 {
            return Err(AppError::HashMismatch(path.display().to_string()));
        }
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
    let asset_index = version_info.asset_index.as_ref().ok_or_else(|| {
        AppError::AssetNotFound("Version has no asset index".to_string())
    })?;

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
    let client = reqwest::Client::new();
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
pub fn get_classpath(version_info: &VersionInfo, version_id: &str, game_dir: Option<&std::path::Path>) -> Vec<PathBuf> {
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
    let classifier = if parts.len() > 3 { Some(parts[3]) } else { None };

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
            .map(|segment| {
                segment
                    .replace('%', "%25")
                    .replace('+', "%2B")
                    .replace(' ', "%20")
            })
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
