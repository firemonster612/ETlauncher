use crate::error::AppError;
use crate::models::content::{
    ContentDownloadProgressWithId, ContentPlatform, ContentSource, ContentType, InstalledContent,
    InstalledContentManifest, QueueInstallRequest, QueueItemStatus, QueueStatusEvent,
    QueuedContentInstall, MANIFEST_VERSION,
};
use crate::models::AppSettings;
use crate::services::{curseforge_service, modrinth_service};
use crate::state::AppState;
use crate::utils::paths::get_instance_dir_with_base;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

/// Maximum concurrent content downloads
const MAX_CONCURRENT_DOWNLOADS: usize = 2;

/// Shared state for queue processing tasks (cloneable for spawned tasks)
#[derive(Clone)]
struct QueueContext {
    http_client: reqwest::Client,
    settings: AppSettings,
    queue: Arc<Mutex<VecDeque<QueuedContentInstall>>>,
    active: Arc<Mutex<HashSet<String>>>,
    tokens: Arc<Mutex<HashMap<String, CancellationToken>>>,
}

/// Add content to the download queue
pub async fn queue_content_install(
    state: &AppState,
    app_handle: AppHandle,
    request: QueueInstallRequest,
) -> Result<(), AppError> {
    let queued_item = QueuedContentInstall {
        queue_id: request.queue_id.clone(),
        instance_id: request.instance_id,
        platform: request.platform,
        content_id: request.content_id.clone(),
        content_name: request.content_name,
        content_slug: request.content_slug,
        content_type: request.content_type,
        version_id: request.version_id,
        version_name: request.version_name,
        mc_version: request.mc_version,
        loader: request.loader,
    };

    // Add to queue
    {
        let mut queue = state.content_download_queue.lock().await;
        queue.push_back(queued_item);
    }

    // Emit pending status
    let _ = app_handle.emit(
        "content_queue_status",
        QueueStatusEvent {
            queue_id: request.queue_id.clone(),
            content_id: request.content_id,
            status: QueueItemStatus::Pending,
            error: None,
        },
    );

    // Create context for processing
    let ctx = QueueContext {
        http_client: state.http_client.clone(),
        settings: state.settings.read().clone(),
        queue: state.content_download_queue.clone(),
        active: state.active_content_downloads.clone(),
        tokens: state.content_download_tokens.clone(),
    };

    // Try to start processing
    start_queue_processing(ctx, app_handle).await;

    Ok(())
}

/// Try to process any pending items in the queue (called when a slot becomes available)
pub async fn try_process_queue(state: &AppState, app_handle: AppHandle) {
    let ctx = QueueContext {
        http_client: state.http_client.clone(),
        settings: state.settings.read().clone(),
        queue: state.content_download_queue.clone(),
        active: state.active_content_downloads.clone(),
        tokens: state.content_download_tokens.clone(),
    };
    start_queue_processing(ctx, app_handle).await;
}

/// Cancel a queued or in-progress download
pub async fn cancel_queue_item(
    state: &AppState,
    app_handle: AppHandle,
    queue_id: String,
) -> Result<(), AppError> {
    // First try to remove from pending queue
    {
        let mut queue = state.content_download_queue.lock().await;
        if let Some(pos) = queue.iter().position(|item| item.queue_id == queue_id) {
            let item = queue.remove(pos).unwrap();
            // Emit cancelled status
            let _ = app_handle.emit(
                "content_queue_status",
                QueueStatusEvent {
                    queue_id: queue_id.to_string(),
                    content_id: item.content_id,
                    status: QueueItemStatus::Failed,
                    error: Some("Cancelled".to_string()),
                },
            );
            return Ok(());
        }
    }

    // If not in queue, check if it's actively downloading
    {
        let tokens = state.content_download_tokens.lock().await;
        if let Some(token) = tokens.get(&queue_id) {
            token.cancel();
        }
    }

    Ok(())
}

