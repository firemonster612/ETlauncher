use base64::Engine;
use tauri::State;

use crate::error::CommandError;
use crate::models::{AuthPollStatus, DeviceCodeResponse, MinecraftAccount};
use crate::services::{account_service, auth_service};
use crate::state::AppState;

/// Start device code authentication flow
#[tauri::command]
pub async fn start_device_auth(
    state: State<'_, AppState>,
) -> Result<DeviceCodeResponse, CommandError> {
    auth_service::start_device_code_flow(&state.http_client)
        .await
        .map_err(CommandError::from)
}

/// Poll for device code authentication status
#[tauri::command]
pub async fn poll_device_auth(
    state: State<'_, AppState>,
    device_code: String,
) -> Result<AuthPollStatus, CommandError> {
    let status = auth_service::poll_device_code(&state.http_client, &device_code)
        .await
        .map_err(CommandError::from)?;

    // If authentication succeeded, save the account
    if let AuthPollStatus::Success { ref account } = status {
        account_service::add_account(account.clone()).map_err(CommandError::from)?;
    }

    Ok(status)
}

/// Refresh account token
#[tauri::command]
pub async fn refresh_account_token(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<MinecraftAccount, CommandError> {
    let (_, expires_in) = auth_service::refresh_tokens(&state.http_client, &account_id)
        .await
        .map_err(CommandError::from)?;

    let now = chrono::Utc::now().timestamp();
    account_service::update_account_expiry(&account_id, now + expires_in)
        .map_err(CommandError::from)?;

    account_service::get_account(&account_id).map_err(CommandError::from)
}

#[tauri::command]
pub fn create_offline_account(username: String) -> Result<MinecraftAccount, CommandError> {
    account_service::create_offline_account(&username).map_err(CommandError::from)
}

#[tauri::command]
pub fn set_offline_skin(
    account_id: String,
    skin_data: Vec<u8>,
    variant: String,
) -> Result<String, CommandError> {
    account_service::set_offline_skin(&account_id, &skin_data, &variant).map_err(CommandError::from)
}

#[tauri::command]
pub fn set_offline_cape(account_id: String, cape_data: Vec<u8>) -> Result<String, CommandError> {
    account_service::set_offline_cape(&account_id, &cape_data).map_err(CommandError::from)
}

#[tauri::command]
pub fn remove_offline_skin(account_id: String) -> Result<(), CommandError> {
    account_service::remove_offline_skin(&account_id).map_err(CommandError::from)
}

#[tauri::command]
pub fn remove_offline_cape(account_id: String) -> Result<(), CommandError> {
    account_service::remove_offline_cape(&account_id).map_err(CommandError::from)
}

/// Get a bundled default skin (steve or alex) as raw PNG bytes
#[tauri::command]
pub fn get_default_skin(name: String) -> Result<Vec<u8>, CommandError> {
    const STEVE_SKIN: &[u8] = include_bytes!("../../assets/steve.png");
    const ALEX_SKIN: &[u8] = include_bytes!("../../assets/alex.png");

    match name.as_str() {
        "steve" => Ok(STEVE_SKIN.to_vec()),
        "alex" => Ok(ALEX_SKIN.to_vec()),
        _ => Err(CommandError::from(crate::error::AppError::InvalidInput(
            format!("Unknown default skin: {}", name),
        ))),
    }
}

/// Get the offline skin texture as base64 PNG data URL
#[tauri::command]
pub fn get_offline_skin_data(account_id: String) -> Result<Option<String>, CommandError> {
    let account = account_service::get_account(&account_id).map_err(CommandError::from)?;
    if let Some(ref hash) = account.offline_skin_hash {
        let skins_dir = crate::utils::paths::get_app_data_dir().join("skins");
        let skin_path = skins_dir.join(format!("{}.png", hash));
        if skin_path.exists() {
            let data = std::fs::read(&skin_path)
                .map_err(|e| CommandError::from(crate::error::AppError::IoError(e)))?;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
            Ok(Some(format!("data:image/png;base64,{}", b64)))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

/// Get the offline cape texture as base64 PNG data URL
#[tauri::command]
pub fn get_offline_cape_data(account_id: String) -> Result<Option<String>, CommandError> {
    let account = account_service::get_account(&account_id).map_err(CommandError::from)?;
    if let Some(ref hash) = account.offline_cape_hash {
        let skins_dir = crate::utils::paths::get_app_data_dir().join("skins");
        let cape_path = skins_dir.join(format!("{}.png", hash));
        if cape_path.exists() {
            let data = std::fs::read(&cape_path)
                .map_err(|e| CommandError::from(crate::error::AppError::IoError(e)))?;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
            Ok(Some(format!("data:image/png;base64,{}", b64)))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}
