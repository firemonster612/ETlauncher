use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccountType {
    #[serde(rename = "microsoft")]
    Microsoft,
    #[serde(rename = "offline")]
    Offline,
}

fn default_account_type() -> AccountType {
    AccountType::Microsoft
}

/// A Minecraft account authenticated via Microsoft
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftAccount {
    /// Unique identifier for this account entry
    pub id: String,
    /// Minecraft username (display name)
    pub username: String,
    /// Minecraft player UUID
    pub uuid: String,
    /// Whether this is the currently active account
    pub is_active: bool,
    /// URL to player's skin texture
    pub skin_url: Option<String>,
    /// URL to player's cape texture
    pub cape_url: Option<String>,
    /// Unix timestamp when account was added
    pub created_at: i64,
    /// Unix timestamp when account was last used
    pub last_used_at: i64,
    /// Unix timestamp when tokens expire
    pub token_expires_at: i64,
    /// Account type (Microsoft or Offline)
    #[serde(default = "default_account_type")]
    pub account_type: AccountType,
    /// Local skin file hash for offline accounts
    pub offline_skin_hash: Option<String>,
    /// Skin variant for offline accounts ("classic" or "slim")
    pub offline_skin_variant: Option<String>,
    /// Local cape file hash for offline accounts
    pub offline_cape_hash: Option<String>,
}

/// Authentication tokens from Microsoft/Xbox/Minecraft auth chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub microsoft_access_token: String,
    pub microsoft_refresh_token: String,
    pub xbox_token: String,
    pub xsts_token: String,
    pub user_hash: String,
    pub minecraft_access_token: String,
    pub expires_at: i64,
}

/// Response from Microsoft device code flow initiation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: i32,
    pub interval: i32,
}

/// Status of device code authentication polling
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum AuthPollStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "success")]
    Success { account: MinecraftAccount },
    #[serde(rename = "expired")]
    Expired,
    #[serde(rename = "error")]
    Error { message: String },
}

/// Minecraft profile response from API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
    pub skins: Vec<SkinInfo>,
    pub capes: Vec<CapeInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinInfo {
    pub id: String,
    pub state: String,
    pub url: String,
    pub variant: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapeInfo {
    pub id: String,
    pub state: String,
    pub url: String,
    pub alias: String,
}

/// Request to upload a new skin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadSkinRequest {
    /// "classic" or "slim"
    pub variant: String,
    /// Either a URL or base64 encoded image data
    pub skin_data: String,
    /// Whether skin_data is a URL (true) or file data (false)
    pub is_url: bool,
}

/// Account list stored in accounts.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountStore {
    pub accounts: Vec<MinecraftAccount>,
}
