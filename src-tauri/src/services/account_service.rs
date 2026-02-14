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

    let is_offline = accounts
        .iter()
        .find(|a| a.id == account_id)
        .map(|a| a.account_type == crate::models::account::AccountType::Offline)
        .unwrap_or(false);
    accounts.retain(|a| a.id != account_id);
    if !is_offline {
        auth_service::delete_tokens(account_id)?;
    }

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

    // Offline accounts don't need token refresh
    if account.account_type == crate::models::account::AccountType::Offline {
        return Ok("0".to_string());
    }

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

/// Create a new offline account with a given username
pub fn create_offline_account(username: &str) -> Result<MinecraftAccount, AppError> {
    // Require at least one Microsoft account to prevent piracy
    let accounts = load_accounts()?;
    let has_microsoft = accounts
        .iter()
        .any(|a| a.account_type == crate::models::account::AccountType::Microsoft);
    if !has_microsoft {
        return Err(AppError::OfflineAccountError(
            "You must have at least one Microsoft account logged in to create offline accounts"
                .to_string(),
        ));
    }

    // Validate username (3-16 chars, alphanumeric + underscore)
    if username.len() < 3 || username.len() > 16 {
        return Err(AppError::OfflineAccountError(
            "Username must be between 3 and 16 characters".to_string(),
        ));
    }
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(AppError::OfflineAccountError(
            "Username can only contain letters, numbers, and underscores".to_string(),
        ));
    }

    // Generate deterministic UUID using MD5("OfflinePlayer:" + username)
    let uuid = generate_offline_uuid(username);
    let now = chrono::Utc::now().timestamp();

    let account = MinecraftAccount {
        id: uuid::Uuid::new_v4().to_string(),
        username: username.to_string(),
        uuid,
        is_active: false,
        skin_url: None,
        cape_url: None,
        created_at: now,
        last_used_at: now,
        token_expires_at: i64::MAX,
        account_type: crate::models::account::AccountType::Offline,
        offline_skin_hash: None,
        offline_skin_variant: None,
        offline_cape_hash: None,
    };

    add_account(account.clone())?;

    // Set Steve as the default skin for offline accounts
    const STEVE_SKIN: &[u8] = include_bytes!("../../assets/steve.png");
    let _ = set_offline_skin(&account.id, STEVE_SKIN, "classic");

    // Return the account with correct is_active state
    get_account(&account.id)
}

/// Generate a deterministic UUID for an offline player (same as Minecraft's algorithm)
fn generate_offline_uuid(username: &str) -> String {
    let input = format!("OfflinePlayer:{}", username);
    let digest = md5::compute(input.as_bytes());
    let mut bytes = digest.0;

    // Set version to 3 (MD5-based UUID)
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    // Set variant to RFC 4122
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

/// Set skin for an offline account
/// Takes a PNG file path, copies it to the skins directory with SHA-1 hash name
pub fn set_offline_skin(
    account_id: &str,
    skin_data: &[u8],
    variant: &str,
) -> Result<String, AppError> {
    let account = get_account(account_id)?;
    if account.account_type != crate::models::account::AccountType::Offline {
        return Err(AppError::OfflineAccountError(
            "Can only set offline skin for offline accounts".to_string(),
        ));
    }

    // Compute SHA-1 hash of the skin data
    use sha1::Digest;
    let mut hasher = sha1::Sha1::new();
    hasher.update(skin_data);
    let hash = format!("{:x}", hasher.finalize());

    // Save to skins directory
    let skins_dir = crate::utils::paths::get_app_data_dir().join("skins");
    std::fs::create_dir_all(&skins_dir)?;
    let skin_path = skins_dir.join(format!("{}.png", hash));
    std::fs::write(&skin_path, skin_data)?;

    // Update account
    let mut accounts = load_accounts()?;
    if let Some(acc) = accounts.iter_mut().find(|a| a.id == account_id) {
        acc.offline_skin_hash = Some(hash.clone());
        acc.offline_skin_variant = Some(variant.to_string());
    }
    save_accounts(&accounts)?;

    Ok(hash)
}

/// Set cape for an offline account
pub fn set_offline_cape(account_id: &str, cape_data: &[u8]) -> Result<String, AppError> {
    let account = get_account(account_id)?;
    if account.account_type != crate::models::account::AccountType::Offline {
        return Err(AppError::OfflineAccountError(
            "Can only set offline cape for offline accounts".to_string(),
        ));
    }

    // Compute SHA-1 hash of the cape data
    use sha1::Digest;
    let mut hasher = sha1::Sha1::new();
    hasher.update(cape_data);
    let hash = format!("{:x}", hasher.finalize());

    // Save to skins directory (capes go here too)
    let skins_dir = crate::utils::paths::get_app_data_dir().join("skins");
    std::fs::create_dir_all(&skins_dir)?;
    let cape_path = skins_dir.join(format!("{}.png", hash));
    std::fs::write(&cape_path, cape_data)?;

    // Update account
    let mut accounts = load_accounts()?;
    if let Some(acc) = accounts.iter_mut().find(|a| a.id == account_id) {
        acc.offline_cape_hash = Some(hash.clone());
    }
    save_accounts(&accounts)?;

    Ok(hash)
}

/// Remove skin from offline account
pub fn remove_offline_skin(account_id: &str) -> Result<(), AppError> {
    let mut accounts = load_accounts()?;
    if let Some(acc) = accounts.iter_mut().find(|a| a.id == account_id) {
        acc.offline_skin_hash = None;
        acc.offline_skin_variant = None;
    }
    save_accounts(&accounts)?;
    Ok(())
}

/// Remove cape from offline account
pub fn remove_offline_cape(account_id: &str) -> Result<(), AppError> {
    let mut accounts = load_accounts()?;
    if let Some(acc) = accounts.iter_mut().find(|a| a.id == account_id) {
        acc.offline_cape_hash = None;
    }
    save_accounts(&accounts)?;
    Ok(())
}
