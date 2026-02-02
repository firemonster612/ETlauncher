use serde::Serialize;
use std::path::Path;
use tauri::{State, WebviewWindow};
use uuid::Uuid;

use crate::cache::clear_all_disk_caches;
use crate::error::CommandError;
use crate::models::{AppSettings, UpdateSettingsRequest};
use crate::services::settings_service;
use crate::state::{ApiCache, AppState};
use crate::utils::paths;

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
    let settings = AppSettings {
        instances_path: settings_service::get_default_instances_path(),
        ..Default::default()
    };

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

/// Response from cache clearing operation
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheClearResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// Number of in-memory caches cleared
    pub memory_caches_cleared: u32,
    /// Whether disk cache was cleared
    pub disk_cache_cleared: bool,
}

/// Clear all API caches (for troubleshooting)
#[tauri::command]
pub fn clear_api_caches(state: State<'_, AppState>) -> Result<CacheClearResult, CommandError> {
    // Clear in-memory caches
    state.api_cache.clear_all();

    // Clear disk caches
    let disk_cleared = clear_all_disk_caches().is_ok();

    Ok(CacheClearResult {
        success: true,
        memory_caches_cleared: ApiCache::CACHE_COUNT,
        disk_cache_cleared: disk_cleared,
    })
}

// Note: Folder picker will be handled via tauri-plugin-dialog in frontend
// The frontend will use the dialog plugin directly

/// Get the system theme (light or dark)
/// This uses multiple detection methods for cross-platform support
#[tauri::command]
pub fn get_system_theme(window: WebviewWindow) -> String {
    // On Linux, try to detect via gsettings (GNOME/GTK color-scheme)
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "color-scheme"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let scheme = stdout.trim().trim_matches('\'');
            if scheme == "prefer-light" || scheme == "default" {
                return "light".to_string();
            } else if scheme == "prefer-dark" {
                return "dark".to_string();
            }
        }
    }

    // Fallback to Tauri's window theme detection
    match window.theme() {
        Ok(tauri::Theme::Light) => "light".to_string(),
        Ok(tauri::Theme::Dark) | Ok(_) => "dark".to_string(),
        Err(_) => "dark".to_string(),
    }
}

/// Copy a background file to the backgrounds directory
/// Returns the new filename (uuid-based)
#[tauri::command]
pub fn copy_background_file(source_path: String) -> Result<String, CommandError> {
    use crate::error::AppError;

    let source = Path::new(&source_path);

    // Validate source exists
    if !source.exists() {
        return Err(
            AppError::InvalidInput(format!("Source file does not exist: {}", source_path)).into(),
        );
    }

    // Get extension from source file
    let extension = source.extension().and_then(|e| e.to_str()).unwrap_or("bin");

    // Generate UUID-based filename
    let new_filename = format!("{}.{}", Uuid::new_v4(), extension);

    // Ensure backgrounds directory exists
    let backgrounds_dir = paths::get_backgrounds_dir();
    std::fs::create_dir_all(&backgrounds_dir).map_err(AppError::from)?;

    // Copy file to backgrounds directory
    let dest_path = backgrounds_dir.join(&new_filename);
    std::fs::copy(source, &dest_path).map_err(AppError::from)?;

    Ok(new_filename)
}

/// Delete a background file from the backgrounds directory
#[tauri::command]
pub fn delete_background_file(filename: String) -> Result<(), CommandError> {
    use crate::error::AppError;

    // Validate filename doesn't contain path traversal
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(AppError::InvalidInput("Invalid filename".to_string()).into());
    }

    let file_path = paths::get_backgrounds_dir().join(&filename);

    // Only delete if it exists
    if file_path.exists() {
        std::fs::remove_file(&file_path).map_err(AppError::from)?;
    }

    Ok(())
}

/// Get the full path to a background file
#[tauri::command]
pub fn get_background_path(filename: String) -> Result<String, CommandError> {
    use crate::error::AppError;

    // Validate filename doesn't contain path traversal
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(AppError::InvalidInput("Invalid filename".to_string()).into());
    }

    let file_path = paths::get_backgrounds_dir().join(&filename);

    if !file_path.exists() {
        return Err(AppError::InvalidInput(format!(
            "Background file does not exist: {}",
            filename
        ))
        .into());
    }

    Ok(file_path.to_string_lossy().to_string())
}

/// Get background file data as base64
#[tauri::command]
pub fn get_background_data(filename: String) -> Result<String, CommandError> {
    use crate::error::AppError;

    // Validate filename doesn't contain path traversal
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(AppError::InvalidInput("Invalid filename".to_string()).into());
    }

    let file_path = paths::get_backgrounds_dir().join(&filename);

    if !file_path.exists() {
        return Err(AppError::InvalidInput(format!(
            "Background file does not exist: {}",
            filename
        ))
        .into());
    }

    let data = std::fs::read(&file_path).map_err(AppError::from)?;
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &data,
    ))
}
