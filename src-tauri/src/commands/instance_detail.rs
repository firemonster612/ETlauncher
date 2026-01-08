use crate::error::CommandError;
use crate::models::instance_detail::{
    InstanceDetail, ScreenshotsResponse, ServersResponse, WorldsResponse,
};
use crate::services::{instance_detail_service, instance_service, launch_service};
use crate::state::AppState;
use tauri::{AppHandle, State};
use std::process::Command;

/// Get instance screenshots (all)
#[tauri::command]
pub fn get_instance_screenshots(
    instance_id: String,
    state: State<'_, AppState>,
) -> Result<ScreenshotsResponse, CommandError> {
    let game_dir = instance_service::get_game_directory(&state, &instance_id);
    instance_detail_service::get_screenshots(&game_dir)
        .map_err(CommandError::from)
}

/// Get instance worlds (all)
#[tauri::command]
pub fn get_instance_worlds(
    instance_id: String,
    state: State<'_, AppState>,
) -> Result<WorldsResponse, CommandError> {
    let game_dir = instance_service::get_game_directory(&state, &instance_id);
    instance_detail_service::get_worlds(&game_dir).map_err(CommandError::from)
}

/// Get instance servers (all)
#[tauri::command]
pub fn get_instance_servers(
    instance_id: String,
    state: State<'_, AppState>,
) -> Result<ServersResponse, CommandError> {
    let game_dir = instance_service::get_game_directory(&state, &instance_id);
    instance_detail_service::get_servers(&game_dir).map_err(CommandError::from)
}

/// Get overview detail for dashboard view
#[tauri::command]
pub fn get_instance_detail(
    instance_id: String,
    state: State<'_, AppState>,
) -> Result<InstanceDetail, CommandError> {
    let instance = instance_service::get_instance(&state, &instance_id).map_err(CommandError::from)?;
    let game_dir = instance_service::get_game_directory(&state, &instance_id);

    instance_detail_service::get_instance_detail(&game_dir, instance.total_play_time)
        .map_err(CommandError::from)
}

/// Return base64-encoded screenshot data for lightbox
#[tauri::command]
pub fn get_screenshot_data(
    instance_id: String,
    filename: String,
    state: State<'_, AppState>,
) -> Result<String, CommandError> {
    let game_dir = instance_service::get_game_directory(&state, &instance_id);
    instance_detail_service::get_screenshot_data(&game_dir, &filename)
        .map_err(CommandError::from)
}

/// Launch directly into a world (quick play)
#[tauri::command]
pub async fn launch_into_world(
    instance_id: String,
    account_id: String,
    world_folder: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<u32, CommandError> {
    if state.is_instance_running(&instance_id) {
        return Err(CommandError {
            code: "ALREADY_RUNNING".to_string(),
            message: "Instance is already running".to_string(),
        });
    }

    let game_dir = instance_service::get_game_directory(&state, &instance_id);
    if !game_dir.join("saves").join(&world_folder).exists() {
        return Err(CommandError {
            code: "WORLD_NOT_FOUND".to_string(),
            message: format!("World '{}' not found", world_folder),
        });
    }

    let instance = instance_service::get_instance(&state, &instance_id).map_err(CommandError::from)?;

    launch_service::launch_instance_with_quick_play(
        &instance,
        &account_id,
        &app_handle,
        launch_service::QuickPlayTarget::World(world_folder),
    )
    .await
    .map_err(CommandError::from)
}

/// Launch directly into a server (quick play)
#[tauri::command]
pub async fn launch_into_server(
    instance_id: String,
    account_id: String,
    server_ip: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<u32, CommandError> {
    if state.is_instance_running(&instance_id) {
        return Err(CommandError {
            code: "ALREADY_RUNNING".to_string(),
            message: "Instance is already running".to_string(),
        });
    }

    let instance = instance_service::get_instance(&state, &instance_id).map_err(CommandError::from)?;

    launch_service::launch_instance_with_quick_play(
        &instance,
        &account_id,
        &app_handle,
        launch_service::QuickPlayTarget::Server(server_ip),
    )
    .await
    .map_err(CommandError::from)
}

/// Open a world folder in the file explorer
#[tauri::command]
pub fn open_world_folder(
    instance_id: String,
    world_folder: String,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let game_dir = instance_service::get_game_directory(&state, &instance_id);
    let world_path = game_dir.join("saves").join(&world_folder);

    if !world_path.exists() {
        return Err(CommandError {
            code: "FOLDER_NOT_FOUND".to_string(),
            message: format!("World folder does not exist: {:?}", world_path),
        });
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&world_path)
            .spawn()
            .map_err(|e| CommandError {
                code: "OPEN_FOLDER_FAILED".to_string(),
                message: format!("Failed to open folder: {}", e),
            })?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&world_path)
            .spawn()
            .map_err(|e| CommandError {
                code: "OPEN_FOLDER_FAILED".to_string(),
                message: format!("Failed to open folder: {}", e),
            })?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(&world_path)
            .spawn()
            .map_err(|e| CommandError {
                code: "OPEN_FOLDER_FAILED".to_string(),
                message: format!("Failed to open folder: {}", e),
            })?;
    }

    Ok(())
}

/// Delete a screenshot file
#[tauri::command]
pub fn delete_screenshot(
    instance_id: String,
    filename: String,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let game_dir = instance_service::get_game_directory(&state, &instance_id);
    let screenshot_path = game_dir.join("screenshots").join(&filename);

    if !screenshot_path.exists() {
        return Err(CommandError {
            code: "FILE_NOT_FOUND".to_string(),
            message: format!("Screenshot does not exist: {:?}", screenshot_path),
        });
    }

    std::fs::remove_file(&screenshot_path).map_err(|e| CommandError {
        code: "DELETE_FAILED".to_string(),
        message: format!("Failed to delete screenshot: {}", e),
    })?;

    Ok(())
}

/// Delete a world folder
#[tauri::command]
pub fn delete_world(
    instance_id: String,
    world_folder: String,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let game_dir = instance_service::get_game_directory(&state, &instance_id);
    let world_path = game_dir.join("saves").join(&world_folder);

    if !world_path.exists() {
        return Err(CommandError {
            code: "FOLDER_NOT_FOUND".to_string(),
            message: format!("World folder does not exist: {:?}", world_path),
        });
    }

    std::fs::remove_dir_all(&world_path).map_err(|e| CommandError {
        code: "DELETE_FAILED".to_string(),
        message: format!("Failed to delete world: {}", e),
    })?;

    Ok(())
}
