use std::fs;
use std::path::PathBuf;
use tauri::State;

use crate::error::CommandError;
use crate::models::MinecraftProfile;
use crate::services::{account_service, auth_service, skin_service};
use crate::state::AppState;

/// Save a skin to the local library
#[tauri::command]
pub fn save_skin_to_library(
    name: String,
    variant: String,
    skin_data: Vec<u8>,
) -> Result<skin_service::SavedSkin, CommandError> {
    skin_service::save_skin_to_library(name, variant, &skin_data).map_err(CommandError::from)
}

/// Get all saved skins from the library
#[tauri::command]
pub fn get_skin_library() -> Result<Vec<skin_service::SavedSkin>, CommandError> {
    skin_service::get_skin_library().map_err(CommandError::from)
}

/// Delete a skin from the library
#[tauri::command]
pub fn delete_skin_from_library(skin_id: String) -> Result<(), CommandError> {
    skin_service::delete_skin_from_library(&skin_id).map_err(CommandError::from)
}

/// Apply a skin from the library to the Minecraft account
#[tauri::command]
pub async fn apply_skin_from_library(
    state: State<'_, AppState>,
    account_id: String,
    skin_id: String,
) -> Result<MinecraftProfile, CommandError> {
    // Get the skin data from library
    let skin_data = skin_service::get_skin_data(&skin_id).map_err(CommandError::from)?;

    // Get the skin metadata for the variant
    let library = skin_service::get_skin_library().map_err(CommandError::from)?;
    let skin = library.iter().find(|s| s.id == skin_id).ok_or_else(|| {
        CommandError::from(crate::error::AppError::NotFound("Skin not found".into()))
    })?;

    let variant = skin.variant.clone();

    // Get access token
    let access_token = account_service::get_valid_access_token(&state.http_client, &account_id)
        .await
        .map_err(CommandError::from)?;

    // Upload the skin
    auth_service::upload_skin(&state.http_client, &access_token, &variant, &skin_data)
        .await
        .map_err(CommandError::from)?;

    // Fetch updated profile
    let profile = auth_service::get_minecraft_profile(&state.http_client, &access_token)
        .await
        .map_err(CommandError::from)?;

    // Update stored account with new skin URL
    let skin_url = profile.skins.first().map(|s| s.url.clone());
    let cape_url = profile
        .capes
        .iter()
        .find(|c| c.state == "ACTIVE")
        .map(|c| c.url.clone());

    account_service::update_account_profile(&account_id, profile.name.clone(), skin_url, cape_url)
        .map_err(CommandError::from)?;

    Ok(profile)
}

/// Get skin data from library (for preview)
#[tauri::command]
pub fn get_skin_data(skin_id: String) -> Result<Vec<u8>, CommandError> {
    skin_service::get_skin_data(&skin_id).map_err(CommandError::from)
}

/// Read a skin file from a path (for importing)
#[tauri::command]
pub fn read_skin_file(file_path: String) -> Result<Vec<u8>, CommandError> {
    let path = PathBuf::from(&file_path);

    // Validate the file exists and is a PNG
    if !path.exists() {
        return Err(CommandError::from(crate::error::AppError::NotFound(
            format!("File not found: {}", file_path),
        )));
    }

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    if extension != Some("png".to_string()) {
        return Err(CommandError::from(crate::error::AppError::InvalidInput(
            "File must be a PNG image".to_string(),
        )));
    }

    // Read the file
    let data =
        fs::read(&path).map_err(|e| CommandError::from(crate::error::AppError::IoError(e)))?;

    // Validate file size (1MB max)
    if data.len() > 1024 * 1024 {
        return Err(CommandError::from(crate::error::AppError::InvalidInput(
            "File is too large. Maximum size is 1MB.".to_string(),
        )));
    }

    Ok(data)
}
