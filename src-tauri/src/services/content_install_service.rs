use crate::error::AppError;
use crate::models::{
    Content, ContentDownloadProgress, ContentDownloadProgressWithId, ContentPlatform,
    ContentSource, ContentType, ContentVersion, DependencyType, InstalledContent, LoaderType,
    ResolvedDependency,
};
use crate::services::{
    curseforge_service, manifest_service, modrinth_service, resource_pool_service,
};
use crate::state::AppState;
use crate::utils::hash::murmur2_bytes;
use crate::utils::paths::get_instance_game_dir_with_base;
use chrono::Utc;
use futures::StreamExt;
use reqwest::Client;
use sha1::{Digest as Sha1Digest, Sha1};
use sha2::{Digest as Sha2Digest, Sha512};
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;
use zip::ZipArchive;

/// Get the primary filename from a content version
fn get_primary_filename(version: &ContentVersion) -> Option<String> {
    version
        .files
        .iter()
        .find(|f| f.primary)
        .or_else(|| version.files.first())
        .map(|f| f.filename.clone())
}

/// Extract a world ZIP file to the saves directory
/// Returns the name of the extracted world folder
fn extract_world_zip(zip_bytes: &[u8], saves_dir: &PathBuf) -> Result<String, AppError> {
    let cursor = Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| AppError::DownloadError(format!("Failed to read world ZIP: {}", e)))?;

    // Find the world folder by looking for level.dat
    let mut world_folder_prefix: Option<String> = None;

    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| AppError::DownloadError(format!("Failed to read ZIP entry: {}", e)))?;
        let name = file.name();

        // Look for level.dat to identify the world folder
        if name.ends_with("level.dat") {
            // The world folder is the parent of level.dat
            // Could be "WorldName/level.dat" or just "level.dat"
            if let Some(parent) = name.strip_suffix("level.dat") {
                let parent = parent.trim_end_matches('/');
                if parent.is_empty() {
                    // level.dat is at root - world files are at root level
                    world_folder_prefix = Some(String::new());
                } else {
                    // level.dat is in a subfolder
                    world_folder_prefix = Some(parent.to_string());
                }
            }
            break;
        }
    }

    let prefix = world_folder_prefix
        .ok_or_else(|| AppError::DownloadError("No level.dat found in world ZIP".to_string()))?;

    // Determine the world folder name
    let world_name = if prefix.is_empty() {
        // Files are at root, use the ZIP filename without extension as world name
        "Imported World".to_string()
    } else {
        // Use the folder name from the ZIP
        prefix
            .split('/')
            .next()
            .unwrap_or("Imported World")
            .to_string()
    };

    // Create destination path, handling conflicts
    let mut dest_path = saves_dir.join(&world_name);
    let mut counter = 1;
    while dest_path.exists() {
        dest_path = saves_dir.join(format!("{} ({})", world_name, counter));
        counter += 1;
    }

    let final_world_name = dest_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&world_name)
        .to_string();

    fs::create_dir_all(&dest_path)?;

    // Canonicalize dest_path to get absolute path for zip-slip protection
    let dest_path_canonical = dest_path.canonicalize().map_err(|e| {
        AppError::DownloadError(format!("Failed to resolve destination path: {}", e))
    })?;

    // Re-open archive for extraction
    let cursor = Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| AppError::DownloadError(format!("Failed to read world ZIP: {}", e)))?;

    // Extract files
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| AppError::DownloadError(format!("Failed to read ZIP entry: {}", e)))?;

        let file_path = file.name().to_string();

        // Skip files outside the world folder (if prefix is set)
        let relative_path = if prefix.is_empty() {
            file_path.clone()
        } else if let Some(rel) = file_path.strip_prefix(&format!("{}/", prefix)) {
            rel.to_string()
        } else if file_path == prefix {
            // This is the folder itself
            continue;
        } else {
            // File is outside world folder, skip
            continue;
        };

        if relative_path.is_empty() {
            continue;
        }

        // Zip-slip protection: ensure the resolved path is within dest_path
        let out_path = dest_path.join(&relative_path);
        let out_path_canonical = match out_path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                // Path doesn't exist yet, check parent and construct
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                // Re-check after creating parent dirs
                let parent_canonical = out_path
                    .parent()
                    .and_then(|p| p.canonicalize().ok())
                    .unwrap_or_else(|| dest_path_canonical.clone());
                let file_name = out_path.file_name().ok_or_else(|| {
                    AppError::DownloadError("Invalid file path in ZIP".to_string())
                })?;
                parent_canonical.join(file_name)
            }
        };

        // Verify the path is within the destination directory (zip-slip protection)
        if !out_path_canonical.starts_with(&dest_path_canonical) {
            return Err(AppError::DownloadError(format!(
                "Zip entry attempts to escape destination: {}",
                relative_path
            )));
        }

        if file.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            // Ensure parent directory exists
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut out_file = File::create(&out_path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).map_err(|e| {
                AppError::DownloadError(format!("Failed to read file from ZIP: {}", e))
            })?;
            out_file.write_all(&buffer)?;
        }
    }

    Ok(final_world_name)
}

