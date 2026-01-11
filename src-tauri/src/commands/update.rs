//! Update commands
//!
//! Tauri commands for checking and executing updates.

use crate::error::CommandError;
use crate::models::content::{
    InstalledContentManifest, InstanceUpdateCheck, InstanceUpdatePlan, ModpackInstanceUpdateCheck,
    ModpackUpdateInfo, ModpackUpdatePlan, UpdateCheckResult, UpdatePlan,
};
use crate::models::instance::{Instance, LoaderType};
use crate::services::{manifest_service, update_execution_service, update_service};
use crate::state::AppState;
use tauri::{AppHandle, State};

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

/// Execute MC version migration
#[tauri::command]
pub async fn migrate_instance_version(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    instance_id: String,
    target_mc_version: String,
    target_loader: LoaderType,
    target_loader_version: String,
    plan: UpdatePlan,
) -> Result<Instance, CommandError> {
    update_execution_service::execute_version_migration(
        &state,
        &instance_id,
        &target_mc_version,
        &target_loader,
        &target_loader_version,
        &plan,
        Some(&app_handle),
    )
    .await
    .map_err(CommandError::from)
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

/// Execute modpack update
#[tauri::command]
pub async fn execute_modpack_update(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    instance_id: String,
    plan: ModpackUpdatePlan,
) -> Result<Instance, CommandError> {
    update_execution_service::execute_modpack_update(&state, &instance_id, &plan, Some(&app_handle))
        .await
        .map_err(CommandError::from)
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

/// Execute non-modpack instance update
#[tauri::command]
pub async fn execute_instance_update(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    instance_id: String,
    plan: InstanceUpdatePlan,
) -> Result<Instance, CommandError> {
    update_execution_service::execute_instance_update(
        &state,
        &instance_id,
        &plan,
        Some(&app_handle),
    )
    .await
    .map_err(CommandError::from)
}
