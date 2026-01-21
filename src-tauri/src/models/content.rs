use crate::models::instance::{LoaderType, ModpackPlatform};
use crate::models::loader::LoaderVersion;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Platform that hosts individual content (mods, shaders, resourcepacks)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ContentPlatform {
    Modrinth,
    CurseForge,
}

/// Source of how content was installed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum ContentSource {
    /// Content came from original modpack installation
    ModpackOriginal,
    /// User installed this content manually
    #[default]
    UserAdded,
    /// Installed as a dependency of user-added content
    UserDependency,
}

impl std::fmt::Display for ContentPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentPlatform::Modrinth => write!(f, "modrinth"),
            ContentPlatform::CurseForge => write!(f, "curseforge"),
        }
    }
}

/// Type of content
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Mod,
    Shader,
    ResourcePack,
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::Mod => write!(f, "mod"),
            ContentType::Shader => write!(f, "shader"),
            ContentType::ResourcePack => write!(f, "resourcepack"),
        }
    }
}

/// Sort order for content search
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ContentSortBy {
    #[default]
    Downloads,
    RecentlyUpdated,
    Name,
    Relevance,
}

/// Dependency type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    Required,
    Optional,
    Incompatible,
    Embedded,
}

/// A dependency of a content version
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentDependency {
    /// ID of the dependency on the same platform
    pub id: String,
    /// Name of the dependency (for display)
    pub name: Option<String>,
    /// Type of dependency
    pub dependency_type: DependencyType,
    /// Version requirement (if any)
    pub version_req: Option<String>,
}

/// Search parameters for content queries
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ContentSearchParams {
    /// Search query (by name)
    pub query: Option<String>,
    /// Filter by Minecraft version
    pub mc_version: Option<String>,
    /// Filter by mod loader
    pub loader: Option<LoaderType>,
    /// Filter by content type
    pub content_type: Option<ContentType>,
    /// Filter by category
    pub category: Option<String>,
    /// Sort order
    pub sort_by: Option<ContentSortBy>,
    /// Page number (0-indexed)
    pub page: Option<u32>,
    /// Number of results per page
    pub page_size: Option<u32>,
    /// Filter by platform (None = search all)
    pub platform: Option<ContentPlatform>,
}

/// A file within a content version
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentFile {
    /// Download URL
    pub url: String,
    /// File hash (SHA1 or SHA512)
    pub hash: Option<String>,
    /// Hash algorithm used
    pub hash_algorithm: Option<String>,
    /// File size in bytes
    pub size: u64,
    /// Filename
    pub filename: String,
    /// Whether this is the primary file
    #[serde(default)]
    pub primary: bool,
}

/// A specific version of content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentVersion {
    /// Version ID on the platform
    pub id: String,
    /// Project/content ID this version belongs to
    pub project_id: String,
    /// Version name/number
    pub name: String,
    /// Version number (semantic if available)
    pub version_number: String,
    /// Minecraft versions this is for
    pub mc_versions: Vec<String>,
    /// Mod loaders this is for
    pub loaders: Vec<LoaderType>,
    /// Release date (Unix timestamp)
    pub released_at: Option<i64>,
    /// Download count for this version
    pub downloads: Option<u64>,
    /// Files included in this version
    #[serde(default)]
    pub files: Vec<ContentFile>,
    /// Dependencies
    #[serde(default)]
    pub dependencies: Vec<ContentDependency>,
    /// Changelog
    pub changelog: Option<String>,
}

/// Image entry for a piece of content (e.g., Modrinth gallery)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentGalleryImage {
    /// Processed image URL
    pub url: String,
    /// Original image URL (if provided)
    pub raw_url: Option<String>,
    /// Optional title
    pub title: Option<String>,
    /// Optional description
    pub description: Option<String>,
    /// Whether the platform marks this as featured
    #[serde(default)]
    pub featured: bool,
}

/// Content item from any platform (mod, shader, resourcepack)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    /// Unique ID on the platform
    pub id: String,
    /// Slug/URL-friendly name
    pub slug: String,
    /// Display name
    pub name: String,
    /// Author name
    pub author: String,
    /// Short description
    pub description: String,
    /// Full description (may be markdown)
    pub body: Option<String>,
    /// Icon URL
    pub icon_url: Option<String>,
    /// Total download count
    pub downloads: u64,
    /// Platform this content is from
    pub platform: ContentPlatform,
    /// Type of content
    pub content_type: ContentType,
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
    pub latest_version: Option<ContentVersion>,
    /// External URL to content page
    pub url: Option<String>,
    /// Last updated timestamp
    pub updated_at: Option<i64>,
    /// Created timestamp
    pub created_at: Option<i64>,
}