/// Result of downloading a file including computed hashes and bytes
struct DownloadResult {
    sha512_hash: String,
    murmur2_fingerprint: u32,
    bytes: Vec<u8>,
}

/// Get the content install directory based on content type
fn get_content_dir(game_dir: &PathBuf, content_type: &ContentType) -> PathBuf {
    match content_type {
        ContentType::Mod => game_dir.join("mods"),
        ContentType::Shader => game_dir.join("shaderpacks"),
        ContentType::ResourcePack => game_dir.join("resourcepacks"),
        ContentType::Datapack => game_dir.join("datapacks"),
        ContentType::World => game_dir.join("saves"),
    }
}

/// Install content to an instance
///
/// # Arguments
/// * `source` - How the content is being installed (UserAdded, ModpackOriginal, etc.)
///   If None, defaults to UserAdded for regular installs, UserDependency for dependencies
/// * `parent_filename` - If this is a dependency, the filename of the content it's a dependency of
/// * `cancel_token` - Optional cancellation token for queue-based installs
/// * `queue_id` - Optional queue ID for queue-aware progress events
pub async fn install_content(
    state: &AppState,
    instance_id: &str,
    platform: ContentPlatform,
    content_id: &str,
    content_name: &str,
    content_slug: &str,
    content_type: ContentType,
    version: &ContentVersion,
    is_dependency: bool,
    parent_filename: Option<&str>,
    source: Option<ContentSource>,
    app_handle: Option<&AppHandle>,
    cancel_token: Option<&CancellationToken>,
    queue_id: Option<&str>,
) -> Result<InstalledContent, AppError> {
    // Check for cancellation at start
    if let Some(token) = cancel_token {
        if token.is_cancelled() {
            return Err(AppError::Cancelled);
        }
    }

    let settings = state.get_settings();
    let instances_base = settings.instances_path.clone();
    let pool_enabled = settings.resource_pool.enabled;
    let link_strategy = settings.resource_pool.link_strategy;

    let game_dir = get_instance_game_dir_with_base(&instances_base, instance_id);
    let content_dir = get_content_dir(&game_dir, &content_type);

    // Ensure content directory exists
    fs::create_dir_all(&content_dir)?;

    // Get the primary file to download
    let file = version
        .files
        .iter()
        .find(|f| f.primary)
        .or_else(|| version.files.first())
        .ok_or_else(|| AppError::ContentNotFound("No files in version".to_string()))?;

    // Download the file into memory
    let download_result = download_content_bytes(
        &state.http_client,
        &file.url,
        &file.filename,
        content_id,
        file.size,
        file.hash.as_deref(),
        file.hash_algorithm.as_deref(),
        app_handle,
        cancel_token,
        queue_id,
    )
    .await?;

    // Handle worlds specially - they need to be extracted
    // Handle datapacks specially - they need to go into each world's datapacks folder
    let (is_pooled, installed_filename) = if content_type == ContentType::World {
        // Extract world ZIP to saves folder
        let world_folder_name = extract_world_zip(&download_result.bytes, &content_dir)?;
        (false, world_folder_name)
    } else if content_type == ContentType::Datapack {
        // Datapacks go into each world's datapacks folder, not a global folder
        let saves_dir = game_dir.join("saves");

        if saves_dir.exists() {
            for entry in fs::read_dir(&saves_dir)? {
                let entry = entry?;
                let path = entry.path();
                // Check if it's a world folder (has level.dat)
                if path.is_dir() && path.join("level.dat").exists() {
                    let world_datapacks_dir = path.join("datapacks");
                    fs::create_dir_all(&world_datapacks_dir)?;
                    let dest_path = world_datapacks_dir.join(&file.filename);
                    fs::write(&dest_path, &download_result.bytes)?;
                }
            }
        }

        // Also store in global datapacks folder for new worlds (as a staging area)
        fs::create_dir_all(&content_dir)?;
        let global_path = content_dir.join(&file.filename);
        fs::write(&global_path, &download_result.bytes)?;

        (false, file.filename.clone())
    } else if pool_enabled {
        // Add to pool first (for non-world content)
        let _pool_path = resource_pool_service::add_resource_from_bytes(
            state,
            &download_result.bytes,
            &download_result.sha512_hash,
            content_type,
            &file.filename,
        )?;

        // Link from pool to instance
        let link_result = resource_pool_service::link_to_instance(
            state,
            &download_result.sha512_hash,
            &content_type,
            instance_id,
            &file.filename,
            link_strategy,
        )?;

        (link_result.success, file.filename.clone())
    } else {
        // Write directly to instance (old behavior)
        let file_path = content_dir.join(&file.filename);
        let mut out_file = File::create(&file_path)?;
        out_file.write_all(&download_result.bytes)?;
        (false, file.filename.clone())
    };

    // Emit progress event
    if let Some(handle) = app_handle {
        let _ = handle.emit(
            "content_installed",
            serde_json::json!({
                "instanceId": instance_id,
                "contentType": content_type,
                "name": content_name,
                "filename": installed_filename,
                "isPooled": is_pooled,
            }),
        );
    }

    // Determine source: use provided source, or default based on is_dependency
    let content_source = source.unwrap_or(if is_dependency {
        ContentSource::UserDependency
    } else {
        ContentSource::UserAdded
    });

    // Extract dependency IDs from version for reverse lookup
    // Format: "platform:id" (e.g., "modrinth:abc123" or "curseforge:12345")
    let dependency_ids: Vec<String> = version
        .dependencies
        .iter()
        .filter(|d| d.dependency_type == DependencyType::Required)
        .map(|d| format!("{}:{}", platform, d.id))
        .collect();

    // Create installed content entry
    let installed = InstalledContent {
        name: content_name.to_string(),
        slug: content_slug.to_string(),
        modrinth_id: if platform == ContentPlatform::Modrinth {
            Some(content_id.to_string())
        } else {
            None
        },
        curseforge_id: if platform == ContentPlatform::CurseForge {
            content_id.parse::<u32>().ok()
        } else {
            None
        },
        installed_from: platform,
        version: version.version_number.clone(),
        version_id: version.id.clone(),
        filename: installed_filename,
        content_type,
        installed_at: Utc::now().timestamp(),
        is_dependency,
        dependency_of: parent_filename
            .map(|p| vec![p.to_string()])
            .unwrap_or_default(),
        dependency_ids,
        source: content_source,
        sha512_hash: Some(download_result.sha512_hash),
        murmur2_fingerprint: Some(download_result.murmur2_fingerprint),
        is_pooled,
    };

    // Save to manifest for persistence
    manifest_service::add_content(state, instance_id, installed.clone())?;

    // After saving, check if any existing content depends on this newly installed content
    // and update this content's dependency_of accordingly
    manifest_service::update_reverse_dependencies(state, instance_id, &installed)?;

    Ok(installed)
}

