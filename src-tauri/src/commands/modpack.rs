use crate::app_info;
use crate::cache::hash_params;
use crate::error::CommandError;
use crate::models::instance::ModpackPlatform;
use crate::models::{
    Instance, Modpack, ModpackMod, ModpackSearchParams, ModpackSearchResult, ModpackSortBy,
    ModpackVersion,
};
use crate::services::{
    atlauncher_service, curseforge_service, ftb_service, modpack_install_service, modrinth_service,
    technic_service,
};
use crate::state::{AppState, QueuedModpackInstall};
use serde::Serialize;
use std::cmp::Reverse;
use std::collections::VecDeque;
use tauri::{AppHandle, Emitter, Manager, State};

/// Info sent when a modpack install starts
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackInstallStarted {
    pub modpack_name: String,
}

/// Info returned when a modpack install is queued
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackInstallQueued {
    pub queue_id: String,
    pub modpack_name: String,
}

/// Search for modpacks across platforms
#[tauri::command]
pub async fn search_modpacks(
    state: State<'_, AppState>,
    params: ModpackSearchParams,
) -> Result<ModpackSearchResult, CommandError> {
    app_info!(
        "[modpack_cmd] search_modpacks: platform={:?}, query={:?}",
        params.platform,
        params.query
    );

    // Check cache first
    let cache_key = hash_params(&params);
    if let Some(cached) = state.api_cache.modpack_search.get(&cache_key) {
        return Ok(cached);
    }

    // If no platform specified, search all platforms and aggregate results
    let result = if let Some(platform) = params.platform {
        match platform {
            ModpackPlatform::Modrinth => {
                modrinth_service::search_modpacks(&state.http_client, &params)
                    .await
                    .map_err(CommandError::from)?
            }
            ModpackPlatform::CurseForge => {
                let api_key =
                    state
                        .get_settings()
                        .curseforge_api_key
                        .ok_or_else(|| CommandError {
                            code: "API_KEY_REQUIRED".to_string(),
                            message: "CurseForge API key not configured. Add it to your settings."
                                .to_string(),
                        })?;
                curseforge_service::search_modpacks(&state.http_client, &api_key, &params)
                    .await
                    .map_err(CommandError::from)?
            }
            ModpackPlatform::FTB => ftb_service::search_modpacks(&state.http_client, &params)
                .await
                .map_err(CommandError::from)?,
            ModpackPlatform::Technic => {
                technic_service::search_modpacks(&state.http_client, &params)
                    .await
                    .map_err(CommandError::from)?
            }
            ModpackPlatform::ATLauncher => ModpackSearchResult {
                modpacks: vec![],
                total_count: 0,
                page: params.page.unwrap_or(0),
                page_size: params.page_size.unwrap_or(20),
            },
        }
    } else {
        search_all_platforms(&state, &params).await?
    };

    app_info!(
        "[modpack_cmd] search_modpacks: success, count={}",
        result.modpacks.len()
    );

    // Store in cache
    state
        .api_cache
        .modpack_search
        .insert(cache_key, result.clone());

    Ok(result)
}

