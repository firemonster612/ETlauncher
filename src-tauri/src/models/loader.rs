use serde::{Deserialize, Serialize};

/// Information about a mod loader version
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderVersion {
    /// Version string (e.g., "0.18.4" for Fabric, "61.0.5" for Forge)
    pub version: String,
    /// Maven coordinates for downloading
    pub maven: String,
    /// Whether this version is marked as stable
    pub stable: bool,
    /// Build number (for some loaders)
    #[serde(default)]
    pub build: u32,
    /// Separator character used in version strings
    #[serde(default)]
    pub separator: String,
}

/// Request to install a mod loader
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderInstallRequest {
    /// Type of loader to install
    pub loader_type: String,
    /// Minecraft version
    pub minecraft_version: String,
    /// Loader version to install
    pub loader_version: String,
}

/// Progress of a loader installation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderInstallProgress {
    /// Current stage of installation
    pub stage: String,
    /// Progress percentage (0-100)
    pub progress: u32,
    /// Current file being downloaded/processed
    pub current_file: Option<String>,
    /// Total bytes to download (if applicable)
    pub total_bytes: Option<u64>,
    /// Bytes downloaded so far (if applicable)
    pub downloaded_bytes: Option<u64>,
}

/// Fabric/Quilt meta API response for loader versions (generic endpoint)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricLoaderMeta {
    pub separator: String,
    pub build: u32,
    pub maven: String,
    pub version: String,
    #[serde(default)]
    pub stable: Option<bool>,
}

/// Fabric/Quilt meta API response for MC-version-specific loader versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricLoaderForVersion {
    pub loader: FabricLoaderMeta,
}

/// Fabric meta API response for installer versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricInstallerMeta {
    pub url: String,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

/// Forge promotions API response
#[derive(Debug, Clone, Deserialize)]
pub struct ForgePromotions {
    pub homepage: String,
    pub promos: std::collections::HashMap<String, String>,
}

/// Forge Maven metadata response
#[derive(Debug, Clone, Deserialize)]
pub struct ForgeMavenMetadata {
    #[serde(rename = "versioning")]
    pub versioning: ForgeVersioning,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ForgeVersioning {
    #[serde(rename = "versions")]
    pub versions: ForgeVersionsList,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ForgeVersionsList {
    #[serde(rename = "version", default)]
    pub version: Vec<String>,
}

/// NeoForge Maven API response
#[derive(Debug, Clone, Deserialize)]
pub struct NeoForgeMavenResponse {
    pub versions: Vec<String>,
}

/// LiteLoader versions API response
#[derive(Debug, Clone, Deserialize)]
pub struct LiteLoaderVersionsResponse {
    pub meta: LiteLoaderMeta,
    pub versions: std::collections::HashMap<String, LiteLoaderMcVersion>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LiteLoaderMeta {
    pub description: String,
    pub authors: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LiteLoaderMcVersion {
    #[serde(default)]
    pub repo: Option<LiteLoaderRepo>,
    #[serde(default)]
    pub artefacts: Option<LiteLoaderArtefacts>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LiteLoaderRepo {
    pub stream: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LiteLoaderArtefacts {
    #[serde(rename = "com.mumfrey:liteloader", default)]
    pub liteloader: Option<std::collections::HashMap<String, LiteLoaderArtefact>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LiteLoaderArtefact {
    #[serde(rename = "tweakClass")]
    pub tweak_class: Option<String>,
    pub file: Option<String>,
    pub version: Option<String>,
    pub timestamp: Option<String>,
    pub md5: Option<String>,
}
