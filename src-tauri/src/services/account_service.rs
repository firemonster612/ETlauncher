use crate::error::AppError;
use crate::models::{AccountStore, MinecraftAccount};
use crate::services::auth_service;
use crate::utils::paths;

/// Load accounts from disk
pub fn load_accounts() -> Result<Vec<MinecraftAccount>, AppError> {
    let accounts_path = paths::get_accounts_path();

    if accounts_path.exists() {
        let content = std::fs::read_to_string(&accounts_path)?;
        let store: AccountStore = serde_json::from_str(&content)?;
        Ok(store.accounts)
    } else {
        Ok(Vec::new())
    }
}

/// Save accounts to disk
pub fn save_accounts(accounts: &[MinecraftAccount]) -> Result<(), AppError> {
    let accounts_path = paths::get_accounts_path();

    // Ensure parent directory exists
    if let Some(parent) = accounts_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let store = AccountStore {
        accounts: accounts.to_vec(),
    };

    let content = serde_json::to_string_pretty(&store)?;
    std::fs::write(&accounts_path, content)?;

    Ok(())
}

/// Add a new account
pub fn add_account(account: MinecraftAccount) -> Result<Vec<MinecraftAccount>, AppError> {
    let mut accounts = load_accounts()?;

    // Check if account already exists (by UUID)
    if accounts.iter().any(|a| a.uuid == account.uuid) {
        // Update existing account instead
        accounts.retain(|a| a.uuid != account.uuid);
    }

    // If this is the first account or it's marked as active, set it as active
    let should_be_active = account.is_active || accounts.is_empty();

    // Deactivate other accounts if this one should be active
    if should_be_active {
        for a in &mut accounts {
            a.is_active = false;
        }
    }

    let mut new_account = account;
    new_account.is_active = should_be_active;
    accounts.push(new_account);

    save_accounts(&accounts)?;
    Ok(accounts)
}

/// Get all accounts
pub fn get_accounts() -> Result<Vec<MinecraftAccount>, AppError> {
    load_accounts()
}

/// Get account by ID
pub fn get_account(account_id: &str) -> Result<MinecraftAccount, AppError> {
    let accounts = load_accounts()?;
    accounts
        .into_iter()
        .find(|a| a.id == account_id)
        .ok_or_else(|| AppError::AccountNotFound(account_id.to_string()))
}

/// Get the active account
pub fn get_active_account() -> Result<Option<MinecraftAccount>, AppError> {
    let accounts = load_accounts()?;
    Ok(accounts.into_iter().find(|a| a.is_active))
}

/// Set account as active
pub fn set_active_account(account_id: &str) -> Result<Vec<MinecraftAccount>, AppError> {
    let mut accounts = load_accounts()?;

    let account_exists = accounts.iter().any(|a| a.id == account_id);
    if !account_exists {
        return Err(AppError::AccountNotFound(account_id.to_string()));
    }

    for account in &mut accounts {
        account.is_active = account.id == account_id;
        if account.is_active {
            account.last_used_at = chrono::Utc::now().timestamp();
        }
    }

    save_accounts(&accounts)?;
    Ok(accounts)
}

/// Delete an account
pub fn delete_account(account_id: &str) -> Result<Vec<MinecraftAccount>, AppError> {
    let mut accounts = load_accounts()?;

    let was_active = accounts
        .iter()
        .find(|a| a.id == account_id)
        .map(|a| a.is_active)
        .unwrap_or(false);

    accounts.retain(|a| a.id != account_id);

    // Delete tokens from keyring
    auth_service::delete_tokens(account_id)?;

    // If we deleted the active account, make the first remaining account active
    if was_active && !accounts.is_empty() {
        accounts[0].is_active = true;
    }

    save_accounts(&accounts)?;
    Ok(accounts)
}

/// Update account token expiry
pub fn update_account_expiry(account_id: &str, expires_at: i64) -> Result<(), AppError> {
    let mut accounts = load_accounts()?;

    if let Some(account) = accounts.iter_mut().find(|a| a.id == account_id) {
        account.token_expires_at = expires_at;
        account.last_used_at = chrono::Utc::now().timestamp();
    }

    save_accounts(&accounts)?;
    Ok(())
}

/// Update account profile info (username, skin, cape)
pub fn update_account_profile(
    account_id: &str,
    username: String,
    skin_url: Option<String>,
    cape_url: Option<String>,
) -> Result<MinecraftAccount, AppError> {
    let mut accounts = load_accounts()?;

    let account = accounts
        .iter_mut()
        .find(|a| a.id == account_id)
        .ok_or_else(|| AppError::AccountNotFound(account_id.to_string()))?;

    account.username = username;
    account.skin_url = skin_url;
    account.cape_url = cape_url;

    let updated = account.clone();

    save_accounts(&accounts)?;
    Ok(updated)
}

/// Check if token needs refresh (within 5 minutes of expiry)
pub fn needs_token_refresh(account: &MinecraftAccount) -> bool {
    let now = chrono::Utc::now().timestamp();
    account.token_expires_at <= now + 300 // 5 minute buffer
}

/// Get valid access token, refreshing if necessary
pub async fn get_valid_access_token(
    client: &reqwest::Client,
    account_id: &str,
) -> Result<String, AppError> {
    let account = get_account(account_id)?;

    if needs_token_refresh(&account) {
        // Refresh the token
        let (new_token, expires_in) = auth_service::refresh_tokens(client, account_id).await?;
        let now = chrono::Utc::now().timestamp();
        update_account_expiry(account_id, now + expires_in)?;
        Ok(new_token)
    } else {
        // Use existing token
        auth_service::get_access_token(account_id)
    }
}