/// Search all platforms and aggregate results
async fn search_all_platforms(
    state: &State<'_, AppState>,
    params: &ModpackSearchParams,
) -> Result<ModpackSearchResult, CommandError> {
    let page = params.page.unwrap_or(0);
    let page_size = params.page_size.unwrap_or(20);

    // Create futures for each platform search
    let modrinth_future = modrinth_service::search_modpacks(&state.http_client, params);
    let ftb_future = ftb_service::search_modpacks(&state.http_client, params);

    // Technic uses trending endpoint when no query provided
    let technic_future = technic_service::search_modpacks(&state.http_client, params);

    // CurseForge requires API key
    let curseforge_future = async {
        if let Some(api_key) = state.get_settings().curseforge_api_key {
            curseforge_service::search_modpacks(&state.http_client, &api_key, params).await
        } else {
            Ok(ModpackSearchResult {
                modpacks: vec![],
                total_count: 0,
                page,
                page_size,
            })
        }
    };

    // Run all searches in parallel
    let results = tokio::join!(
        modrinth_future,
        ftb_future,
        technic_future,
        curseforge_future
    );

    // Collect successful results, log errors but don't fail
    let mut modrinth_modpacks: Vec<Modpack> = Vec::new();
    let mut ftb_modpacks: Vec<Modpack> = Vec::new();
    let mut technic_modpacks: Vec<Modpack> = Vec::new();
    let mut curseforge_modpacks: Vec<Modpack> = Vec::new();
    let mut total_count: u64 = 0;

    if let Ok(result) = results.0 {
        app_info!("[modpack_cmd] Modrinth: {} packs", result.modpacks.len());
        total_count += result.total_count;
        modrinth_modpacks = result.modpacks;
    } else if let Err(e) = &results.0 {
        app_info!("[modpack_cmd] Modrinth error: {:?}", e);
    }

    if let Ok(result) = results.1 {
        app_info!("[modpack_cmd] FTB: {} packs", result.modpacks.len());
        total_count += result.total_count;
        ftb_modpacks = result.modpacks;
    } else if let Err(e) = &results.1 {
        app_info!("[modpack_cmd] FTB error: {:?}", e);
    }

    if let Ok(result) = results.2 {
        app_info!("[modpack_cmd] Technic: {} packs", result.modpacks.len());
        total_count += result.total_count;
        technic_modpacks = result.modpacks;
    } else if let Err(e) = &results.2 {
        app_info!("[modpack_cmd] Technic error: {:?}", e);
    }

    if let Ok(result) = results.3 {
        app_info!("[modpack_cmd] CurseForge: {} packs", result.modpacks.len());
        total_count += result.total_count;
        curseforge_modpacks = result.modpacks;
    } else if let Err(e) = &results.3 {
        app_info!("[modpack_cmd] CurseForge error: {:?}", e);
    }

    let sort_by = params.sort_by.unwrap_or_default();
    let all_modpacks: Vec<Modpack> = match sort_by {
        ModpackSortBy::Relevance => {
            // Best-effort "relevance" across platforms: preserve each platform's native ordering
            // and interleave results so "All" doesn't look grouped by platform.
            let mut modrinth = VecDeque::from(modrinth_modpacks);
            let mut ftb = VecDeque::from(ftb_modpacks);
            let mut technic = VecDeque::from(technic_modpacks);
            let mut curseforge = VecDeque::from(curseforge_modpacks);

            let mut combined = Vec::new();
            while !(modrinth.is_empty()
                && ftb.is_empty()
                && technic.is_empty()
                && curseforge.is_empty())
            {
                if let Some(m) = modrinth.pop_front() {
                    combined.push(m);
                }
                if let Some(m) = curseforge.pop_front() {
                    combined.push(m);
                }
                if let Some(m) = ftb.pop_front() {
                    combined.push(m);
                }
                if let Some(m) = technic.pop_front() {
                    combined.push(m);
                }
            }
            combined
        }
        ModpackSortBy::RecentlyUpdated => {
            let mut combined = Vec::new();
            combined.extend(modrinth_modpacks);
            combined.extend(ftb_modpacks);
            combined.extend(technic_modpacks);
            combined.extend(curseforge_modpacks);
            combined.sort_by_key(|m| Reverse(m.updated_at.unwrap_or(0)));
            combined
        }
        ModpackSortBy::Name => {
            let mut combined = Vec::new();
            combined.extend(modrinth_modpacks);
            combined.extend(ftb_modpacks);
            combined.extend(technic_modpacks);
            combined.extend(curseforge_modpacks);
            combined.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            combined
        }
        ModpackSortBy::Downloads => {
            let mut combined = Vec::new();
            combined.extend(modrinth_modpacks);
            combined.extend(ftb_modpacks);
            combined.extend(technic_modpacks);
            combined.extend(curseforge_modpacks);
            combined.sort_by(|a, b| b.downloads.cmp(&a.downloads));
            combined
        }
        ModpackSortBy::Newest => {
            // Sort by created_at (newest first)
            let mut combined = Vec::new();
            combined.extend(modrinth_modpacks);
            combined.extend(ftb_modpacks);
            combined.extend(technic_modpacks);
            combined.extend(curseforge_modpacks);
            combined.sort_by_key(|m| Reverse(m.created_at.unwrap_or(0)));
            combined
        }
    };

    app_info!(
        "[modpack_cmd] All platforms total: {} packs",
        all_modpacks.len()
    );

    Ok(ModpackSearchResult {
        modpacks: all_modpacks,
        total_count,
        page,
        page_size,
    })
}

