use crate::models::resource_pool::ResourcePoolConfig;
use crate::utils::paths;
use serde::{Deserialize, Serialize};

pub use crate::models::resource_pool::LinkStrategy;

/// Application theme
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Dark,
    Light,
    System,
}

/// Global application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// Path to instances directory
    pub instances_path: String,
    /// Minimum memory allocation (MB)
    pub memory_min_mb: u32,
    /// Maximum memory allocation (MB)
    pub memory_max_mb: u32,
    /// Number of concurrent downloads
    pub concurrent_downloads: u32,
    /// Close launcher when game starts
    pub close_launcher_on_game_start: bool,
    /// Reopen launcher when game closes
    pub reopen_launcher_on_game_close: bool,
    /// Show snapshot versions in version list
    pub show_snapshots: bool,
    /// Show old_alpha and old_beta versions
    pub show_old_versions: bool,
    /// UI theme
    pub theme: Theme,
    /// CurseForge API key for accessing CurseForge content
    #[serde(default)]
    pub curseforge_api_key: Option<String>,
    /// Whether the first-launch setup/tutorial has been completed
    #[serde(default)]
    pub setup_completed: bool,
    /// Resource pool configuration for shared content management
    #[serde(default)]
    pub resource_pool: ResourcePoolConfig,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            instances_path: paths::get_instances_dir().to_string_lossy().to_string(),
            memory_min_mb: 512,
            memory_max_mb: 4096,
            concurrent_downloads: 4,
            close_launcher_on_game_start: false,
            reopen_launcher_on_game_close: true,
            show_snapshots: false,
            show_old_versions: false,
            theme: Theme::Dark,
            curseforge_api_key: None,
            setup_completed: false,
            resource_pool: ResourcePoolConfig::default(),
        }
    }
}

/// Request to update settings (partial update)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsRequest {
    pub instances_path: Option<String>,
    pub memory_min_mb: Option<u32>,
    pub memory_max_mb: Option<u32>,
    pub concurrent_downloads: Option<u32>,
    pub close_launcher_on_game_start: Option<bool>,
    pub reopen_launcher_on_game_close: Option<bool>,
    pub show_snapshots: Option<bool>,
    pub show_old_versions: Option<bool>,
    pub theme: Option<Theme>,
    pub curseforge_api_key: Option<String>,
    pub setup_completed: Option<bool>,
    pub resource_pool: Option<ResourcePoolConfig>,
}
