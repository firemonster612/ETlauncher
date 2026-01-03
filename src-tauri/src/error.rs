use serde::Serialize;
use thiserror::Error;

/// Application-wide error types
#[derive(Debug, Error)]
pub enum AppError {
    // Authentication errors
    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Device code expired")]
    DeviceCodeExpired,

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Token refresh failed: {0}")]
    TokenRefreshFailed(String),

    // Instance errors
    #[error("Instance not found: {0}")]
    InstanceNotFound(String),

    #[error("Instance already exists: {0}")]
    InstanceAlreadyExists(String),

    // Minecraft errors
    #[error("Version not found: {0}")]
    VersionNotFound(String),

    #[error("Download failed: {0}")]
    DownloadError(String),

    #[error("Hash verification failed for: {0}")]
    HashMismatch(String),

    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    // Launch errors
    #[error("Java not found")]
    JavaNotFound,

    #[error("Launch failed: {0}")]
    LaunchError(String),

    #[error("Game crashed: {0}")]
    GameCrashed(String),

    // Settings errors
    #[error("Settings error: {0}")]
    SettingsError(String),

    // General errors
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Zip error: {0}")]
    ZipError(#[from] zip::result::ZipError),

    #[error("Keyring error: {0}")]
    KeyringError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Serializable error for frontend
#[derive(Debug, Clone, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl From<AppError> for CommandError {
    fn from(err: AppError) -> Self {
        let code = match &err {
            AppError::AuthError(_) => "AUTH_ERROR",
            AppError::DeviceCodeExpired => "DEVICE_CODE_EXPIRED",
            AppError::AccountNotFound(_) => "ACCOUNT_NOT_FOUND",
            AppError::TokenRefreshFailed(_) => "TOKEN_REFRESH_FAILED",
            AppError::InstanceNotFound(_) => "INSTANCE_NOT_FOUND",
            AppError::InstanceAlreadyExists(_) => "INSTANCE_EXISTS",
            AppError::VersionNotFound(_) => "VERSION_NOT_FOUND",
            AppError::DownloadError(_) => "DOWNLOAD_ERROR",
            AppError::HashMismatch(_) => "HASH_MISMATCH",
            AppError::AssetNotFound(_) => "ASSET_NOT_FOUND",
            AppError::JavaNotFound => "JAVA_NOT_FOUND",
            AppError::LaunchError(_) => "LAUNCH_ERROR",
            AppError::GameCrashed(_) => "GAME_CRASHED",
            AppError::SettingsError(_) => "SETTINGS_ERROR",
            AppError::IoError(_) => "IO_ERROR",
            AppError::JsonError(_) => "JSON_ERROR",
            AppError::HttpError(_) => "HTTP_ERROR",
            AppError::ZipError(_) => "ZIP_ERROR",
            AppError::KeyringError(_) => "KEYRING_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
        };

        CommandError {
            code: code.to_string(),
            message: err.to_string(),
        }
    }
}

impl From<keyring::Error> for AppError {
    fn from(err: keyring::Error) -> Self {
        AppError::KeyringError(err.to_string())
    }
}

// Allow CommandError to be returned from Tauri commands
impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for CommandError {}