/// Get a modpack by ID
#[tauri::command]
pub async fn get_modpack(
    state: State<'_, AppState>,
    platform: ModpackPlatform,
    id: String,
) -> Result<Modpack, CommandError> {
    // Check cache first
    let cache_key = format!("{}:{}", platform, id);
    if let Some(cached) = state.api_cache.modpack_details.get(&cache_key) {
        return Ok(cached);
    }

    app_info!(
        "[modpack_cmd] get_modpack: platform={:?}, id={}",
        platform,
        id
    );
    let result = match platform {
        ModpackPlatform::Modrinth => modrinth_service::get_modpack(&state.http_client, &id)
            .await
            .map_err(CommandError::from),
        ModpackPlatform::CurseForge => {
            let api_key = state
                .get_settings()
                .curseforge_api_key
                .ok_or_else(|| CommandError {
                    code: "API_KEY_REQUIRED".to_string(),
                    message: "CurseForge API key not configured".to_string(),
                })?;
            curseforge_service::get_modpack(&state.http_client, &api_key, &id)
                .await
                .map_err(CommandError::from)
        }
        ModpackPlatform::FTB => ftb_service::get_modpack(&state.http_client, &id)
            .await
            .map_err(CommandError::from),
        ModpackPlatform::Technic => technic_service::get_modpack(&state.http_client, &id)
            .await
            .map_err(CommandError::from),
        ModpackPlatform::ATLauncher => atlauncher_service::get_modpack(&state.http_client, &id)
            .await
            .map_err(CommandError::from),
    }?;

    app_info!("[modpack_cmd] get_modpack: success, name={}", result.name);

    // Store in cache
    state
        .api_cache
        .modpack_details
        .insert(cache_key, result.clone());

    Ok(result)
}

/// Get versions for a modpack
#[tauri::command]
pub async fn get_modpack_versions(
    state: State<'_, AppState>,
    platform: ModpackPlatform,
    id: String,
) -> Result<Vec<ModpackVersion>, CommandError> {
    // Check cache first
    let cache_key = format!("{}:{}", platform, id);
    if let Some(cached) = state.api_cache.modpack_versions.get(&cache_key) {
        return Ok(cached);
    }

    app_info!(
        "[modpack_cmd] get_modpack_versions: platform={:?}, id={}",
        platform,
        id
    );
    let result = match platform {
        ModpackPlatform::Modrinth => {
            modrinth_service::get_modpack_versions(&state.http_client, &id)
                .await
                .map_err(CommandError::from)
        }
        ModpackPlatform::CurseForge => {
            let api_key = state
                .get_settings()
                .curseforge_api_key
                .ok_or_else(|| CommandError {
                    code: "API_KEY_REQUIRED".to_string(),
                    message: "CurseForge API key not configured".to_string(),
                })?;
            curseforge_service::get_modpack_versions(&state.http_client, &api_key, &id)
                .await
                .map_err(CommandError::from)
        }
        ModpackPlatform::FTB => ftb_service::get_modpack_versions(&state.http_client, &id)
            .await
            .map_err(CommandError::from),
        ModpackPlatform::Technic => technic_service::get_modpack_versions(&state.http_client, &id)
            .await
            .map_err(CommandError::from),
        ModpackPlatform::ATLauncher => {
            atlauncher_service::get_modpack_versions(&state.http_client, &id)
                .await
                .map_err(CommandError::from)
        }
    }?;

    app_info!(
        "[modpack_cmd] get_modpack_versions: success, count={}",
        result.len()
    );

    // Store in cache
    state
        .api_cache
        .modpack_versions
        .insert(cache_key, result.clone());

    Ok(result)
}

