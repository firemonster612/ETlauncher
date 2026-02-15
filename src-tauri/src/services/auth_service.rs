use crate::error::AppError;
use crate::models::{AuthPollStatus, DeviceCodeResponse, MinecraftAccount, MinecraftProfile};
use reqwest::Client;
use serde::{Deserialize, Serialize};

// Microsoft OAuth endpoints
const MS_DEVICE_CODE_URL: &str =
    "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const MS_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";

// Xbox Live endpoints
const XBOX_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";

// Minecraft endpoints
const MC_AUTH_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MC_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";
const MC_SKINS_URL: &str = "https://api.minecraftservices.com/minecraft/profile/skins";
const MC_CAPES_URL: &str = "https://api.minecraftservices.com/minecraft/profile/capes/active";

// ETlauncher Microsoft OAuth Client ID
const CLIENT_ID: &str = "1fe9961c-2553-4cab-8286-1279defbfe1e";

/// Start the device code authentication flow
pub async fn start_device_code_flow(client: &Client) -> Result<DeviceCodeResponse, AppError> {
    #[derive(Deserialize)]
    struct MsDeviceCodeResponse {
        device_code: String,
        user_code: String,
        verification_uri: String,
        expires_in: i32,
        interval: i32,
    }

    let response = client
        .post(MS_DEVICE_CODE_URL)
        .form(&[
            ("client_id", CLIENT_ID),
            ("scope", "XboxLive.signin offline_access"),
        ])
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::AuthError(format!(
            "Failed to start device code flow: {}",
            error_text
        )));
    }

    let ms_response: MsDeviceCodeResponse = response.json().await?;

    Ok(DeviceCodeResponse {
        device_code: ms_response.device_code,
        user_code: ms_response.user_code,
        verification_uri: ms_response.verification_uri,
        expires_in: ms_response.expires_in,
        interval: ms_response.interval,
    })
}

/// Poll for device code authentication completion
pub async fn poll_device_code(
    client: &Client,
    device_code: &str,
) -> Result<AuthPollStatus, AppError> {
    #[derive(Deserialize)]
    struct MsTokenResponse {
        access_token: String,
        refresh_token: String,
        expires_in: i64,
    }

    #[derive(Deserialize)]
    struct MsErrorResponse {
        error: String,
        error_description: Option<String>,
    }

    let response = client
        .post(MS_TOKEN_URL)
        .form(&[
            ("client_id", CLIENT_ID),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("device_code", device_code),
        ])
        .send()
        .await?;

    if response.status().is_success() {
        let token_response: MsTokenResponse = response.json().await?;

        // Continue with Xbox Live authentication
        match complete_authentication(
            client,
            &token_response.access_token,
            &token_response.refresh_token,
            token_response.expires_in,
        )
        .await
        {
            Ok(account) => Ok(AuthPollStatus::Success { account }),
            Err(e) => Ok(AuthPollStatus::Error {
                message: e.to_string(),
            }),
        }
    } else {
        let error_response: MsErrorResponse = response.json().await?;

        match error_response.error.as_str() {
            "authorization_pending" => Ok(AuthPollStatus::Pending),
            "expired_token" => Ok(AuthPollStatus::Expired),
            "authorization_declined" => Ok(AuthPollStatus::Error {
                message: "Authorization was declined".to_string(),
            }),
            _ => Ok(AuthPollStatus::Error {
                message: error_response
                    .error_description
                    .unwrap_or(error_response.error),
            }),
        }
    }
}