/// Start processing queue items if we have capacity
async fn start_queue_processing(ctx: QueueContext, app_handle: AppHandle) {
    loop {
        // Check how many downloads are active
        let active_count = ctx.active.lock().await.len();
        if active_count >= MAX_CONCURRENT_DOWNLOADS {
            break;
        }

        // Get next item from queue
        let next_item = {
            let mut queue = ctx.queue.lock().await;
            queue.pop_front()
        };

        let Some(item) = next_item else {
            break;
        };

        // Mark as active and create cancellation token
        let token = CancellationToken::new();
        {
            let mut active = ctx.active.lock().await;
            active.insert(item.queue_id.clone());
        }
        {
            let mut tokens = ctx.tokens.lock().await;
            tokens.insert(item.queue_id.clone(), token.clone());
        }

        // Emit downloading status
        let _ = app_handle.emit(
            "content_queue_status",
            QueueStatusEvent {
                queue_id: item.queue_id.clone(),
                content_id: item.content_id.clone(),
                status: QueueItemStatus::Downloading,
                error: None,
            },
        );

        // Clone context and handle for the spawned task
        let ctx_for_task = ctx.clone();
        let handle_for_task = app_handle.clone();
        let item_clone = item.clone();
        let token_clone = token.clone();

        // Spawn download task
        tokio::spawn(async move {
            let result = process_download(
                &ctx_for_task,
                &handle_for_task,
                item_clone.clone(),
                token_clone,
            )
            .await;

            // Remove from active downloads
            {
                let mut active = ctx_for_task.active.lock().await;
                active.remove(&item_clone.queue_id);
            }
            {
                let mut tokens = ctx_for_task.tokens.lock().await;
                tokens.remove(&item_clone.queue_id);
            }

            // Emit final status
            let status_event = match result {
                Ok(_) => QueueStatusEvent {
                    queue_id: item_clone.queue_id.clone(),
                    content_id: item_clone.content_id.clone(),
                    status: QueueItemStatus::Completed,
                    error: None,
                },
                Err(e) => QueueStatusEvent {
                    queue_id: item_clone.queue_id.clone(),
                    content_id: item_clone.content_id.clone(),
                    status: QueueItemStatus::Failed,
                    error: Some(e.to_string()),
                },
            };
            let _ = handle_for_task.emit("content_queue_status", status_event);

            // Signal that we should try to process more items
            let _ = handle_for_task.emit("content_queue_slot_available", ());
        });
    }
}