/// Search results from a platform
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentSearchResult {
    /// List of content items
    pub items: Vec<Content>,
    /// Total number of results (for pagination)
    pub total_count: u64,
    /// Current page
    pub page: u32,
    /// Page size
    pub page_size: u32,
}

/// Installed content tracking for an instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledContent {
    /// Display name
    pub name: String,
    /// Slug for matching across platforms
    pub slug: String,
    /// Modrinth project ID (if known)
    pub modrinth_id: Option<String>,
    /// CurseForge project ID (if known)
    pub curseforge_id: Option<u32>,
    /// Platform the content was installed from
    pub installed_from: ContentPlatform,
    /// Installed version string
    pub version: String,
    /// Installed version ID
    pub version_id: String,
    /// Filename of the installed file
    pub filename: String,
    /// Content type
    pub content_type: ContentType,
    /// Install timestamp
    pub installed_at: i64,
    /// Whether this was installed as a dependency
    #[serde(default)]
    pub is_dependency: bool,
    /// Filenames of content this is a dependency of (parent mods)
    #[serde(default)]
    pub dependency_of: Vec<String>,
    /// IDs of content this mod depends on (for reverse lookup when dependencies are reinstalled)
    /// Format: "modrinth:ID" or "curseforge:ID"
    #[serde(default)]
    pub dependency_ids: Vec<String>,
    /// How this content was installed (modpack-original vs user-added)
    #[serde(default)]
    pub source: ContentSource,
    /// SHA512 hash for quick lookup (Modrinth)
    pub sha512_hash: Option<String>,
    /// Murmur2 fingerprint for CurseForge lookup
    pub murmur2_fingerprint: Option<u32>,
    /// Whether this content is linked from the resource pool
    #[serde(default)]
    pub is_pooled: bool,
}

/// Current manifest format version
pub const MANIFEST_VERSION: u32 = 1;

/// Manifest of all installed content in an instance
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InstalledContentManifest {
    /// Manifest format version for future migration support
    #[serde(default = "default_manifest_version")]
    pub manifest_version: u32,
    /// Installed mods
    #[serde(default)]
    pub mods: Vec<InstalledContent>,
    /// Installed shaders
    #[serde(default)]
    pub shaders: Vec<InstalledContent>,
    /// Installed resource packs
    #[serde(default)]
    pub resource_packs: Vec<InstalledContent>,
    /// Last sync timestamp (when manifest was reconciled with filesystem)
    pub last_synced_at: Option<i64>,
}

fn default_manifest_version() -> u32 {
    MANIFEST_VERSION
}

/// Request to install content to an instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallContentRequest {
    /// Instance to install to
    pub instance_id: String,
    /// Platform the content is from
    pub platform: ContentPlatform,
    /// Content ID on the platform
    pub content_id: String,
    /// Version ID to install
    pub version_id: String,
    /// Whether to also install dependencies
    #[serde(default = "default_true")]
    pub install_dependencies: bool,
}

fn default_true() -> bool {
    true
}

/// Progress of content installation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentInstallProgress {
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
}

/// Progress of a single file download (for real-time UI updates)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentDownloadProgress {
    /// Filename being downloaded
    pub filename: String,
    /// Bytes downloaded so far
    pub downloaded_bytes: u64,
    /// Total file size in bytes
    pub total_bytes: u64,
    /// Progress percentage (0-100)
    pub progress_percent: u8,
}

/// Progress with queue identification for parallel downloads
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentDownloadProgressWithId {
    /// Queue entry ID
    pub queue_id: String,
    /// Content ID for matching
    pub content_id: String,
    /// Filename being downloaded
    pub filename: String,
    /// Bytes downloaded so far
    pub downloaded_bytes: u64,
    /// Total file size in bytes
    pub total_bytes: u64,
    /// Progress percentage (0-100)
    pub progress_percent: u8,
}

/// Status of a queued content download
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum QueueItemStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
}

/// Request to queue a content installation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueInstallRequest {
    pub queue_id: String,
    pub instance_id: String,
    pub platform: ContentPlatform,
    pub content_id: String,
    pub content_name: String,
    pub content_slug: String,
    pub content_type: ContentType,
    pub version_id: String,
    pub version_name: String,
    pub mc_version: String,
    pub loader: Option<LoaderType>,
    /// Whether this is a dependency (auto-resolved) vs user-requested
    #[serde(default)]
    pub is_dependency: bool,
}

/// Queue status change event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueStatusEvent {
    pub queue_id: String,
    pub content_id: String,
    pub content_name: String,
    pub content_type: ContentType,
    pub status: QueueItemStatus,
    pub error: Option<String>,
}