/// Complete the full authentication chain: MS -> Xbox Live -> XSTS -> Minecraft
async fn complete_authentication(
    client: &Client,
    ms_access_token: &str,
    ms_refresh_token: &str,
    _expires_in: i64,
) -> Result<MinecraftAccount, AppError> {
    // Step 1: Authenticate with Xbox Live
    let (xbox_token, user_hash) = authenticate_xbox_live(client, ms_access_token).await?;

    // Step 2: Get XSTS token
    let xsts_token = authenticate_xsts(client, &xbox_token).await?;

    // Step 3: Authenticate with Minecraft
    let (mc_access_token, mc_expires_in) =
        authenticate_minecraft(client, &xsts_token, &user_hash).await?;

    // Step 4: Get Minecraft profile
    let profile = get_minecraft_profile(client, &mc_access_token).await?;

    // Store tokens in keyring
    let account_id = uuid::Uuid::new_v4().to_string();
    store_tokens(&account_id, ms_refresh_token, &mc_access_token)?;

    let now = chrono::Utc::now().timestamp();

    Ok(MinecraftAccount {
        id: account_id,
        username: profile.name,
        uuid: profile.id,
        is_active: true,
        skin_url: profile.skins.first().map(|s| s.url.clone()),
        cape_url: profile
            .capes
            .iter()
            .find(|c| c.state == "ACTIVE")
            .map(|c| c.url.clone()),
        created_at: now,
        last_used_at: now,
        token_expires_at: now + mc_expires_in,
        account_type: crate::models::account::AccountType::Microsoft,
        offline_skin_hash: None,
        offline_skin_variant: None,
        offline_cape_hash: None,
    })
}

/// Authenticate with Xbox Live
async fn authenticate_xbox_live(
    client: &Client,
    ms_access_token: &str,
) -> Result<(String, String), AppError> {
    #[derive(Serialize)]
    #[serde(rename_all = "PascalCase")]
    struct XboxAuthRequest {
        properties: XboxAuthProperties,
        relying_party: String,
        token_type: String,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "PascalCase")]
    struct XboxAuthProperties {
        auth_method: String,
        site_name: String,
        rps_ticket: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct XboxAuthResponse {
        token: String,
        display_claims: DisplayClaims,
    }

    #[derive(Deserialize)]
    struct DisplayClaims {
        xui: Vec<XuiClaim>,
    }

    #[derive(Deserialize)]
    struct XuiClaim {
        uhs: String,
    }

    let request = XboxAuthRequest {
        properties: XboxAuthProperties {
            auth_method: "RPS".to_string(),
            site_name: "user.auth.xboxlive.com".to_string(),
            rps_ticket: format!("d={}", ms_access_token),
        },
        relying_party: "http://auth.xboxlive.com".to_string(),
        token_type: "JWT".to_string(),
    };

    let response = client
        .post(XBOX_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::AuthError(format!(
            "Xbox Live authentication failed: {}",
            error_text
        )));
    }

    let xbox_response: XboxAuthResponse = response.json().await?;
    let user_hash = xbox_response
        .display_claims
        .xui
        .first()
        .ok_or_else(|| AppError::AuthError("No user hash in Xbox response".to_string()))?
        .uhs
        .clone();

    Ok((xbox_response.token, user_hash))
}

/// Authenticate with XSTS
async fn authenticate_xsts(client: &Client, xbox_token: &str) -> Result<String, AppError> {
    #[derive(Serialize)]
    #[serde(rename_all = "PascalCase")]
    struct XstsAuthRequest {
        properties: XstsAuthProperties,
        relying_party: String,
        token_type: String,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "PascalCase")]
    struct XstsAuthProperties {
        sandbox_id: String,
        user_tokens: Vec<String>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct XstsAuthResponse {
        token: String,
    }

    let request = XstsAuthRequest {
        properties: XstsAuthProperties {
            sandbox_id: "RETAIL".to_string(),
            user_tokens: vec![xbox_token.to_string()],
        },
        relying_party: "rp://api.minecraftservices.com/".to_string(),
        token_type: "JWT".to_string(),
    };

    let response = client
        .post(XSTS_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();

        // Check for specific XSTS errors
        if status.as_u16() == 401 {
            if error_text.contains("2148916233") {
                return Err(AppError::AuthError(
                    "This Microsoft account doesn't have an Xbox account. Please create one at xbox.com".to_string()
                ));
            } else if error_text.contains("2148916238") {
                return Err(AppError::AuthError(
                    "This account is a child account. A parent must add this account to a Microsoft family".to_string()
                ));
            }
        }

        return Err(AppError::AuthError(format!(
            "XSTS authentication failed: {}",
            error_text
        )));
    }

    let xsts_response: XstsAuthResponse = response.json().await?;
    Ok(xsts_response.token)
}

