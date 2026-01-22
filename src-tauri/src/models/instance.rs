use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Type of mod loader for an instance
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum LoaderType {
    #[default]
    Vanilla,
    Forge,
    NeoForge,
    Fabric,
    Quilt,
    LiteLoader,
    /// Unknown loader - used when platform doesn't provide loader info
    Unknown,
}

impl FromStr for LoaderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "vanilla" => Ok(LoaderType::Vanilla),
            "forge" => Ok(LoaderType::Forge),
            "neoforge" => Ok(LoaderType::NeoForge),
            "fabric" => Ok(LoaderType::Fabric),
            "quilt" => Ok(LoaderType::Quilt),
            "liteloader" => Ok(LoaderType::LiteLoader),
            "unknown" => Ok(LoaderType::Unknown),
            _ => Err(format!("Unknown loader type: {}", s)),
        }
    }
}

impl std::fmt::Display for LoaderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoaderType::Vanilla => write!(f, "vanilla"),
            LoaderType::Forge => write!(f, "forge"),
            LoaderType::NeoForge => write!(f, "neoforge"),
            LoaderType::Fabric => write!(f, "fabric"),
            LoaderType::Quilt => write!(f, "quilt"),
            LoaderType::LiteLoader => write!(f, "liteloader"),
            LoaderType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Platform that hosts modpacks
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ModpackPlatform {
    Modrinth,
    CurseForge,
    FTB,
    Technic,
    ATLauncher,
}

impl std::fmt::Display for ModpackPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModpackPlatform::Modrinth => write!(f, "modrinth"),
            ModpackPlatform::CurseForge => write!(f, "curseforge"),
            ModpackPlatform::FTB => write!(f, "ftb"),
            ModpackPlatform::Technic => write!(f, "technic"),
            ModpackPlatform::ATLauncher => write!(f, "atlauncher"),
        }
    }
}

/// A Minecraft instance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    /// Unique identifier (UUID)
    pub id: String,
    /// Display name
    pub name: String,
    /// Minecraft version (e.g., "1.21.4")
    pub minecraft_version: String,
    /// Mod loader type (if any)
    pub loader_type: LoaderType,
    /// Mod loader version (if applicable)
    pub loader_version: Option<String>,
    /// Unix timestamp when instance was created
    pub created_at: i64,
    /// Unix timestamp when instance was last played
    pub last_played_at: Option<i64>,
    /// Total play time in seconds
    pub total_play_time: u64,
    /// Path to custom icon (relative to instance folder)
    pub icon_path: Option<String>,
    /// Override: path to Java executable
    pub java_path: Option<String>,
    /// Override: minimum memory allocation (MB)
    pub memory_min_mb: Option<u32>,
    /// Override: maximum memory allocation (MB)
    pub memory_max_mb: Option<u32>,
    /// Override: additional JVM arguments
    pub jvm_args: Option<String>,
    /// Override: additional game arguments
    pub game_args: Option<String>,
    /// Override: game window width
    pub resolution_width: Option<u32>,
    /// Override: game window height
    pub resolution_height: Option<u32>,
    /// Modpack platform (if created from a modpack)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modpack_platform: Option<ModpackPlatform>,
    /// Modpack ID on the platform
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modpack_id: Option<String>,
    /// Installed modpack version ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modpack_version_id: Option<String>,
}

impl Default for Instance {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            minecraft_version: String::new(),
            loader_type: LoaderType::Vanilla,
            loader_version: None,
            created_at: 0,
            last_played_at: None,
            total_play_time: 0,
            icon_path: None,
            java_path: None,
            memory_min_mb: None,
            memory_max_mb: None,
            jvm_args: None,
            game_args: None,
            resolution_width: None,
            resolution_height: None,
            modpack_platform: None,
            modpack_id: None,
            modpack_version_id: None,
        }
    }
}

/// Request to create a new instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstanceRequest {
    pub name: String,
    pub minecraft_version: String,
    pub loader_type: Option<LoaderType>,
    pub loader_version: Option<String>,
}

/// Request to update an existing instance
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstanceRequest {
    pub name: Option<String>,
    pub loader_type: Option<LoaderType>,
    pub loader_version: Option<String>,
    pub icon_path: Option<String>,
    pub java_path: Option<String>,
    pub memory_min_mb: Option<u32>,
    pub memory_max_mb: Option<u32>,
    pub jvm_args: Option<String>,
    pub game_args: Option<String>,
    pub resolution_width: Option<u32>,
    pub resolution_height: Option<u32>,
}

/// Instance launch status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum LaunchStatus {
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "checkingAccount")]
    CheckingAccount,
    #[serde(rename = "refreshingToken")]
    RefreshingToken,
    #[serde(rename = "loadingVersion")]
    LoadingVersion,
    #[serde(rename = "verifyingFiles")]
    VerifyingFiles { checked: u32, total: u32 },
    #[serde(rename = "downloading")]
    Downloading { progress: DownloadProgress },
    #[serde(rename = "checkingJava")]
    CheckingJava { version: u32 },
    #[serde(rename = "downloadingJava")]
    DownloadingJava { version: u32, progress: u32 },
    #[serde(rename = "buildingClasspath")]
    BuildingClasspath,
    #[serde(rename = "launching")]
    Launching,
    #[serde(rename = "running")]
    Running { pid: u32 },
    #[serde(rename = "windowReady")]
    WindowReady { pid: u32 },
    #[serde(rename = "stopped")]
    Stopped { exit_code: i32 },
    #[serde(rename = "crashed")]
    Crashed { message: String },
}

/// Instance setup status (used during instance creation)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum InstanceSetupStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "preparing")]
    Preparing { message: String },
    #[serde(rename = "downloadingGameFiles")]
    DownloadingGameFiles { progress: DownloadProgress },
    #[serde(rename = "installingLoader")]
    InstallingLoader { stage: String, progress: u32 },
    #[serde(rename = "complete")]
    Complete,
    #[serde(rename = "failed")]
    Failed { message: String },
}

/// Download progress information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub total_files: u32,
    pub completed_files: u32,
    #[serde(default)]
    pub current_file: String,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub speed_bytes_per_sec: u64,
}
