use crate::error::CommandError;
use crate::models::instance::{CreateInstanceRequest, Instance, UpdateInstanceRequest};
use crate::services::{instance_service, instance_export_service};
use crate::state::AppState;
use std::path::PathBuf;
use std::process::Command;
use tauri::State;

/// Get all instances
#[tauri::command]
pub fn get_instances(state: State<'_, AppState>) -> Result<Vec<Instance>, CommandError> {
    instance_service::get_all_instances(&state).map_err(CommandError::from)
}

/// Get a single instance by ID
#[tauri::command]
pub fn get_instance(state: State<'_, AppState>, instance_id: String) -> Result<Instance, CommandError> {
    instance_service::get_instance(&state, &instance_id).map_err(CommandError::from)
}

/// Create a new instance
#[tauri::command]
pub fn create_instance(state: State<'_, AppState>, request: CreateInstanceRequest) -> Result<Instance, CommandError> {
    instance_service::create_instance(&state, request).map_err(CommandError::from)
}

/// Update an existing instance
#[tauri::command]
pub fn update_instance(
    state: State<'_, AppState>,
    instance_id: String,
    updates: UpdateInstanceRequest,
) -> Result<Instance, CommandError> {
    instance_service::update_instance(&state, &instance_id, updates).map_err(CommandError::from)
}

/// Delete an instance
#[tauri::command]
pub fn delete_instance(state: State<'_, AppState>, instance_id: String, delete_files: bool) -> Result<(), CommandError> {
    instance_service::delete_instance(&state, &instance_id, delete_files).map_err(CommandError::from)
}

/// Duplicate an instance with a new name
#[tauri::command]
pub fn duplicate_instance(state: State<'_, AppState>, instance_id: String, new_name: String) -> Result<Instance, CommandError> {
    instance_service::duplicate_instance(&state, &instance_id, new_name).map_err(CommandError::from)
}

/// Open the game folder for an instance in the file explorer
#[tauri::command]
pub fn open_instance_folder(state: State<'_, AppState>, instance_id: String) -> Result<(), CommandError> {
    let game_dir = instance_service::get_game_directory(&state, &instance_id);

    if !game_dir.exists() {
        return Err(CommandError {
            code: "FOLDER_NOT_FOUND".to_string(),
            message: format!("Game folder does not exist: {:?}", game_dir),
        });
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&game_dir)
            .spawn()
            .map_err(|e| CommandError {
                code: "OPEN_FOLDER_FAILED".to_string(),
                message: format!("Failed to open folder: {}", e),
            })?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&game_dir)
            .spawn()
            .map_err(|e| CommandError {
                code: "OPEN_FOLDER_FAILED".to_string(),
                message: format!("Failed to open folder: {}", e),
            })?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(&game_dir)
            .spawn()
            .map_err(|e| CommandError {
                code: "OPEN_FOLDER_FAILED".to_string(),
                message: format!("Failed to open folder: {}", e),
            })?;
    }

    Ok(())
}

/// Export an instance to Modrinth .mrpack format
#[tauri::command]
pub async fn export_instance(
    state: State<'_, AppState>,
    instance_id: String,
    output_path: String,
) -> Result<String, CommandError> {
    let path = PathBuf::from(&output_path);
    let result_path = instance_export_service::export_to_mrpack(&state, &instance_id, &path)
        .await
        .map_err(CommandError::from)?;

    Ok(result_path.to_string_lossy().to_string())
}
