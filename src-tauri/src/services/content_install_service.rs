use crate::error::AppError;
use crate::models::{
    Content, ContentDownloadProgress, ContentDownloadProgressWithId, ContentPlatform,
    ContentSource, ContentType, ContentVersion, DependencyType, InstalledContent, LoaderType,
    ResolvedDependency,
};
use crate::services::{curseforge_service, manifest_service, modrinth_service};
use crate::state::AppState;
use crate::utils::hash::murmur2_bytes;
use crate::utils::paths::get_instance_game_dir_with_base;
use chrono::Utc;
use futures::StreamExt;
use reqwest::Client;
use sha1::{Digest as Sha1Digest, Sha1};
use sha2::{Digest as Sha2Digest, Sha512};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;

/// Result of downloading a file including computed hashes
struct DownloadResult {
    sha512_hash: String,
    murmur2_fingerprint: u32,
}

/// Get the content install directory based on content type
fn get_content_dir(game_dir: &PathBuf, content_type: &ContentType) -> PathBuf {
    match content_type {
        ContentType::Mod => game_dir.join("mods"),
        ContentType::Shader => game_dir.join("shaderpacks"),
        ContentType::ResourcePack => game_dir.join("resourcepacks"),
    }
}

/// Install content to an instance
///
/// # Arguments
/// * `source` - How the content is being installed (UserAdded, ModpackOriginal, etc.)
///   If None, defaults to UserAdded for regular installs, UserDependency for dependencies
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

    let instances_base = state.settings.read().instances_path.clone();
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

    // Download the file with progress tracking
    let file_path = content_dir.join(&file.filename);
    let download_result = download_file_with_progress(
        &state.http_client,
        &file.url,
        &file_path,
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

    // Emit progress event
    if let Some(handle) = app_handle {
        let _ = handle.emit(
            "content_installed",
            serde_json::json!({
                "instanceId": instance_id,
                "contentType": content_type,
                "name": content_name,
                "filename": file.filename,
            }),
        );
    }

    // Determine source: use provided source, or default based on is_dependency
    let content_source = source.unwrap_or(if is_dependency {
        ContentSource::UserDependency
    } else {
        ContentSource::UserAdded
    });

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
        filename: file.filename.clone(),
        content_type,
        installed_at: Utc::now().timestamp(),
        is_dependency,
        source: content_source,
        sha512_hash: Some(download_result.sha512_hash),
        murmur2_fingerprint: Some(download_result.murmur2_fingerprint),
    };

    // Save to manifest for persistence
    manifest_service::add_content(state, instance_id, installed.clone())?;

    Ok(installed)
}

/// Download a file with streaming progress updates and optional hash verification
/// Returns the computed SHA512 hash and Murmur2 fingerprint for manifest tracking
///
/// # Arguments
/// * `cancel_token` - Optional cancellation token for queue-based installs
/// * `queue_id` - Optional queue ID; when provided, emits queue-specific progress events
async fn download_file_with_progress(
    client: &Client,
    url: &str,
    path: &PathBuf,
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

    // Create parent directories
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
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
                return Err(AppError::HashMismatch(path.display().to_string()));
            }
        }
    }

    // Write to file
    let mut file = File::create(path)?;
    file.write_all(&all_bytes)?;

    Ok(DownloadResult {
        sha512_hash,
        murmur2_fingerprint,
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
                true, // is_dependency = true
                dep_source.clone(),
                app_handle,
                None, // cancel_token: not used in blocking install
                None, // queue_id: not used in blocking install
            )
            .await?;
            installed.push(dep_installed);
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
        source,
        app_handle,
        None, // cancel_token: not used in blocking install
        None, // queue_id: not used in blocking install
    )
    .await?;
    installed.push(main_installed);

    Ok(installed)
}