/// Authenticate with Minecraft services
async fn authenticate_minecraft(
    client: &Client,
    xsts_token: &str,
    user_hash: &str,
) -> Result<(String, i64), AppError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct McAuthRequest {
        identity_token: String,
    }

    #[derive(Deserialize)]
    struct McAuthResponse {
        access_token: String,
        expires_in: i64,
    }

    let request = McAuthRequest {
        identity_token: format!("XBL3.0 x={};{}", user_hash, xsts_token),
    };

    let response = client
        .post(MC_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::AuthError(format!(
            "Minecraft authentication failed: {}",
            error_text
        )));
    }

    let mc_response: McAuthResponse = response.json().await?;
    Ok((mc_response.access_token, mc_response.expires_in))
}

/// Get Minecraft profile (username, UUID, skins, capes)
pub async fn get_minecraft_profile(
    client: &Client,
    access_token: &str,
) -> Result<MinecraftProfile, AppError> {
    let response = client
        .get(MC_PROFILE_URL)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::AuthError(format!(
            "Failed to get Minecraft profile: {}",
            error_text
        )));
    }

    let profile: MinecraftProfile = response.json().await?;
    Ok(profile)
}

/// Refresh Microsoft token and get new Minecraft token
pub async fn refresh_tokens(client: &Client, account_id: &str) -> Result<(String, i64), AppError> {
    // Get refresh token from keyring
    let refresh_token = get_refresh_token(account_id)?;

    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct MsTokenResponse {
        access_token: String,
        refresh_token: String,
        expires_in: i64,
    }

    let response = client
        .post(MS_TOKEN_URL)
        .form(&[
            ("client_id", CLIENT_ID),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.as_str()),
        ])
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::TokenRefreshFailed(error_text));
    }

    let token_response: MsTokenResponse = response.json().await?;

    // Complete the auth chain again
    let (xbox_token, user_hash) =
        authenticate_xbox_live(client, &token_response.access_token).await?;
    let xsts_token = authenticate_xsts(client, &xbox_token).await?;
    let (mc_access_token, mc_expires_in) =
        authenticate_minecraft(client, &xsts_token, &user_hash).await?;

    // Update stored tokens
    store_tokens(account_id, &token_response.refresh_token, &mc_access_token)?;

    Ok((mc_access_token, mc_expires_in))
}

/// Upload a new skin
pub async fn upload_skin(
    client: &Client,
    access_token: &str,
    variant: &str,
    skin_data: &[u8],
) -> Result<(), AppError> {
    let form = reqwest::multipart::Form::new()
        .text("variant", variant.to_string())
        .part(
            "file",
            reqwest::multipart::Part::bytes(skin_data.to_vec())
                .file_name("skin.png")
                .mime_str("image/png")
                .unwrap(),
        );

    let response = client
        .post(MC_SKINS_URL)
        .header("Authorization", format!("Bearer {}", access_token))
        .multipart(form)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::AuthError(format!(
            "Failed to upload skin: {}",
            error_text
        )));
    }

    Ok(())
}

/// Set active cape
pub async fn set_cape(client: &Client, access_token: &str, cape_id: &str) -> Result<(), AppError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct SetCapeRequest {
        cape_id: String,
    }

    let response = client
        .put(MC_CAPES_URL)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(&SetCapeRequest {
            cape_id: cape_id.to_string(),
        })
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::AuthError(format!(
            "Failed to set cape: {}",
            error_text
        )));
    }

    Ok(())
}

/// Hide cape (remove active cape)
pub async fn hide_cape(client: &Client, access_token: &str) -> Result<(), AppError> {
    let response = client
        .delete(MC_CAPES_URL)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AppError::AuthError(format!(
            "Failed to hide cape: {}",
            error_text
        )));
    }

    Ok(())
}

// Token storage operations
// Uses the OS keyring/credential store for secure token storage:
// - macOS: Keychain
// - Windows: Credential Manager
// - Linux: Secret Service (GNOME Keyring / KDE Wallet)
//
// When the OS keyring is unavailable (e.g. no Secret Service on a minimal
// Linux compositor like niri), tokens fall back to a plaintext JSON file.
// This still works but is insecure — the user is warned via a banner.

use crate::utils::paths::get_app_data_dir;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

const KEYRING_SERVICE: &str = "etlauncher";