/// Download content and return bytes with computed hashes
///
/// Downloads the file into memory, computes hashes, and returns the bytes.
/// Does NOT write to disk - caller handles file creation.
///
/// # Arguments
/// * `cancel_token` - Optional cancellation token for queue-based installs
/// * `queue_id` - Optional queue ID; when provided, emits queue-specific progress events
async fn download_content_bytes(
    client: &Client,
    url: &str,
    filename: &str,
    content_id: &str,
    expected_size: u64,
    expected_hash: Option<&str>,
    hash_algorithm: Option<&str>,
    app_handle: Option<&AppHandle>,
    cancel_token: Option<&CancellationToken>,
    queue_id: Option<&str>,
) -> Result<DownloadResult, AppError> {
    // Check for cancellation
    if let Some(token) = cancel_token {
        if token.is_cancelled() {
            return Err(AppError::Cancelled);
        }
    }

    // Validate URL is not empty
    if url.is_empty() {
        return Err(AppError::DownloadError(
            "Cannot download content: URL is empty".to_string(),
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

    // Get content length from response or fall back to expected size
    let total_bytes = response.content_length().unwrap_or(expected_size);

    // Stream the download
    let mut stream = response.bytes_stream();
    let mut downloaded_bytes: u64 = 0;
    let mut all_bytes = Vec::with_capacity(total_bytes as usize);
    let mut last_emit_bytes: u64 = 0;
    const EMIT_THRESHOLD: u64 = 65536; // Emit every 64KB

    while let Some(chunk_result) = stream.next().await {
        // Check for cancellation periodically during download
        if let Some(token) = cancel_token {
            if token.is_cancelled() {
                return Err(AppError::Cancelled);
            }
        }

        let chunk = chunk_result
            .map_err(|e| AppError::DownloadError(format!("Failed to read chunk: {}", e)))?;

        downloaded_bytes += chunk.len() as u64;
        all_bytes.extend_from_slice(&chunk);

        // Emit progress event periodically
        if let Some(handle) = app_handle {
            if downloaded_bytes - last_emit_bytes >= EMIT_THRESHOLD
                || downloaded_bytes == total_bytes
            {
                let progress_percent = if total_bytes > 0 {
                    ((downloaded_bytes as f64 / total_bytes as f64) * 100.0).min(100.0) as u8
                } else {
                    0
                };

                // Emit queue-specific progress if queue_id is provided
                if let Some(qid) = queue_id {
                    let progress = ContentDownloadProgressWithId {
                        queue_id: qid.to_string(),
                        content_id: content_id.to_string(),
                        filename: filename.to_string(),
                        downloaded_bytes,
                        total_bytes,
                        progress_percent,
                    };
                    let _ = handle.emit("content_download_progress", &progress);
                } else {
                    // Non-queue progress (legacy)
                    let progress = ContentDownloadProgress {
                        filename: filename.to_string(),
                        downloaded_bytes,
                        total_bytes,
                        progress_percent,
                    };
                    let _ = handle.emit("content_download_progress", &progress);
                }
                last_emit_bytes = downloaded_bytes;
            }
        }
    }

    // Compute SHA512 hash (always compute for manifest)
    let mut sha512_hasher = Sha512::new();
    Sha2Digest::update(&mut sha512_hasher, &all_bytes);
    let sha512_hash = format!("{:x}", Sha2Digest::finalize(sha512_hasher));

    // Compute Murmur2 fingerprint for CurseForge compatibility
    let murmur2_fingerprint = murmur2_bytes(&all_bytes);

    // Verify hash if provided
    if let Some(expected) = expected_hash {
        if !expected.is_empty() {
            let matches = match hash_algorithm.unwrap_or("sha1") {
                "sha512" => sha512_hash == expected.to_lowercase(),
                _ => {
                    // Compute SHA1 for verification
                    let mut hasher = Sha1::new();
                    Sha1Digest::update(&mut hasher, &all_bytes);
                    let sha1_hash = format!("{:x}", Sha1Digest::finalize(hasher));
                    sha1_hash == expected.to_lowercase()
                }
            };

            if !matches {
                return Err(AppError::HashMismatch(filename.to_string()));
            }
        }
    }

    Ok(DownloadResult {
        sha512_hash,
        murmur2_fingerprint,
        bytes: all_bytes,
    })
}

/// Resolve dependencies for a content version
/// Returns a list of required dependencies with their compatible versions
/// Note: already_installed is determined by checking if the file exists in the mods folder
pub async fn resolve_dependencies(
    state: &AppState,
    instance_id: &str,
    platform: &ContentPlatform,
    version: &ContentVersion,
    mc_version: &str,
    loader: Option<&LoaderType>,
) -> Result<Vec<ResolvedDependency>, AppError> {
    let instances_base = state.settings.read().instances_path.clone();
    let game_dir = get_instance_game_dir_with_base(&instances_base, instance_id);
    let mods_dir = game_dir.join("mods");

    let mut resolved = Vec::new();

    // Only resolve required dependencies
    let required_deps: Vec<_> = version
        .dependencies
        .iter()
        .filter(|d| d.dependency_type == DependencyType::Required)
        .collect();

    for dep in required_deps {
        // Handle version-based dependency IDs (format: "version:VERSION_ID")
        // These occur when Modrinth returns a dependency with version_id but no project_id
        let (content_id, specific_version_id) = if dep.id.starts_with("version:") {
            let version_id = dep.id.strip_prefix("version:").unwrap();
            // Fetch the version to get the project_id
            if let Ok(ver) = modrinth_service::get_version(&state.http_client, version_id).await {
                (ver.project_id.clone(), Some(ver))
            } else {
                continue; // Skip if we can't resolve the version
            }
        } else {
            (dep.id.clone(), None)
        };

        // Fetch content info
        let content_result = match platform {
            ContentPlatform::Modrinth => {
                modrinth_service::get_content(&state.http_client, &content_id).await
            }
            ContentPlatform::CurseForge => {
                let api_key = state.get_settings().curseforge_api_key.ok_or_else(|| {
                    AppError::ApiError("CurseForge API key not configured".to_string())
                })?;
                curseforge_service::get_content(&state.http_client, &api_key, &content_id).await
            }
        };

        if let Ok(content) = content_result {
            // If we already have a specific version from version-based ID, use that
            if let Some(dep_version) = specific_version_id {
                // Check if the file already exists
                let already_installed = dep_version
                    .files
                    .iter()
                    .find(|f| f.primary)
                    .or_else(|| dep_version.files.first())
                    .map(|f| mods_dir.join(&f.filename).exists())
                    .unwrap_or(false);

                resolved.push(ResolvedDependency {
                    content,
                    version: dep_version,
                    already_installed,
                });
                continue;
            }

            // Find compatible version
            let versions_result = match platform {
                ContentPlatform::Modrinth => {
                    modrinth_service::get_content_versions(
                        &state.http_client,
                        &content_id,
                        Some(mc_version),
                        loader,
                    )
                    .await
                }
                ContentPlatform::CurseForge => {
                    let api_key = state.get_settings().curseforge_api_key.ok_or_else(|| {
                        AppError::ApiError("CurseForge API key not configured".to_string())
                    })?;
                    curseforge_service::get_content_versions(
                        &state.http_client,
                        &api_key,
                        &content_id,
                        Some(mc_version),
                        loader,
                    )
                    .await
                }
            };

            if let Ok(versions) = versions_result {
                // Use the first compatible version (they're usually sorted by date)
                if let Some(dep_version) = versions.into_iter().next() {
                    // Check if the file already exists
                    let already_installed = dep_version
                        .files
                        .iter()
                        .find(|f| f.primary)
                        .or_else(|| dep_version.files.first())
                        .map(|f| mods_dir.join(&f.filename).exists())
                        .unwrap_or(false);

                    resolved.push(ResolvedDependency {
                        content,
                        version: dep_version,
                        already_installed,
                    });
                }
            }
        }
    }

    Ok(resolved)
}

/// Install content with its dependencies (blocking/synchronous version)
///
/// NOTE: For user-initiated installs, prefer using the queue system via
/// `content_queue_service::queue_content_with_deps()` which is non-blocking.
/// This function is kept for modpack installation which may need synchronous behavior.
///
/// # Arguments
/// * `source` - How the content is being installed (UserAdded, ModpackOriginal, etc.)
///   If None, defaults to UserAdded for main content, UserDependency for dependencies
pub async fn install_content_with_dependencies(
    state: &AppState,
    instance_id: &str,
    platform: ContentPlatform,
    content: &Content,
    version: &ContentVersion,
    mc_version: &str,
    loader: Option<&LoaderType>,
    source: Option<ContentSource>,
    app_handle: Option<&AppHandle>,
) -> Result<Vec<InstalledContent>, AppError> {
    let mut installed = Vec::new();

    // Get the main content's filename to track as parent for dependencies
    let main_filename = version
        .files
        .iter()
        .find(|f| f.primary)
        .or_else(|| version.files.first())
        .map(|f| f.filename.clone());

    // Determine dependency source based on main content source
    let dep_source = match &source {
        Some(ContentSource::ModpackOriginal) => Some(ContentSource::ModpackOriginal),
        _ => Some(ContentSource::UserDependency),
    };

    // First, resolve and install dependencies
    let dependencies =
        resolve_dependencies(state, instance_id, &platform, version, mc_version, loader).await?;

    for dep in dependencies {
        if !dep.already_installed {
            let dep_installed = install_content(
                state,
                instance_id,
                platform,
                &dep.content.id,
                &dep.content.name,
                &dep.content.slug,
                dep.content.content_type,
                &dep.version,
                true,                     // is_dependency = true
                main_filename.as_deref(), // parent filename
                dep_source.clone(),
                app_handle,
                None, // cancel_token: not used in blocking install
                None, // queue_id: not used in blocking install
            )
            .await?;
            installed.push(dep_installed);
        } else if let Some(parent_fn) = &main_filename {
            // Dependency is already installed - update its dependency_of to include this content
            if let Some(dep_filename) = get_primary_filename(&dep.version) {
                let _ = manifest_service::add_dependent(
                    state,
                    instance_id,
                    &dep_filename,
                    &dep.content.content_type,
                    parent_fn,
                );
            }
        }
    }

    // Install the main content
    let main_installed = install_content(
        state,
        instance_id,
        platform,
        &content.id,
        &content.name,
        &content.slug,
        content.content_type,
        version,
        false, // is_dependency = false
        None,  // no parent for main content
        source,
        app_handle,
        None, // cancel_token: not used in blocking install
        None, // queue_id: not used in blocking install
    )
    .await?;
    installed.push(main_installed);

    Ok(installed)
}

/// Sync all installed datapacks to all worlds in an instance
/// This ensures datapacks are present in worlds created after the datapack was installed
pub fn sync_datapacks_to_worlds(state: &AppState, instance_id: &str) -> Result<u32, AppError> {
    let instances_base = state.settings.read().instances_path.clone();
    let game_dir = get_instance_game_dir_with_base(&instances_base, instance_id);
    let saves_dir = game_dir.join("saves");
    let global_datapacks_dir = game_dir.join("datapacks");

    // Get list of datapacks from global folder
    let mut datapack_files: Vec<(String, Vec<u8>)> = Vec::new();

    if global_datapacks_dir.exists() {
        for entry in fs::read_dir(&global_datapacks_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|e| e == "zip") {
                if let Some(filename) = path.file_name() {
                    let filename = filename.to_string_lossy().to_string();
                    // Skip cache files
                    if filename.starts_with('.') {
                        continue;
                    }
                    if let Ok(bytes) = fs::read(&path) {
                        datapack_files.push((filename, bytes));
                    }
                }
            }
        }
    }

    if datapack_files.is_empty() {
        return Ok(0);
    }

    let mut synced_count = 0u32;

    // Copy datapacks to each world
    if saves_dir.exists() {
        for entry in fs::read_dir(&saves_dir)? {
            let entry = entry?;
            let path = entry.path();
            // Check if it's a world folder (has level.dat)
            if path.is_dir() && path.join("level.dat").exists() {
                let world_datapacks_dir = path.join("datapacks");
                fs::create_dir_all(&world_datapacks_dir)?;

                for (filename, bytes) in &datapack_files {
                    let dest_path = world_datapacks_dir.join(filename);
                    if !dest_path.exists() {
                        fs::write(&dest_path, bytes)?;
                        synced_count += 1;
                    }
                }
            }
        }
    }

    Ok(synced_count)
}