/// Internal queued item for processing
#[derive(Debug, Clone)]
pub struct QueuedContentInstall {
    pub queue_id: String,
    pub instance_id: String,
    pub platform: ContentPlatform,
    pub content_id: String,
    pub content_name: String,
    pub content_slug: String,
    pub content_type: ContentType,
    pub version_id: String,
    pub version_name: String,
    pub mc_version: String,
    pub loader: Option<LoaderType>,
    /// Whether this is a dependency (auto-resolved) vs user-requested
    pub is_dependency: bool,
    /// If this is a dependency, the filename of the parent content that depends on this
    pub parent_filename: Option<String>,
}

/// Resolved dependency with install info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedDependency {
    /// The content item
    pub content: Content,
    /// The version to install
    pub version: ContentVersion,
    /// Whether this is already installed
    pub already_installed: bool,
}

/// Modrinth project info from hash lookup
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedModrinthProject {
    /// Project ID
    pub project_id: String,
    /// Project slug
    pub slug: String,
    /// Display name
    pub name: String,
    /// Version ID that was matched
    pub version_id: String,
    /// Version number string
    pub version_number: String,
}

/// CurseForge project info from fingerprint lookup
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedCurseForgeProject {
    /// Project ID
    pub project_id: u64,
    /// File ID
    pub file_id: u64,
    /// Display name
    pub name: String,
    /// Filename
    pub filename: String,
    /// Project slug for cross-platform matching
    pub slug: String,
}

/// A detected mod file from scanning the mods folder
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedMod {
    /// Filename of the mod
    pub filename: String,
    /// File size in bytes
    pub size: u64,
    /// SHA512 hash of the file
    pub sha512: String,
    /// Murmur2 fingerprint for CurseForge
    pub murmur2_fingerprint: u32,
    /// Modrinth project info (if identified)
    pub modrinth_project: Option<DetectedModrinthProject>,
    /// CurseForge project info (if identified)
    pub curseforge_project: Option<DetectedCurseForgeProject>,
    /// Whether this mod was identified
    pub is_identified: bool,
    /// Whether this item is disabled (in disabled subfolder)
    #[serde(default)]
    pub is_disabled: bool,
    /// Whether this was installed as a dependency
    #[serde(default)]
    pub is_dependency: bool,
    /// Filenames of content this is a dependency of (parent mods)
    #[serde(default)]
    pub dependency_of: Vec<String>,
}

/// Result of scanning an instance's content folder
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    /// Content folder was found
    pub folder_exists: bool,
    /// All detected content items
    pub items: Vec<DetectedMod>,
    /// Count of identified items (matched via Modrinth or CurseForge)
    pub identified_count: u32,
    /// Count of unidentified items
    pub unidentified_count: u32,
    /// Timestamp of scan
    pub scanned_at: i64,
}

/// Cached file hash entry for scan caching
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedFileHash {
    /// Filename
    pub filename: String,
    /// File size in bytes
    pub size: u64,
    /// File modification time (Unix timestamp)
    pub modified_time: i64,
    /// SHA512 hash
    pub sha512: String,
    /// Murmur2 fingerprint for CurseForge
    pub murmur2_fingerprint: u32,
}

/// Cache for scan results to avoid rehashing unchanged files
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScanCache {
    /// Map of filename -> cached hash info
    pub files: std::collections::HashMap<String, CachedFileHash>,
    /// Last full scan timestamp
    pub last_scan: i64,
}

// =============================================================================
// UPDATE CHECKING TYPES
// =============================================================================

/// Status of a content item during update check
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ContentUpdateStatus {
    /// Newer version available
    UpdateAvailable {
        current_version: String,
        available_version: String,
        available_version_id: String,
    },
    /// Already on latest version
    UpToDate,
    /// No compatible version for target MC version/loader
    NoCompatibleVersion,
    /// Could not identify content (unmanaged file)
    Unidentified,
    /// Content was removed from platform or API error
    Unavailable,
}

/// Single content item update info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentUpdateInfo {
    /// Filename of the content
    pub filename: String,
    /// Display name
    pub name: String,
    /// How this content was installed
    pub source: ContentSource,
    /// Platform (if identified)
    pub platform: Option<ContentPlatform>,
    /// Project ID on platform (if identified)
    pub project_id: Option<String>,
    /// Current installed version ID
    pub current_version_id: Option<String>,
    /// Update status
    pub status: ContentUpdateStatus,
}

/// Modpack-specific update information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackUpdateInfo {
    /// Platform the modpack is from
    pub platform: crate::models::instance::ModpackPlatform,
    /// Modpack ID on platform
    pub modpack_id: String,
    /// Currently installed version ID
    pub current_version_id: String,
    /// Currently installed version name
    pub current_version_name: String,
    /// Available newer version ID
    pub available_version_id: String,
    /// Available newer version name
    pub available_version_name: String,
    /// MC version of the available update
    pub available_mc_version: String,
    /// Changelog for the new version
    pub changelog: Option<String>,
}

