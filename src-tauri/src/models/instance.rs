use serde::{Deserialize, Serialize};

/// Type of mod loader for an instance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LoaderType {
    #[default]
    Vanilla,
    Forge,
    NeoForge,
    Fabric,
    Quilt,
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
    #[serde(rename = "preparing")]
    Preparing { message: String },
    #[serde(rename = "downloading")]
    Downloading { progress: DownloadProgress },
    #[serde(rename = "launching")]
    Launching,
    #[serde(rename = "running")]
    Running { pid: u32 },
    #[serde(rename = "stopped")]
    Stopped { exit_code: i32 },
    #[serde(rename = "crashed")]
    Crashed { message: String },
}

/// Download progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub total_files: u32,
    pub completed_files: u32,
    pub current_file: String,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub speed_bytes_per_sec: u64,
}

impl Default for DownloadProgress {
    fn default() -> Self {
        Self {
            total_files: 0,
            completed_files: 0,
            current_file: String::new(),
            total_bytes: 0,
            downloaded_bytes: 0,
            speed_bytes_per_sec: 0,
        }
    }
}