/// Get a mod list for a given modpack version (best-effort)
#[tauri::command]
pub async fn get_modpack_mods(
    state: State<'_, AppState>,
    platform: ModpackPlatform,
    modpack_id: String,
    version_id: String,
) -> Result<Vec<ModpackMod>, CommandError> {
    app_info!(
        "[modpack_cmd] get_modpack_mods: platform={:?}, modpack_id={}, version_id={}",
        platform,
        modpack_id,
        version_id
    );

    let result = match platform {
        ModpackPlatform::Modrinth => {
            modrinth_service::get_modpack_mods(&state.http_client, &version_id)
                .await
                .map_err(CommandError::from)
        }
        ModpackPlatform::CurseForge => {
            let api_key = state
                .get_settings()
                .curseforge_api_key
                .ok_or_else(|| CommandError {
                    code: "API_KEY_REQUIRED".to_string(),
                    message: "CurseForge API key not configured".to_string(),
                })?;
            curseforge_service::get_modpack_mods(
                &state.http_client,
                &api_key,
                &modpack_id,
                &version_id,
            )
            .await
            .map_err(CommandError::from)
        }
        ModpackPlatform::FTB => {
            ftb_service::get_modpack_mods(&state.http_client, &modpack_id, &version_id)
                .await
                .map_err(CommandError::from)
        }
        ModpackPlatform::Technic => {
            technic_service::get_modpack_mods(&state.http_client, &modpack_id, &version_id)
                .await
                .map_err(CommandError::from)
        }
        _ => Ok(vec![]),
    };

    match &result {
        Ok(mods) => app_info!(
            "[modpack_cmd] get_modpack_mods: success, count={}",
            mods.len()
        ),
        Err(e) => app_info!("[modpack_cmd] get_modpack_mods: error={:?}", e),
    }

    result
}

/// Install a modpack and create a new instance (non-blocking, queue-based)
#[tauri::command]
pub async fn install_modpack(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    platform: ModpackPlatform,
    modpack_id: String,
    version_id: String,
    instance_name: Option<String>,
) -> Result<ModpackInstallQueued, CommandError> {
    app_info!("[modpack_cmd] install_modpack: platform={:?}, modpack_id={}, version_id={}, instance_name={:?}",
        platform, modpack_id, version_id, instance_name);

    let modpack_name = instance_name
        .clone()
        .unwrap_or_else(|| format!("{:?} modpack", platform));
    let queue_id = uuid::Uuid::new_v4().to_string();

    // Register task immediately as Pending in the task registry
    state.task_registry.register(
        queue_id.clone(),
        crate::task_registry::TaskType::ModpackInstall,
        modpack_name.clone(),
        None,
        None, // CancellationToken will be created when the install actually starts
    );

    // Add to queue
    {
        let mut queue = state.modpack_install_queue.lock().await;
        queue.push_back(QueuedModpackInstall {
            queue_id: queue_id.clone(),
            platform,
            modpack_id,
            version_id,
            instance_name,
            modpack_name: modpack_name.clone(),
        });
    }

    // Trigger queue processing
    process_modpack_queue(app_handle).await;

    Ok(ModpackInstallQueued {
        queue_id,
        modpack_name,
    })
}

/// Process the modpack install queue - start installs up to the concurrent limit
fn process_modpack_queue(
    app_handle: AppHandle,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
    Box::pin(async move {
        process_modpack_queue_impl(app_handle).await;
    })
}

