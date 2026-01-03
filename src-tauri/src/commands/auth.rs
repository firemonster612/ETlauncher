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