/// Process a single download
async fn process_download(
    ctx: &QueueContext,
    app_handle: &AppHandle,
    item: QueuedContentInstall,
    cancel_token: CancellationToken,
) -> Result<(), AppError> {
    // Check for cancellation
    if cancel_token.is_cancelled() {
        return Err(AppError::ContentNotFound("Cancelled".to_string()));
    }

    // Fetch version info from the appropriate platform
    let version = match item.platform {
        ContentPlatform::Modrinth => {
            let versions = modrinth_service::get_content_versions(
                &ctx.http_client,
                &item.content_id,
                Some(&item.mc_version),
                item.loader.as_ref(),
            )
            .await?;

            versions
                .into_iter()
                .find(|v| v.id == item.version_id)
                .ok_or_else(|| AppError::ContentNotFound("Version not found".to_string()))?
        }
        ContentPlatform::CurseForge => {
            let api_key = ctx.settings.curseforge_api_key.as_ref().ok_or_else(|| {
                AppError::ApiError("CurseForge API key not configured".to_string())
            })?;

            let versions = curseforge_service::get_content_versions(
                &ctx.http_client,
                api_key,
                &item.content_id,
                Some(&item.mc_version),
                item.loader.as_ref(),
            )
            .await?;

            versions
                .into_iter()
                .find(|v| v.id == item.version_id)
                .ok_or_else(|| AppError::ContentNotFound("Version not found".to_string()))?
        }
    };

    // Check for cancellation
    if cancel_token.is_cancelled() {
        return Err(AppError::ContentNotFound("Cancelled".to_string()));
    }

    // Get the file to download
    let file = version
        .files
        .iter()
        .find(|f| f.primary)
        .or_else(|| version.files.first())
        .ok_or_else(|| AppError::ContentNotFound("No files in version".to_string()))?;

    // Get paths
    let game_dir = crate::utils::paths::get_instance_game_dir_with_base(
        &ctx.settings.instances_path,
        &item.instance_id,
    );
    let content_dir = match item.content_type {
        ContentType::Mod => game_dir.join("mods"),
        ContentType::Shader => game_dir.join("shaderpacks"),
        ContentType::ResourcePack => game_dir.join("resourcepacks"),
    };
    std::fs::create_dir_all(&content_dir)?;
    let file_path = content_dir.join(&file.filename);

    // Download with progress and get computed hashes
    let download_result = download_file_with_queue_progress(
        &ctx.http_client,
        &file.url,
        &file_path,
        &file.filename,
        file.size,
        file.hash.as_deref(),
        file.hash_algorithm.as_deref(),
        app_handle,
        &item.queue_id,
        &item.content_id,
        &cancel_token,
    )
    .await?;

    // Save to manifest with computed hashes
    let installed = InstalledContent {
        name: item.content_name.clone(),
        slug: item.content_slug.clone(),
        modrinth_id: if item.platform == ContentPlatform::Modrinth {
            Some(item.content_id.clone())
        } else {
            None
        },
        curseforge_id: if item.platform == ContentPlatform::CurseForge {
            item.content_id.parse::<u32>().ok()
        } else {
            None
        },
        installed_from: item.platform.clone(),
        version: version.version_number.clone(),
        version_id: version.id.clone(),
        filename: file.filename.clone(),
        content_type: item.content_type.clone(),
        installed_at: chrono::Utc::now().timestamp(),
        is_dependency: false,
        source: ContentSource::UserAdded,
        sha512_hash: Some(download_result.sha512_hash),
        murmur2_fingerprint: Some(download_result.murmur2_fingerprint),
    };

    // Save to the correct manifest file (etlauncher_manifest.json in instance directory)
    let instance_dir = get_instance_dir_with_base(&ctx.settings.instances_path, &item.instance_id);
    let manifest_path = instance_dir.join("etlauncher_manifest.json");
    let mut manifest: InstalledContentManifest = if manifest_path.exists() {
        let content = std::fs::read_to_string(&manifest_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| InstalledContentManifest {
            manifest_version: MANIFEST_VERSION,
            mods: Vec::new(),
            shaders: Vec::new(),
            resource_packs: Vec::new(),
            last_synced_at: None,
        })
    } else {
        InstalledContentManifest {
            manifest_version: MANIFEST_VERSION,
            mods: Vec::new(),
            shaders: Vec::new(),
            resource_packs: Vec::new(),
            last_synced_at: None,
        }
    };

    // Remove existing entry with same filename (if updating) and add the new content
    match item.content_type {
        ContentType::Mod => {
            manifest.mods.retain(|c| c.filename != installed.filename);
            manifest.mods.push(installed);
        }
        ContentType::Shader => {
            manifest
                .shaders
                .retain(|c| c.filename != installed.filename);
            manifest.shaders.push(installed);
        }
        ContentType::ResourcePack => {
            manifest
                .resource_packs
                .retain(|c| c.filename != installed.filename);
            manifest.resource_packs.push(installed);
        }
    }

    // Save manifest
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    std::fs::write(&manifest_path, manifest_json)?;

    // Emit installed event
    let _ = app_handle.emit(
        "content_installed",
        serde_json::json!({
            "instanceId": item.instance_id,
            "contentType": item.content_type,
            "name": item.content_name,
            "filename": file.filename,
        }),
    );

    Ok(())
}

/// Result of downloading a file with computed hashes
struct QueueDownloadResult {
    sha512_hash: String,
    murmur2_fingerprint: u32,
}

