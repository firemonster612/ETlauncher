use crate::error::CommandError;
use crate::models::instance::{CreateInstanceRequest, Instance, UpdateInstanceRequest};
use crate::services::instance_service;
use crate::state::AppState;
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
