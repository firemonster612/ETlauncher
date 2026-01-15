use crate::error::AppError;
use crate::models::content::{
    Content, ContentPlatform, ContentSource, ContentType, ContentVersion, InstalledContentManifest,
    QueueInstallRequest, QueueItemStatus, QueueStatusEvent, QueuedContentInstall,
};
use crate::models::instance::LoaderType;
use crate::models::AppSettings;
use crate::services::{content_install_service, curseforge_service, modrinth_service};
use crate::state::AppState;
use crate::utils::paths::get_instance_dir_with_base;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
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
        content_name: request.content_name.clone(),
        content_slug: request.content_slug,
        content_type: request.content_type.clone(),
        version_id: request.version_id,
        version_name: request.version_name,
        mc_version: request.mc_version,
        loader: request.loader,
        is_dependency: request.is_dependency,
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
            content_name: request.content_name,
            content_type: request.content_type.clone(),
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

/// Check if content is already installed by checking manifest
fn is_content_installed_sync(
    instances_base: &str,
    instance_id: &str,
    content_id: &str,
    platform: &ContentPlatform,
    content_type: &ContentType,
) -> bool {
    let instance_dir = get_instance_dir_with_base(instances_base, instance_id);
    let manifest_path = instance_dir.join("etlauncher_manifest.json");

    if !manifest_path.exists() {
        return false;
    }

    let Ok(content) = std::fs::read_to_string(&manifest_path) else {
        return false;
    };

    let Ok(manifest) = serde_json::from_str::<InstalledContentManifest>(&content) else {
        return false;
    };

    let items = match content_type {
        ContentType::Mod => &manifest.mods,
        ContentType::Shader => &manifest.shaders,
        ContentType::ResourcePack => &manifest.resource_packs,
    };

    items.iter().any(|item| match platform {
        ContentPlatform::Modrinth => item.modrinth_id.as_deref() == Some(content_id),
        ContentPlatform::CurseForge => content_id
            .parse::<u32>()
            .ok()
            .map(|id| item.curseforge_id == Some(id))
            .unwrap_or(false),
    })
}

/// Queue a single item (internal helper, doesn't resolve dependencies)
/// Returns Some(queue_id) if the item was queued, None if it was already in the queue.
/// This function atomically checks and inserts to prevent race conditions.
async fn queue_single_item(
    state: &AppState,
    app_handle: &AppHandle,
    instance_id: &str,
    platform: ContentPlatform,
    content_id: &str,
    content_name: &str,
    content_slug: &str,
    content_type: ContentType,
    version_id: &str,
    version_name: &str,
    mc_version: &str,
    loader: Option<LoaderType>,
    is_dependency: bool,
) -> Result<Option<String>, AppError> {
    let queue_id = uuid::Uuid::new_v4().to_string();

    let queued_item = QueuedContentInstall {
        queue_id: queue_id.clone(),
        instance_id: instance_id.to_string(),
        platform: platform.clone(),
        content_id: content_id.to_string(),
        content_name: content_name.to_string(),
        content_slug: content_slug.to_string(),
        content_type: content_type.clone(),
        version_id: version_id.to_string(),
        version_name: version_name.to_string(),
        mc_version: mc_version.to_string(),
        loader,
        is_dependency,
    };

    // Atomically check if already queued and add to queue
    {
        let mut queue = state.content_download_queue.lock().await;

        // Check if content is already in the queue
        if queue.iter().any(|item| item.content_id == content_id) {
            return Ok(None);
        }

        queue.push_back(queued_item);
    }

    // Emit pending status
    let _ = app_handle.emit(
        "content_queue_status",
        QueueStatusEvent {
            queue_id: queue_id.clone(),
            content_id: content_id.to_string(),
            content_name: content_name.to_string(),
            content_type: content_type.clone(),
            status: QueueItemStatus::Pending,
            error: None,
        },
    );

    Ok(Some(queue_id))
}