/// Download a file with queue-aware progress, returning computed hashes
async fn download_file_with_queue_progress(
    client: &reqwest::Client,
    url: &str,
    path: &std::path::PathBuf,
    filename: &str,
    expected_size: u64,
    expected_hash: Option<&str>,
    hash_algorithm: Option<&str>,
    app_handle: &AppHandle,
    queue_id: &str,
    content_id: &str,
    cancel_token: &CancellationToken,
) -> Result<QueueDownloadResult, AppError> {
    use futures::StreamExt;
    use sha1::{Digest as Sha1Digest, Sha1};
    use sha2::{Digest as Sha2Digest, Sha512};
    use std::io::Write;

    // Create parent directories
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
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

    let total_bytes = response.content_length().unwrap_or(expected_size);
    let mut stream = response.bytes_stream();
    let mut downloaded_bytes: u64 = 0;
    let mut all_bytes = Vec::with_capacity(total_bytes as usize);
    let mut last_emit_bytes: u64 = 0;
    const EMIT_THRESHOLD: u64 = 65536;

    while let Some(chunk_result) = stream.next().await {
        // Check for cancellation
        if cancel_token.is_cancelled() {
            return Err(AppError::ContentNotFound("Cancelled".to_string()));
        }

        let chunk = chunk_result
            .map_err(|e| AppError::DownloadError(format!("Failed to read chunk: {}", e)))?;

        downloaded_bytes += chunk.len() as u64;
        all_bytes.extend_from_slice(&chunk);

        // Emit progress with queue ID
        if downloaded_bytes - last_emit_bytes >= EMIT_THRESHOLD || downloaded_bytes == total_bytes {
            let progress_percent = if total_bytes > 0 {
                ((downloaded_bytes as f64 / total_bytes as f64) * 100.0).min(100.0) as u8
            } else {
                0
            };

            let progress = ContentDownloadProgressWithId {
                queue_id: queue_id.to_string(),
                content_id: content_id.to_string(),
                filename: filename.to_string(),
                downloaded_bytes,
                total_bytes,
                progress_percent,
            };

            let _ = app_handle.emit("content_download_progress", &progress);
            last_emit_bytes = downloaded_bytes;
        }
    }

    // Verify hash if provided
    if let Some(expected) = expected_hash {
        if !expected.is_empty() {
            let matches = match hash_algorithm.unwrap_or("sha1") {
                "sha512" => {
                    let mut hasher = Sha512::new();
                    Sha2Digest::update(&mut hasher, &all_bytes);
                    let hash = format!("{:x}", Sha2Digest::finalize(hasher));
                    hash == expected.to_lowercase()
                }
                _ => {
                    let mut hasher = Sha1::new();
                    Sha1Digest::update(&mut hasher, &all_bytes);
                    let hash = format!("{:x}", Sha1Digest::finalize(hasher));
                    hash == expected.to_lowercase()
                }
            };

            if !matches {
                return Err(AppError::HashMismatch(path.display().to_string()));
            }
        }
    }

    // Compute SHA512 hash for Modrinth compatibility
    let mut sha512_hasher = Sha512::new();
    Sha2Digest::update(&mut sha512_hasher, &all_bytes);
    let sha512_hash = format!("{:x}", Sha2Digest::finalize(sha512_hasher));

    // Compute Murmur2 fingerprint for CurseForge compatibility
    // Strip whitespace (tab, lf, cr, space) before hashing as required by CurseForge
    let filtered: Vec<u8> = all_bytes
        .iter()
        .copied()
        .filter(|&b| b != 9 && b != 10 && b != 13 && b != 32)
        .collect();
    let murmur2_fingerprint = murmur2::murmur2(&filtered, 1);

    // Write to file
    let mut file = std::fs::File::create(path)?;
    file.write_all(&all_bytes)?;

    Ok(QueueDownloadResult {
        sha512_hash,
        murmur2_fingerprint,
    })
}
