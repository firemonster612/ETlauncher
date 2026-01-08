use serde::{Deserialize, Serialize};
use crate::models::content::ContentGalleryImage;
use crate::models::instance::{LoaderType, ModpackPlatform};

/// Sort order for modpack search
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModpackSortBy {
    #[default]
    Downloads,
    RecentlyUpdated,
    Name,
    Relevance,
}

/// Category of modpack content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModpackCategory {
    Adventure,
    Tech,
    Magic,
    Optimization,
    Kitchen,
    Quests,
    Exploration,
    Survival,
    Skyblock,
    Hardcore,
    Other,
}

impl std::fmt::Display for ModpackCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModpackCategory::Adventure => write!(f, "adventure"),
            ModpackCategory::Tech => write!(f, "tech"),
            ModpackCategory::Magic => write!(f, "magic"),
            ModpackCategory::Optimization => write!(f, "optimization"),
            ModpackCategory::Kitchen => write!(f, "kitchen"),
            ModpackCategory::Quests => write!(f, "quests"),
            ModpackCategory::Exploration => write!(f, "exploration"),
            ModpackCategory::Survival => write!(f, "survival"),
            ModpackCategory::Skyblock => write!(f, "skyblock"),
            ModpackCategory::Hardcore => write!(f, "hardcore"),
            ModpackCategory::Other => write!(f, "other"),
        }
    }
}

/// Search parameters for modpack queries
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModpackSearchParams {
    /// Search query (by name)
    pub query: Option<String>,
    /// Filter by Minecraft version
    pub mc_version: Option<String>,
    /// Filter by mod loader
    pub loader: Option<LoaderType>,
    /// Filter by category
    pub category: Option<String>,
    /// Sort order
    pub sort_by: Option<ModpackSortBy>,
    /// Page number (0-indexed)
    pub page: Option<u32>,
    /// Number of results per page
    pub page_size: Option<u32>,
    /// Filter by platform (None = search all)
    pub platform: Option<ModpackPlatform>,
}

/// A file within a modpack version
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackFile {
    /// Download URL
    pub url: String,
    /// File hash (SHA1 or SHA512)
    pub hash: Option<String>,
    /// Hash algorithm used
    pub hash_algorithm: Option<String>,
    /// File size in bytes
    pub size: u64,
    /// Relative path within the instance
    pub path: String,
    /// Whether this is a required file
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

/// A specific version of a modpack
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackVersion {
    /// Version ID on the platform
    pub id: String,
    /// Version name/number
    pub name: String,
    /// Minecraft version this is for
    pub mc_version: String,
    /// Mod loader type
    pub loader_type: LoaderType,
    /// Mod loader version
    pub loader_version: Option<String>,
    /// Changelog for this version
    pub changelog: Option<String>,
    /// Release date (Unix timestamp)
    pub released_at: Option<i64>,
    /// Download count for this version
    pub downloads: Option<u64>,
    /// Files included in this version
    #[serde(default)]
    pub files: Vec<ModpackFile>,
}

/// A modpack from any platform
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Modpack {
    /// Unique ID on the platform
    pub id: String,
    /// Slug/URL-friendly name
    pub slug: String,
    /// Display name
    pub name: String,
    /// Author/team name
    pub author: String,
    /// Short description
    pub description: String,
    /// Full description (may be markdown)
    pub body: Option<String>,
    /// Icon URL
    pub icon_url: Option<String>,
    /// Banner/cover image URL
    pub banner_url: Option<String>,
    /// Total download count
    pub downloads: u64,
    /// Platform this modpack is from
    pub platform: ModpackPlatform,
    /// Categories/tags
    #[serde(default)]
    pub categories: Vec<String>,
    /// Gallery images (if provided by the platform)
    #[serde(default)]
    pub gallery: Vec<ContentGalleryImage>,
    /// Available Minecraft versions
    #[serde(default)]
    pub mc_versions: Vec<String>,
    /// Available mod loaders
    #[serde(default)]
    pub loaders: Vec<LoaderType>,
    /// Latest version info (if available)
    pub latest_version: Option<ModpackVersion>,
    /// External URL to modpack page
    pub url: Option<String>,
    /// Last updated timestamp
    pub updated_at: Option<i64>,
    /// Created timestamp
    pub created_at: Option<i64>,
}

/// Search results from a platform
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackSearchResult {
    /// List of modpacks
    pub modpacks: Vec<Modpack>,
    /// Total number of results (for pagination)
    pub total_count: u64,
    /// Current page
    pub page: u32,
    /// Page size
    pub page_size: u32,
}

/// Progress of modpack installation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackInstallProgress {
    /// Current stage of installation
    pub stage: String,
    /// Progress percentage (0-100)
    pub progress: u32,
    /// Current file being downloaded
    pub current_file: Option<String>,
    /// Total files to download
    pub total_files: u32,
    /// Files downloaded so far
    pub completed_files: u32,
    /// Total bytes to download
    pub total_bytes: Option<u64>,
    /// Bytes downloaded so far
    pub downloaded_bytes: Option<u64>,
}

/// A mod entry within a modpack (best-effort)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackMod {
    /// Platform-specific ID (Modrinth project ID or CurseForge mod ID)
    pub id: String,
    /// Display name if resolvable
    pub name: String,
    /// Optional icon URL
    pub icon_url: Option<String>,
    /// Optional author
    pub author: Option<String>,
    /// Optional external URL
    pub url: Option<String>,
}

/// Request to install a modpack
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallModpackRequest {
    /// Platform the modpack is from
    pub platform: ModpackPlatform,
    /// Modpack ID on the platform
    pub modpack_id: String,
    /// Version ID to install
    pub version_id: String,
    /// Custom name for the instance (optional)
    pub instance_name: Option<String>,
}
