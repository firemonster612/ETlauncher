use serde::{Deserialize, Serialize};

/// Represents an installed Java version managed by the launcher
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaInstallation {
    /// Major version (e.g., 8, 17, 21)
    pub major_version: u32,
    /// Path to java executable
    pub java_path: String,
    /// Installation timestamp
    pub installed_at: i64,
}

/// Manifest tracking all managed Java installations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JavaManifest {
    pub installations: Vec<JavaInstallation>,
}