/// Full update check result for an instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    /// Instance ID
    pub instance_id: String,
    /// Current Minecraft version
    pub current_mc_version: String,
    /// Current loader version
    pub current_loader_version: Option<String>,
    /// Target Minecraft version (same as current for content-only updates)
    pub target_mc_version: String,
    /// Target loader type
    pub target_loader: crate::models::instance::LoaderType,
    /// Target loader version
    pub target_loader_version: Option<String>,
    /// Available loader versions for target MC version
    pub available_loader_versions: Vec<LoaderVersion>,
    /// Content that can be updated
    pub updatable: Vec<ContentUpdateInfo>,
    /// Content already up to date
    pub up_to_date: Vec<ContentUpdateInfo>,
    /// Content with no compatible version for target
    pub incompatible: Vec<ContentUpdateInfo>,
    /// Unidentified content (user manages manually)
    pub unidentified: Vec<ContentUpdateInfo>,
    /// Modpack update info (if instance is from modpack)
    pub modpack_update: Option<ModpackUpdateInfo>,
}

/// User decision for content that cannot update
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum IncompatibleContentAction {
    /// Keep the old version (may cause compatibility issues)
    Keep,
    /// Remove the content
    Remove,
}

/// Update plan after user reviews and confirms
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlan {
    /// Instance ID
    pub instance_id: String,
    /// New Minecraft version (if changing)
    pub update_minecraft_version: Option<String>,
    /// New loader version (if changing)
    pub update_loader_version: Option<String>,
    /// Filenames of content to update
    pub content_to_update: Vec<String>,
    /// Filenames of content to remove
    pub content_to_remove: Vec<String>,
    /// Filenames of incompatible content user wants to keep
    pub content_to_keep: Vec<String>,
}

// =============================================================================
// NEW UPDATE SYSTEM TYPES
// =============================================================================

/// Available modpack version for update selection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackVersionOption {
    /// Version ID on the platform
    pub version_id: String,
    /// Version name/number
    pub version_name: String,
    /// Minecraft version for this modpack version
    pub mc_version: String,
    /// Mod loader type
    pub loader_type: LoaderType,
    /// Mod loader version
    pub loader_version: Option<String>,
    /// Release timestamp
    pub released_at: Option<i64>,
    /// Changelog for this version
    pub changelog: Option<String>,
    /// Whether this is the currently installed version
    pub is_current: bool,
}

/// Result of checking modpack instance for updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackInstanceUpdateCheck {
    /// Instance ID
    pub instance_id: String,
    /// Modpack display name
    pub modpack_name: String,
    /// Platform the modpack is from
    pub platform: ModpackPlatform,
    /// Modpack ID on the platform
    pub modpack_id: String,
    /// Currently installed version info
    pub current_version: ModpackVersionOption,
    /// All available versions (newest first)
    pub available_versions: Vec<ModpackVersionOption>,
    /// User-added content that may need decisions during update
    pub user_added_content: Vec<InstalledContent>,
    /// Whether a newer version is available
    pub has_update: bool,
}

/// User decision for content during update (no default allowed)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum UserContentDecision {
    /// User has not yet made a decision (must be resolved before update)
    Pending,
    /// Keep the content
    Keep,
    /// Remove the content
    Remove,
}

/// Plan for executing a modpack update
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackUpdatePlan {
    /// Instance ID
    pub instance_id: String,
    /// Target modpack version ID to update to
    pub target_version_id: String,
    /// Map of filename -> decision for user-added content
    pub user_content_decisions: HashMap<String, UserContentDecision>,
}

/// Result of checking non-modpack instance for updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceUpdateCheck {
    /// Instance ID
    pub instance_id: String,
    /// Current Minecraft version
    pub current_mc_version: String,
    /// Current mod loader type
    pub current_loader_type: LoaderType,
    /// Current mod loader version
    pub current_loader_version: Option<String>,
    /// Latest available Minecraft release version
    pub latest_mc_version: String,
    /// Whether a MC version update is available
    pub has_mc_update: bool,
    /// Target loader version for the new MC version
    pub target_loader_version: Option<String>,
    /// Content that can be updated to the new MC version
    pub compatible_content: Vec<ContentUpdateInfo>,
    /// Content that has no compatible version for the new MC version
    pub incompatible_content: Vec<ContentUpdateInfo>,
    /// Content that could not be identified
    pub unidentified_content: Vec<ContentUpdateInfo>,
}

/// Plan for executing a non-modpack instance update
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceUpdatePlan {
    /// Instance ID
    pub instance_id: String,
    /// Target Minecraft version
    pub target_mc_version: String,
    /// Target mod loader type
    pub target_loader_type: LoaderType,
    /// Target mod loader version
    pub target_loader_version: Option<String>,
    /// Map of filename -> decision for incompatible content
    pub incompatible_decisions: HashMap<String, UserContentDecision>,
}
