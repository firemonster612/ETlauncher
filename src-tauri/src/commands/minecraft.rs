use crate::error::CommandError;
use crate::models::minecraft::{VersionEntry, VersionInfo, VersionManifest};
use crate::services::download_service;
use crate::state::AppState;
use tauri::{AppHandle, State};

/// Fetch the version manifest from Mojang
#[tauri::command]
pub async fn fetch_version_manifest(
    state: State<'_, AppState>,
    force_refresh: bool,
) -> Result<VersionManifest, CommandError> {
    download_service::fetch_version_manifest(&state.http_client, force_refresh)
        .await
        .map_err(CommandError::from)
}

/// Get filtered versions (releases, optionally snapshots and old versions)
#[tauri::command]
pub async fn get_versions(
    state: State<'_, AppState>,
    show_snapshots: bool,
    show_old_versions: bool,
) -> Result<Vec<VersionEntry>, CommandError> {
    let manifest = download_service::fetch_version_manifest(&state.http_client, false)
        .await
        .map_err(CommandError::from)?;

    let filtered = download_service::filter_versions(&manifest, show_snapshots, show_old_versions);

    Ok(filtered.into_iter().cloned().collect())
}

/// Get detailed version info
#[tauri::command]
pub async fn get_version_info(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<VersionInfo, CommandError> {
    download_service::get_version_info(&state.http_client, &version_id)
        .await
        .map_err(CommandError::from)
}

/// Download all game files for an instance
#[tauri::command]
pub async fn download_game_files(
    instance_id: String,
    version_id: String,
    app_handle: AppHandle,
) -> Result<(), CommandError> {
    download_service::download_game_files(&instance_id, &version_id, Some(&app_handle))
        .await
        .map_err(CommandError::from)
}