async fn process_modpack_queue_impl(app_handle: AppHandle) {
    let state = app_handle.state::<AppState>();
    let max_concurrent = {
        let settings = state.settings.read();
        settings.concurrent_downloads.max(1) as usize
    };

    loop {
        // Count currently running modpack installs via task registry
        let active_count = state
            .task_registry
            .list()
            .iter()
            .filter(|t| {
                t.task_type == crate::task_registry::TaskType::ModpackInstall
                    && t.status == crate::task_registry::TaskStatusKind::Running
            })
            .count();

        if active_count >= max_concurrent {
            break;
        }

        // Dequeue next item
        let next_item = {
            let mut queue = state.modpack_install_queue.lock().await;
            queue.pop_front()
        };

        let Some(item) = next_item else { break };

        // Spawn the install task
        let handle_clone = app_handle.clone();
        tokio::spawn(async move {
            let state = handle_clone.state::<AppState>();

            // Create cancellation token for this install
            let cancel_token = tokio_util::sync::CancellationToken::new();

            // Emit started event
            let _ = handle_clone.emit(
                "modpack_install_started",
                ModpackInstallStarted {
                    modpack_name: item.modpack_name.clone(),
                },
            );

            let result = match item.platform {
                ModpackPlatform::Modrinth => {
                    modpack_install_service::install_modrinth_modpack(
                        &state,
                        &item.modpack_id,
                        &item.version_id,
                        item.instance_name.clone(),
                        Some(&handle_clone),
                        Some(&cancel_token),
                        Some(&item.queue_id),
                    )
                    .await
                }
                ModpackPlatform::CurseForge => {
                    modpack_install_service::install_curseforge_modpack(
                        &state,
                        &item.modpack_id,
                        &item.version_id,
                        item.instance_name.clone(),
                        Some(&handle_clone),
                        Some(&cancel_token),
                        Some(&item.queue_id),
                    )
                    .await
                }
                ModpackPlatform::FTB => {
                    modpack_install_service::install_ftb_modpack(
                        &state,
                        &item.modpack_id,
                        &item.version_id,
                        item.instance_name.clone(),
                        Some(&handle_clone),
                        Some(&cancel_token),
                        Some(&item.queue_id),
                    )
                    .await
                }
                ModpackPlatform::Technic => {
                    modpack_install_service::install_technic_modpack(
                        &state,
                        &item.modpack_id,
                        &item.version_id,
                        item.instance_name.clone(),
                        Some(&handle_clone),
                        Some(&cancel_token),
                        Some(&item.queue_id),
                    )
                    .await
                }
                ModpackPlatform::ATLauncher => {
                    modpack_install_service::install_atlauncher_modpack(
                        &state,
                        &item.modpack_id,
                        &item.version_id,
                        item.instance_name.clone(),
                        Some(&handle_clone),
                        Some(&cancel_token),
                        Some(&item.queue_id),
                    )
                    .await
                }
            };

            // Emit completion/error/cancel events
            match result {
                Ok(instance) => {
                    app_info!(
                        "[modpack_cmd] install_modpack: success, instance_id={}",
                        instance.id
                    );
                    let _ = handle_clone.emit("modpack_install_complete", &instance);
                }
                Err(crate::error::AppError::Cancelled) => {
                    app_info!("[modpack_cmd] install_modpack: cancelled");
                    let _ = handle_clone.emit("modpack_install_cancelled", ());
                }
                Err(ref e) => {
                    app_info!("[modpack_cmd] install_modpack: error={:?}", e);
                    let _ = handle_clone.emit("modpack_install_error", e.to_string());
                }
            }

            // Process queue again to start next items
            tokio::spawn(process_modpack_queue(handle_clone));
        });
    }
}

/// Import an instance from a local .mrpack file
#[tauri::command]
pub async fn import_modpack_file(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    file_path: String,
    instance_name: Option<String>,
) -> Result<Instance, CommandError> {
    app_info!(
        "[modpack_cmd] import_modpack_file: file_path={}, instance_name={:?}",
        file_path,
        instance_name
    );

    let result = modpack_install_service::import_from_mrpack_file(
        &state,
        &file_path,
        instance_name,
        Some(&app_handle),
    )
    .await
    .map_err(CommandError::from);

    match &result {
        Ok(instance) => app_info!(
            "[modpack_cmd] import_modpack_file: success, instance_id={}",
            instance.id
        ),
        Err(e) => app_info!("[modpack_cmd] import_modpack_file: error={:?}", e),
    }
    result
}
