//! Update commands
//!
//! Tauri commands for checking and executing updates.

use crate::error::CommandError;
use crate::models::content::{
    InstalledContentManifest, InstanceUpdateCheck, InstanceUpdatePlan, ModpackInstanceUpdateCheck,
    ModpackUpdateInfo, ModpackUpdatePlan, UpdateCheckResult, UpdatePlan,
};
use crate::models::instance::LoaderType;
use crate::services::{manifest_service, update_execution_service, update_service};
use crate::state::AppState;
use tauri::{AppHandle, Emitter, Manager, State};

/// Response for queued update operations
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateQueued {
    pub task_id: String,
    pub instance_id: String,
}

/// Check for modpack updates
#[tauri::command]
pub async fn check_modpack_update(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Option<ModpackUpdateInfo>, CommandError> {
    update_service::check_modpack_update(&state, &instance_id)
        .await
        .map_err(CommandError::from)
}

/// Check for content updates (same MC version)
#[tauri::command]
pub async fn check_content_updates(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<UpdateCheckResult, CommandError> {
    update_service::check_content_updates(&state, &instance_id)
        .await
        .map_err(CommandError::from)
}

/// Preview migration to a different MC version
#[tauri::command]
pub async fn preview_version_migration(
    state: State<'_, AppState>,
    instance_id: String,
    target_mc_version: String,
    target_loader: LoaderType,
) -> Result<UpdateCheckResult, CommandError> {
    update_service::check_version_migration(
        &state,
        &instance_id,
        &target_mc_version,
        &target_loader,
    )
    .await
    .map_err(CommandError::from)
}

/// Execute content updates
#[tauri::command]
pub async fn update_instance_content(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    instance_id: String,
    plan: UpdatePlan,
) -> Result<(), CommandError> {
    update_execution_service::execute_content_update(&state, &instance_id, &plan, Some(&app_handle))
        .await
        .map_err(CommandError::from)
}

/// Execute MC version migration (fire-and-forget)
#[tauri::command]
pub async fn migrate_instance_version(
    _state: State<'_, AppState>,
    app_handle: AppHandle,
    instance_id: String,
    target_mc_version: String,
    target_loader: LoaderType,
    target_loader_version: String,
    plan: UpdatePlan,
) -> Result<UpdateQueued, CommandError> {
    let task_id = uuid::Uuid::new_v4().to_string();
    let handle_clone = app_handle.clone();
    let instance_id_clone = instance_id.clone();

    tokio::spawn(async move {
        let state = handle_clone.state::<AppState>();
        let result = update_execution_service::execute_version_migration(
            &state,
            &instance_id_clone,
            &target_mc_version,
            &target_loader,
            &target_loader_version,
            &plan,
            Some(&handle_clone),
        )
        .await;

        match result {
            Ok(instance) => {
                let _ = handle_clone.emit("instance_update_complete", &instance);
            }
            Err(e) => {
                let _ = handle_clone.emit("instance_update_error", e.to_string());
            }
        }
    });

    Ok(UpdateQueued {
        task_id,
        instance_id,
    })
}

/// Get content manifest for an instance
#[tauri::command]
pub async fn get_content_manifest(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<InstalledContentManifest, CommandError> {
    manifest_service::load_manifest(&state, &instance_id).map_err(CommandError::from)
}

// =============================================================================
// NEW UPDATE SYSTEM COMMANDS
// =============================================================================

/// Check for modpack instance updates (returns all available versions)
#[tauri::command]
pub async fn check_modpack_instance_updates(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<ModpackInstanceUpdateCheck, CommandError> {
    update_service::check_modpack_instance_updates(&state, &instance_id)
        .await
        .map_err(CommandError::from)
}

/// Execute modpack update (fire-and-forget)
#[tauri::command]
pub async fn execute_modpack_update(
    _state: State<'_, AppState>,
    app_handle: AppHandle,
    instance_id: String,
    plan: ModpackUpdatePlan,
) -> Result<UpdateQueued, CommandError> {
    let task_id = uuid::Uuid::new_v4().to_string();
    let handle_clone = app_handle.clone();
    let instance_id_clone = instance_id.clone();

    tokio::spawn(async move {
        let state = handle_clone.state::<AppState>();
        let result = update_execution_service::execute_modpack_update(
            &state,
            &instance_id_clone,
            &plan,
            Some(&handle_clone),
        )
        .await;

        match result {
            Ok(instance) => {
                let _ = handle_clone.emit("instance_update_complete", &instance);
            }
            Err(e) => {
                let _ = handle_clone.emit("instance_update_error", e.to_string());
            }
        }
    });

    Ok(UpdateQueued {
        task_id,
        instance_id,
    })
}

/// Check for non-modpack instance updates (targets latest MC version)
#[tauri::command]
pub async fn check_instance_updates(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<InstanceUpdateCheck, CommandError> {
    update_service::check_instance_updates(&state, &instance_id)
        .await
        .map_err(CommandError::from)
}

/// Execute non-modpack instance update (fire-and-forget)
#[tauri::command]
pub async fn execute_instance_update(
    _state: State<'_, AppState>,
    app_handle: AppHandle,
    instance_id: String,
    plan: InstanceUpdatePlan,
) -> Result<UpdateQueued, CommandError> {
    let task_id = uuid::Uuid::new_v4().to_string();
    let handle_clone = app_handle.clone();
    let instance_id_clone = instance_id.clone();

    tokio::spawn(async move {
        let state = handle_clone.state::<AppState>();
        let result = update_execution_service::execute_instance_update(
            &state,
            &instance_id_clone,
            &plan,
            Some(&handle_clone),
        )
        .await;

        match result {
            Ok(instance) => {
                let _ = handle_clone.emit("instance_update_complete", &instance);
            }
            Err(e) => {
                let _ = handle_clone.emit("instance_update_error", e.to_string());
            }
        }
    });

    Ok(UpdateQueued {
        task_id,
        instance_id,
    })
}

/// Get the current executable path (used to detect if running from AppImage)
#[tauri::command]
pub fn get_exe_path() -> Result<String, CommandError> {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| CommandError::from(crate::error::AppError::IoError(e)))
}

/// Check if running from an AppImage (Linux only)
#[tauri::command]
pub fn is_appimage() -> bool {
    // The APPIMAGE environment variable is set when running from an AppImage
    std::env::var("APPIMAGE").is_ok()
}
