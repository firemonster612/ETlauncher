use crate::error::CommandError;
use crate::services::{instance_service, optifine_service};
use crate::state::AppState;
use crate::utils::paths::get_instance_game_dir_with_base;
use tauri::State;

/// Check if OptiFine is available for a specific Minecraft version
#[tauri::command]
pub async fn check_optifine_available(
    state: State<'_, AppState>,
    mc_version: String,
) -> Result<bool, CommandError> {
    optifine_service::check_optifine_available(&state.http_client, &mc_version)
        .await
        .map_err(CommandError::from)
}

/// Install OptiFine for an instance
/// Returns the filename of the installed OptiFine jar
#[tauri::command]
pub async fn install_optifine(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<String, CommandError> {
    // Get the instance to find its MC version and mods directory
    let instance = instance_service::get_instance(&state, &instance_id)
        .map_err(CommandError::from)?;
    
    let mc_version = &instance.minecraft_version;
    
    // Get the mods directory path
    let instances_base = state.settings.read().instances_path.clone();
    let game_dir = get_instance_game_dir_with_base(&instances_base, &instance_id);
    let mods_dir = game_dir.join("mods");
    
    // Download OptiFine to the mods directory
    let filename = optifine_service::download_optifine(
        &state.http_client,
        mc_version,
        &mods_dir,
    )
    .await
    .map_err(CommandError::from)?;
    
    Ok(filename)
}

/// Get OptiFine version info for a specific Minecraft version
/// Returns None if not available
#[tauri::command]
pub async fn get_optifine_version(
    state: State<'_, AppState>,
    mc_version: String,
) -> Result<Option<optifine_service::OptifineVersion>, CommandError> {
    optifine_service::get_optifine_for_mc_version(&state.http_client, &mc_version)
        .await
        .map_err(CommandError::from)
}
