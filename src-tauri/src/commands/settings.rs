use tauri::State;

use crate::error::CommandError;
use crate::models::{AppSettings, UpdateSettingsRequest};
use crate::services::settings_service;
use crate::state::AppState;

/// Get current application settings
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, CommandError> {
    Ok(state.get_settings())
}

/// Update application settings
#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    updates: UpdateSettingsRequest,
) -> Result<AppSettings, CommandError> {
    // Get current settings
    let current = state.get_settings();

    // Merge updates
    let new_settings = settings_service::update_settings(&current, updates);

    // Validate the new settings
    settings_service::validate_settings(&new_settings).map_err(CommandError::from)?;

    // Update state
    {
        let mut settings = state.settings.write();
        *settings = new_settings.clone();
    }

    // Save to disk
    state.save_settings().map_err(CommandError::from)?;

    Ok(new_settings)
}

/// Reset settings to defaults
#[tauri::command]
pub fn reset_settings(state: State<'_, AppState>) -> Result<AppSettings, CommandError> {
    let mut settings = AppSettings::default();
    settings.instances_path = settings_service::get_default_instances_path();

    // Update state
    {
        let mut current = state.settings.write();
        *current = settings.clone();
    }

    // Save to disk
    state.save_settings().map_err(CommandError::from)?;

    Ok(settings)
}

/// Get the default instances path
#[tauri::command]
pub fn get_default_instances_path() -> String {
    settings_service::get_default_instances_path()
}

// Note: Folder picker will be handled via tauri-plugin-dialog in frontend
// The frontend will use the dialog plugin directly
