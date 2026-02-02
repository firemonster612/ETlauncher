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

/// Color preset for accent color
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ColorPreset {
    #[default]
    Default,
    Purple,
    Green,
    Orange,
    Pink,
    Blue,
    Custom,
}

/// Custom theme colors
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ThemeColors {
    /// Primary hue (0-360)
    pub primary_hue: Option<f64>,
    /// Primary chroma (0-0.4)
    pub primary_chroma: Option<f64>,
}

/// Font family preference
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FontFamily {
    #[default]
    Pixel,
    System,
    Custom,
}

/// Custom font configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CustomFont {
    /// Font family name
    pub family: String,
}

/// Sidebar/titlebar style preset
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SidebarStyle {
    #[default]
    Default,
    Accent,
    Custom,
}

/// Custom sidebar color configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CustomSidebarColor {
    /// Sidebar hue (0-360)
    pub hue: f64,
    /// Sidebar chroma (0-0.35)
    pub chroma: f64,
}

/// Background type for app customization
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BackgroundType {
    #[default]
    None,
    Color,
    Image,
    Video,
    Gif,
}

/// Background configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundConfig {
    /// Type of background
    #[serde(rename = "type")]
    pub bg_type: BackgroundType,
    /// Hex color (for type='color')
    #[serde(default)]
    pub color: Option<String>,
    /// Stored filename in app data (for media types)
    #[serde(default)]
    pub filename: Option<String>,
    /// Opacity 0-1 (for media types)
    #[serde(default)]
    pub opacity: Option<f64>,
    /// Blur 0-20px (for media types)
    #[serde(default)]
    pub blur: Option<f64>,
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
    /// Color preset for accent color
    #[serde(default)]
    pub color_preset: Option<ColorPreset>,
    /// Custom theme colors (used when color_preset is 'custom')
    #[serde(default)]
    pub custom_colors: Option<ThemeColors>,
    /// Disable hover lift effect on interactive elements
    #[serde(default)]
    pub disable_hover_lift: Option<bool>,
    /// Font family preference
    #[serde(default)]
    pub font_family: Option<FontFamily>,
    /// Custom font configuration (used when font_family is 'custom')
    #[serde(default)]
    pub custom_font: Option<CustomFont>,
    /// Sidebar/titlebar style
    #[serde(default)]
    pub sidebar_style: Option<SidebarStyle>,
    /// Custom sidebar color (used when sidebar_style is 'custom')
    #[serde(default)]
    pub custom_sidebar_color: Option<CustomSidebarColor>,
    /// CurseForge API key for accessing CurseForge content
    #[serde(default)]
    pub curseforge_api_key: Option<String>,
    /// Whether the first-launch setup/tutorial has been completed
    #[serde(default)]
    pub setup_completed: bool,
    /// Resource pool configuration for shared content management
    #[serde(default)]
    pub resource_pool: ResourcePoolConfig,
    /// Whether auto-updates are enabled (default: true)
    #[serde(default = "default_true")]
    pub auto_update: bool,
    /// Whether to include pre-release versions in updates
    #[serde(default)]
    pub include_pre_releases: bool,
    /// Background customization configuration
    #[serde(default)]
    pub background: Option<BackgroundConfig>,
}

/// Default value helper for serde
fn default_true() -> bool {
    true
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
            color_preset: None,
            custom_colors: None,
            disable_hover_lift: None,
            font_family: None,
            custom_font: None,
            sidebar_style: None,
            custom_sidebar_color: None,
            curseforge_api_key: None,
            setup_completed: false,
            resource_pool: ResourcePoolConfig::default(),
            auto_update: true,
            include_pre_releases: false,
            background: None,
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
    pub color_preset: Option<ColorPreset>,
    pub custom_colors: Option<ThemeColors>,
    pub disable_hover_lift: Option<bool>,
    pub font_family: Option<FontFamily>,
    pub custom_font: Option<CustomFont>,
    pub sidebar_style: Option<SidebarStyle>,
    pub custom_sidebar_color: Option<CustomSidebarColor>,
    pub curseforge_api_key: Option<String>,
    pub setup_completed: Option<bool>,
    pub resource_pool: Option<ResourcePoolConfig>,
    pub auto_update: Option<bool>,
    pub include_pre_releases: Option<bool>,
    pub background: Option<BackgroundConfig>,
}
