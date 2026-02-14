use crate::cache::{load_disk_cache, sanitize_cache_key, save_disk_cache};
use crate::error::CommandError;
use crate::models::instance::LoaderType;
use crate::models::loader::{LoaderInstallProgress, LoaderVersion};
use crate::services::instance_service;
use crate::services::loader_service;
use crate::state::AppState;
use crate::task_registry::{TaskProgress, TaskType};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

/// TTL for loader versions disk cache (1 hour)
const LOADER_VERSIONS_DISK_TTL: Duration = Duration::from_secs(3600);

/// Get available loader versions for a specific loader type and Minecraft version
#[tauri::command]
pub async fn get_loader_versions(
    loader_type: String,
    minecraft_version: String,
    state: State<'_, AppState>,
) -> Result<Vec<LoaderVersion>, CommandError> {
    let loader_type = loader_type
        .parse::<LoaderType>()
        .map_err(|_| CommandError {
            code: "INVALID_LOADER_TYPE".to_string(),
            message: format!("Invalid loader type: {}", loader_type),
        })?;

    let cache_key = (loader_type, minecraft_version.clone());

    // Check in-memory cache first
    if let Some(cached) = state.api_cache.loader_versions.get(&cache_key) {
        return Ok(cached);
    }

    // Check disk cache
    let disk_cache_key = sanitize_cache_key(&format!("{}-{}", loader_type, minecraft_version));
    if let Some(cached) = load_disk_cache::<Vec<LoaderVersion>>("loader_versions", &disk_cache_key)
    {
        // Store in memory cache for faster subsequent access
        state
            .api_cache
            .loader_versions
            .insert(cache_key, cached.clone());
        return Ok(cached);
    }

    // Fetch from API
    let versions = loader_service::get_loader_versions(loader_type, &minecraft_version).await?;

    // Store in both caches
    state
        .api_cache
        .loader_versions
        .insert(cache_key, versions.clone());
    let _ = save_disk_cache(
        "loader_versions",
        &disk_cache_key,
        &versions,
        LOADER_VERSIONS_DISK_TTL,
    );

    Ok(versions)
}

/// Install a mod loader to an instance
#[tauri::command]
pub async fn install_loader(
    instance_id: String,
    loader_type: String,
    loader_version: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let loader_type = loader_type
        .parse::<LoaderType>()
        .map_err(|_| CommandError {
            code: "INVALID_LOADER_TYPE".to_string(),
            message: format!("Invalid loader type: {}", loader_type),
        })?;

    let instance = instance_service::get_instance(&state, &instance_id)?;

    let game_dir = instance_service::get_game_directory(&state, &instance_id);

    let app_handle = Arc::new(app_handle);
    let mc_version = instance.minecraft_version.clone();

    // Register a task in the task registry so the sidebar shows progress
    let task_id = format!("loader-install-{}", instance_id);
    if let Some(app_state) = app_handle.try_state::<AppState>() {
        app_state.task_registry.register(
            task_id.clone(),
            TaskType::LoaderInstall,
            format!("Installing {} {}", loader_type, loader_version),
            Some(instance_id.clone()),
            None,
        );
        app_state.task_registry.start(&task_id);
    }

    let task_id_for_cb = task_id.clone();
    let app_handle_for_cb = app_handle.clone();
    let progress_callback = move |stage: String, percent: u32| {
        let progress = LoaderInstallProgress {
            stage: stage.clone(),
            progress: percent,
            current_file: None,
            total_bytes: None,
            downloaded_bytes: None,
        };

        let _ = app_handle_for_cb.emit("loader-install-progress", progress);
        let _ = app_handle_for_cb.emit("loader-install-status", format!("{}: {}%", stage, percent));

        // Update the task registry progress
        if let Some(app_state) = app_handle_for_cb.try_state::<AppState>() {
            app_state.task_registry.update_progress(
                &task_id_for_cb,
                TaskProgress {
                    current: percent as u64,
                    total: 100,
                    percent: Some(percent as f64),
                    speed_bytes_per_sec: None,
                    current_item: None,
                    stage: Some(stage),
                },
            );
        }
    };

    let result = loader_service::install_loader(
        &game_dir,
        loader_type,
        &mc_version,
        &loader_version,
        progress_callback,
    )
    .await;

    // Update task registry with final status
    match &result {
        Ok(()) => {
            if let Some(app_state) = app_handle.try_state::<AppState>() {
                app_state.task_registry.complete(&task_id);
            }
            let _ = app_handle.emit("loader-install-complete", ());
        }
        Err(e) => {
            if let Some(app_state) = app_handle.try_state::<AppState>() {
                app_state.task_registry.fail(&task_id, e.to_string());
            }
        }
    }

    result.map_err(|e| CommandError {
        code: "LOADER_INSTALLATION_FAILED".to_string(),
        message: format!("Failed to install loader: {}", e),
    })?;

    Ok(())
}

/// Check if a loader is installed for an instance
#[tauri::command]
pub fn check_loader_installed(
    instance_id: String,
    loader_type: String,
    loader_version: String,
    state: State<'_, AppState>,
) -> Result<bool, CommandError> {
    let loader_type = loader_type
        .parse::<LoaderType>()
        .map_err(|_| CommandError {
            code: "INVALID_LOADER_TYPE".to_string(),
            message: format!("Invalid loader type: {}", loader_type),
        })?;

    let instance = instance_service::get_instance(&state, &instance_id)?;

    let game_dir = instance_service::get_game_directory(&state, &instance_id);
    let mc_version = &instance.minecraft_version;

    let installed = loader_service::check_loader_installed(
        &game_dir,
        loader_type,
        mc_version,
        &loader_version,
    )?;

    Ok(installed)
}