/// Whether the OS keyring is available. Set once at startup by `check_keyring_available()`.
static KEYRING_AVAILABLE: AtomicBool = AtomicBool::new(true);

/// Check whether the OS keyring is functional by attempting a write/read/delete cycle.
/// Must be called once at startup before any token operations.
pub fn check_keyring_available() {
    let probe_result = (|| -> Result<(), keyring::Error> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, "etlauncher:probe")?;
        entry.set_password("probe")?;
        let _ = entry.get_password()?;
        entry.delete_credential()?;
        Ok(())
    })();

    match probe_result {
        Ok(()) => {
            KEYRING_AVAILABLE.store(true, Ordering::Relaxed);
            eprintln!("[keyring] OS keyring is available.");
        }
        Err(e) => {
            KEYRING_AVAILABLE.store(false, Ordering::Relaxed);
            eprintln!(
                "[keyring] OS keyring is NOT available ({}). \
                 Tokens will be stored in a plaintext file (insecure). \
                 Install a Secret Service provider (e.g. gnome-keyring-daemon) for secure storage.",
                e
            );
        }
    }
}

/// Returns whether the OS keyring is available for secure persistent storage.
pub fn is_keyring_available() -> bool {
    KEYRING_AVAILABLE.load(Ordering::Relaxed)
}

// --- Keyring helpers ---

/// Get a keyring entry for a specific account token
fn keyring_entry(account_id: &str, token_type: &str) -> Result<keyring::Entry, AppError> {
    let user = format!("{}:{}", account_id, token_type);
    keyring::Entry::new(KEYRING_SERVICE, &user).map_err(|e| {
        AppError::KeyringError(format!(
            "Failed to create keyring entry for {}: {}",
            token_type, e
        ))
    })
}

// --- Plaintext file fallback (insecure) ---

use std::path::PathBuf;

fn get_tokens_file_path() -> PathBuf {
    get_app_data_dir().join("tokens.json")
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct FileTokenStore {
    tokens: HashMap<String, FileStoredTokens>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct FileStoredTokens {
    refresh_token: String,
    access_token: String,
}

fn load_file_token_store() -> FileTokenStore {
    let path = get_tokens_file_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(store) = serde_json::from_str(&content) {
                return store;
            }
        }
    }
    FileTokenStore::default()
}

