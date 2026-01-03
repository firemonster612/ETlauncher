use tauri::State;

use crate::error::CommandError;
use crate::models::{MinecraftAccount, MinecraftProfile};
use crate::services::{account_service, auth_service};
use crate::state::AppState;

/// Get all accounts
#[tauri::command]
pub fn get_accounts() -> Result<Vec<MinecraftAccount>, CommandError> {
    account_service::get_accounts().map_err(CommandError::from)
}

/// Get account by ID
#[tauri::command]
pub fn get_account(account_id: String) -> Result<MinecraftAccount, CommandError> {
    account_service::get_account(&account_id).map_err(CommandError::from)
}

/// Get the active account
#[tauri::command]
pub fn get_active_account() -> Result<Option<MinecraftAccount>, CommandError> {
    account_service::get_active_account().map_err(CommandError::from)
}

/// Set account as active
#[tauri::command]
pub fn set_active_account(account_id: String) -> Result<Vec<MinecraftAccount>, CommandError> {
    account_service::set_active_account(&account_id).map_err(CommandError::from)
}

/// Delete an account (logout)
#[tauri::command]
pub fn delete_account(account_id: String) -> Result<Vec<MinecraftAccount>, CommandError> {
    account_service::delete_account(&account_id).map_err(CommandError::from)
}

/// Get Minecraft profile with skins and capes
#[tauri::command]
pub async fn get_minecraft_profile(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<MinecraftProfile, CommandError> {
    let access_token = account_service::get_valid_access_token(&state.http_client, &account_id)
        .await
        .map_err(CommandError::from)?;

    auth_service::get_minecraft_profile(&state.http_client, &access_token)
        .await
        .map_err(CommandError::from)
}

/// Upload a new skin
#[tauri::command]
pub async fn upload_skin(
    state: State<'_, AppState>,
    account_id: String,
    variant: String, // "classic" or "slim"
    skin_data: Vec<u8>,
) -> Result<MinecraftProfile, CommandError> {
    let access_token = account_service::get_valid_access_token(&state.http_client, &account_id)
        .await
        .map_err(CommandError::from)?;

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

/// Set active cape
#[tauri::command]
pub async fn set_cape(
    state: State<'_, AppState>,
    account_id: String,
    cape_id: String,
) -> Result<MinecraftProfile, CommandError> {
    let access_token = account_service::get_valid_access_token(&state.http_client, &account_id)
        .await
        .map_err(CommandError::from)?;

    auth_service::set_cape(&state.http_client, &access_token, &cape_id)
        .await
        .map_err(CommandError::from)?;

    // Fetch updated profile
    let profile = auth_service::get_minecraft_profile(&state.http_client, &access_token)
        .await
        .map_err(CommandError::from)?;

    // Update stored account
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

/// Hide cape (remove active cape)
#[tauri::command]
pub async fn hide_cape(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<MinecraftProfile, CommandError> {
    let access_token = account_service::get_valid_access_token(&state.http_client, &account_id)
        .await
        .map_err(CommandError::from)?;

    auth_service::hide_cape(&state.http_client, &access_token)
        .await
        .map_err(CommandError::from)?;

    // Fetch updated profile
    let profile = auth_service::get_minecraft_profile(&state.http_client, &access_token)
        .await
        .map_err(CommandError::from)?;

    // Update stored account
    let skin_url = profile.skins.first().map(|s| s.url.clone());

    account_service::update_account_profile(&account_id, profile.name.clone(), skin_url, None)
        .map_err(CommandError::from)?;

    Ok(profile)
}