/// Resolve dependencies and queue all items (deps first, then main content)
/// Returns list of queue IDs for all queued items
///
/// This is the central function for installing content with dependencies.
/// It handles:
/// - Circular dependency detection via the `visited` set
/// - Skipping already-installed content
/// - Atomically checking and skipping already-queued content (via queue_single_item)
/// - Queueing dependencies before the main content
pub async fn resolve_and_queue_with_deps(
    state: &AppState,
    app_handle: &AppHandle,
    instance_id: &str,
    platform: ContentPlatform,
    content: &Content,
    version: &ContentVersion,
    mc_version: &str,
    loader: Option<&LoaderType>,
    is_dependency: bool,
    visited: &mut HashSet<String>,
) -> Result<Vec<String>, AppError> {
    let mut queue_ids = Vec::new();
    let content_id = &content.id;

    // 1. Check if already visited (circular dependency prevention)
    if visited.contains(content_id) {
        return Ok(queue_ids);
    }
    visited.insert(content_id.to_string());

    // 2. Check if already installed
    let instances_base = state.settings.read().instances_path.clone();
    if is_content_installed_sync(
        &instances_base,
        instance_id,
        content_id,
        &platform,
        &content.content_type,
    ) {
        return Ok(queue_ids);
    }

    // 3. Resolve dependencies for all content types that might have them
    // (mods, resource packs, shaders can all have dependencies)
    if !version.dependencies.is_empty() {
        let deps = content_install_service::resolve_dependencies(
            state,
            instance_id,
            &platform,
            version,
            mc_version,
            loader,
        )
        .await?;

        // Recursively queue each uninstalled dependency FIRST
        for dep in &deps {
            if !dep.already_installed {
                let dep_queue_ids = Box::pin(resolve_and_queue_with_deps(
                    state,
                    app_handle,
                    instance_id,
                    platform.clone(),
                    &dep.content,
                    &dep.version,
                    mc_version,
                    loader,
                    true, // is_dependency = true
                    visited,
                ))
                .await?;
                queue_ids.extend(dep_queue_ids);
            }
        }
    }

    // 4. Queue the main content (after dependencies)
    // queue_single_item atomically checks if already queued and returns None if so
    if let Some(queue_id) = queue_single_item(
        state,
        app_handle,
        instance_id,
        platform,
        content_id,
        &content.name,
        &content.slug,
        content.content_type.clone(),
        &version.id,
        &version.version_number,
        mc_version,
        loader.cloned(),
        is_dependency,
    )
    .await?
    {
        queue_ids.push(queue_id);
    }

    Ok(queue_ids)
}

/// Queue content installation with automatic dependency resolution
/// This is the main entry point for queueing installs - it resolves deps and queues everything
pub async fn queue_content_with_deps(
    state: &AppState,
    app_handle: AppHandle,
    instance_id: &str,
    platform: ContentPlatform,
    content: &Content,
    version: &ContentVersion,
    mc_version: &str,
    loader: Option<&LoaderType>,
) -> Result<Vec<String>, AppError> {
    let mut visited = HashSet::new();

    let queue_ids = resolve_and_queue_with_deps(
        state,
        &app_handle,
        instance_id,
        platform,
        content,
        version,
        mc_version,
        loader,
        false, // is_dependency = false (main content)
        &mut visited,
    )
    .await?;

    // Start queue processing after all items are queued
    let ctx = QueueContext {
        http_client: state.http_client.clone(),
        settings: state.settings.read().clone(),
        queue: state.content_download_queue.clone(),
        active: state.active_content_downloads.clone(),
        tokens: state.content_download_tokens.clone(),
    };
    start_queue_processing(ctx, app_handle).await;

    Ok(queue_ids)
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
                    content_name: item.content_name,
                    content_type: item.content_type,
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
                content_name: item.content_name.clone(),
                content_type: item.content_type.clone(),
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
                    content_name: item_clone.content_name.clone(),
                    content_type: item_clone.content_type.clone(),
                    status: QueueItemStatus::Completed,
                    error: None,
                },
                Err(e) => QueueStatusEvent {
                    queue_id: item_clone.queue_id.clone(),
                    content_id: item_clone.content_id.clone(),
                    content_name: item_clone.content_name.clone(),
                    content_type: item_clone.content_type.clone(),
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

/// Process a single download using the unified install_content function
async fn process_download(
    ctx: &QueueContext,
    app_handle: &AppHandle,
    item: QueuedContentInstall,
    cancel_token: CancellationToken,
) -> Result<(), AppError> {
    // Check for cancellation
    if cancel_token.is_cancelled() {
        return Err(AppError::Cancelled);
    }

    // Fetch version info directly by ID from the appropriate platform
    // This avoids issues where filtered queries might not return the expected version
    let version = match item.platform {
        ContentPlatform::Modrinth => {
            modrinth_service::get_version(&ctx.http_client, &item.version_id).await?
        }
        ContentPlatform::CurseForge => {
            let api_key = ctx.settings.curseforge_api_key.as_ref().ok_or_else(|| {
                AppError::ApiError("CurseForge API key not configured".to_string())
            })?;

            curseforge_service::get_version(
                &ctx.http_client,
                api_key,
                &item.content_id,
                &item.version_id,
            )
            .await?
        }
    };

    // Check for cancellation
    if cancel_token.is_cancelled() {
        return Err(AppError::Cancelled);
    }

    // Get state from app_handle for install_content
    let state = app_handle.state::<AppState>();

    // Determine source based on is_dependency flag
    let source = if item.is_dependency {
        Some(ContentSource::UserDependency)
    } else {
        Some(ContentSource::UserAdded)
    };

    // Use the unified install_content function
    content_install_service::install_content(
        &state,
        &item.instance_id,
        item.platform,
        &item.content_id,
        &item.content_name,
        &item.content_slug,
        item.content_type.clone(),
        &version,
        item.is_dependency,
        source,
        Some(app_handle),
        Some(&cancel_token),
        Some(&item.queue_id),
    )
    .await?;

    Ok(())
}