fn save_file_token_store(store: &FileTokenStore) -> Result<(), AppError> {
    let path = get_tokens_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

// --- Token operations (keyring with file fallback) ---

fn store_tokens(
    account_id: &str,
    refresh_token: &str,
    mc_access_token: &str,
) -> Result<(), AppError> {
    if KEYRING_AVAILABLE.load(Ordering::Relaxed) {
        let refresh_result = keyring_entry(account_id, "refresh").and_then(|e| {
            e.set_password(refresh_token).map_err(|e| {
                AppError::KeyringError(format!("Failed to store refresh token: {}", e))
            })
        });

        let access_result = keyring_entry(account_id, "access").and_then(|e| {
            e.set_password(mc_access_token)
                .map_err(|e| AppError::KeyringError(format!("Failed to store access token: {}", e)))
        });

        if refresh_result.is_ok() && access_result.is_ok() {
            eprintln!("Tokens stored in OS keyring for account: {}", account_id);
            return Ok(());
        }

        // Keyring failed unexpectedly — fall through to file
        eprintln!(
            "[keyring] Keyring write failed, falling back to plaintext file for account: {}",
            account_id
        );
        KEYRING_AVAILABLE.store(false, Ordering::Relaxed);
    }

    // Plaintext file fallback
    let mut store = load_file_token_store();
    store.tokens.insert(
        account_id.to_string(),
        FileStoredTokens {
            refresh_token: refresh_token.to_string(),
            access_token: mc_access_token.to_string(),
        },
    );
    save_file_token_store(&store)?;
    eprintln!(
        "Tokens stored in plaintext file (insecure) for account: {}",
        account_id
    );
    Ok(())
}

fn get_refresh_token(account_id: &str) -> Result<String, AppError> {
    if KEYRING_AVAILABLE.load(Ordering::Relaxed) {
        if let Ok(token) = keyring_entry(account_id, "refresh").and_then(|e| {
            e.get_password().map_err(|e| match e {
                keyring::Error::NoEntry => {
                    AppError::KeyringError("No refresh token found".to_string())
                }
                other => AppError::KeyringError(format!("Failed to get refresh token: {}", other)),
            })
        }) {
            return Ok(token);
        }
    }

    // File fallback
    let store = load_file_token_store();
    store
        .tokens
        .get(account_id)
        .map(|t| t.refresh_token.clone())
        .ok_or_else(|| AppError::KeyringError("No refresh token found".to_string()))
}

pub fn get_access_token(account_id: &str) -> Result<String, AppError> {
    if KEYRING_AVAILABLE.load(Ordering::Relaxed) {
        if let Ok(token) = keyring_entry(account_id, "access").and_then(|e| {
            e.get_password().map_err(|e| match e {
                keyring::Error::NoEntry => {
                    AppError::KeyringError("No access token found".to_string())
                }
                other => AppError::KeyringError(format!("Failed to get access token: {}", other)),
            })
        }) {
            return Ok(token);
        }
    }

    // File fallback
    let store = load_file_token_store();
    store
        .tokens
        .get(account_id)
        .map(|t| t.access_token.clone())
        .ok_or_else(|| AppError::KeyringError("No access token found".to_string()))
}

pub fn delete_tokens(account_id: &str) -> Result<(), AppError> {
    // Delete from keyring if available
    if KEYRING_AVAILABLE.load(Ordering::Relaxed) {
        for token_type in &["refresh", "access"] {
            if let Ok(entry) = keyring_entry(account_id, token_type) {
                match entry.delete_credential() {
                    Ok(()) | Err(keyring::Error::NoEntry) => {}
                    Err(e) => {
                        eprintln!(
                            "[keyring] Warning: Failed to delete {} token from keyring: {}",
                            token_type, e
                        );
                    }
                }
            }
        }
    }

    // Always remove from file fallback too
    let mut store = load_file_token_store();
    if store.tokens.remove(account_id).is_some() {
        save_file_token_store(&store)?;
    }

    Ok(())
}

// --- Migration: plaintext file -> keyring ---

/// Migrate tokens from the plaintext tokens.json file into the OS keyring.
/// On startup, if the keyring is available and tokens.json exists, each token
/// is moved into the keyring and the file is deleted.
/// If the keyring is NOT available, tokens.json is left in place and used directly.
pub fn migrate_tokens_to_keyring() {
    if !is_keyring_available() {
        // No keyring — file will be used as-is, nothing to migrate
        return;
    }

    let tokens_path = get_tokens_file_path();
    if !tokens_path.exists() {
        return;
    }

    let content = match std::fs::read_to_string(&tokens_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let store: FileTokenStore = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(_) => return,
    };

    if store.tokens.is_empty() {
        let _ = std::fs::remove_file(&tokens_path);
        return;
    }

    eprintln!(
        "Migrating {} account(s) from tokens.json to OS keyring...",
        store.tokens.len()
    );

    let mut all_migrated = true;
    for (account_id, tokens) in &store.tokens {
        // Write directly to keyring (not through store_tokens, to avoid file fallback)
        let refresh_ok = keyring_entry(account_id, "refresh")
            .and_then(|e| {
                e.set_password(&tokens.refresh_token)
                    .map_err(|e| AppError::KeyringError(e.to_string()))
            })
            .is_ok();

        let access_ok = keyring_entry(account_id, "access")
            .and_then(|e| {
                e.set_password(&tokens.access_token)
                    .map_err(|e| AppError::KeyringError(e.to_string()))
            })
            .is_ok();

        if !refresh_ok || !access_ok {
            eprintln!(
                "Warning: Failed to migrate tokens for account {}",
                account_id
            );
            all_migrated = false;
        }
    }

    if all_migrated {
        if let Err(e) = std::fs::remove_file(&tokens_path) {
            eprintln!("Warning: Could not remove old tokens.json: {}", e);
        } else {
            eprintln!("Migration complete — tokens.json removed.");
        }
    } else {
        eprintln!("Warning: Some tokens could not be migrated. tokens.json kept as fallback.");
    }
}
