use parking_lot::RwLock;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::cache::Cache;
use crate::models::content::{
    Content, ContentSearchResult, ContentVersion, QueuedContentInstall, ResolvedDependency,
};
use crate::models::instance::{LoaderType, ModpackPlatform};
use crate::models::loader::LoaderVersion;
use crate::models::modpack::{Modpack, ModpackSearchResult, ModpackVersion};
use crate::models::resource_pool::ResourcePoolIndex;
use crate::models::{AppSettings, VersionManifest};
use crate::task_registry::TaskRegistry;

/// API cache for reducing redundant API calls.
/// All caches are thread-safe and automatically expire entries based on TTL.
pub struct ApiCache {
    /// Loader versions cache: (LoaderType, mc_version) -> versions
    /// TTL: 1 hour, max 100 entries
    pub loader_versions: Cache<(LoaderType, String), Vec<LoaderVersion>>,

    /// Content search results cache: hash of search params -> results
    /// TTL: 5 minutes, max 50 entries
    pub content_search: Cache<u64, ContentSearchResult>,

    /// Modpack search results cache: hash of search params -> results
    /// TTL: 5 minutes, max 50 entries
    pub modpack_search: Cache<u64, ModpackSearchResult>,

    /// Content details cache: content_id -> content
    /// TTL: 10 minutes, max 200 entries
    pub content_details: Cache<String, Content>,

    /// Modpack details cache: "platform:id" -> modpack
    /// TTL: 10 minutes, max 100 entries
    pub modpack_details: Cache<String, Modpack>,

    /// Modpack versions cache: "platform:id" -> versions
    /// TTL: 10 minutes, max 100 entries
    pub modpack_versions: Cache<String, Vec<ModpackVersion>>,

    /// Content versions cache: "platform:id:mc_version:loader" -> versions
    /// TTL: 10 minutes, max 100 entries
    pub content_versions: Cache<String, Vec<ContentVersion>>,

    /// Resolved dependencies cache: "version_id:instance_id" -> resolved deps
    /// TTL: 5 minutes, max 100 entries
    pub resolved_dependencies: Cache<String, Vec<ResolvedDependency>>,
}

impl ApiCache {
    /// Number of cache types in ApiCache
    pub const CACHE_COUNT: u32 = 8;

    /// Create a new API cache with default TTLs.
    pub fn new() -> Self {
        Self {
            // 1 hour TTL for loader versions
            loader_versions: Cache::new(Duration::from_secs(3600), 100),
            // 5 minute TTL for search results
            content_search: Cache::new(Duration::from_secs(300), 50),
            modpack_search: Cache::new(Duration::from_secs(300), 50),
            // 10 minute TTL for content/modpack details
            content_details: Cache::new(Duration::from_secs(600), 200),
            modpack_details: Cache::new(Duration::from_secs(600), 100),
            modpack_versions: Cache::new(Duration::from_secs(600), 100),
            // 10 minute TTL for content versions
            content_versions: Cache::new(Duration::from_secs(600), 100),
            // 5 minute TTL for resolved dependencies
            resolved_dependencies: Cache::new(Duration::from_secs(300), 100),
        }
    }

    /// Clear all caches.
    pub fn clear_all(&self) {
        self.loader_versions.clear();
        self.content_search.clear();
        self.modpack_search.clear();
        self.content_details.clear();
        self.modpack_details.clear();
        self.modpack_versions.clear();
        self.content_versions.clear();
        self.resolved_dependencies.clear();
    }

    /// Clear loader versions cache only.
    pub fn clear_loader_versions(&self) {
        self.loader_versions.clear();
    }

    /// Clear search caches only.
    pub fn clear_search_caches(&self) {
        self.content_search.clear();
        self.modpack_search.clear();
    }

    /// Clean up expired entries from all caches.
    pub fn cleanup_expired(&self) {
        self.loader_versions.cleanup_expired();
        self.content_search.cleanup_expired();
        self.modpack_search.cleanup_expired();
        self.content_details.cleanup_expired();
        self.modpack_details.cleanup_expired();
        self.modpack_versions.cleanup_expired();
        self.content_versions.cleanup_expired();
        self.resolved_dependencies.cleanup_expired();
    }
}

impl Default for ApiCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Global application state managed by Tauri
pub struct AppState {
    /// Application settings
    pub settings: RwLock<AppSettings>,
    /// Cached version manifest
    pub version_manifest: RwLock<Option<CachedManifest>>,
    /// Running game processes by instance ID
    pub running_instances: RwLock<HashMap<String, RunningInstance>>,
    /// HTTP client for API requests
    pub http_client: reqwest::Client,
    /// Queue of pending modpack installations
    pub modpack_install_queue: Arc<Mutex<VecDeque<QueuedModpackInstall>>>,
    /// Content download queue for parallel downloads
    pub content_download_queue: Arc<Mutex<VecDeque<QueuedContentInstall>>>,
    /// Currently active content downloads (queue IDs)
    pub active_content_downloads: Arc<Mutex<HashSet<String>>>,
    /// Cancellation tokens for active downloads (queue_id -> token)
    pub content_download_tokens: Arc<Mutex<HashMap<String, CancellationToken>>>,
    /// API cache for reducing redundant API calls
    pub api_cache: ApiCache,
    /// Resource pool index for shared content management
    pub resource_pool_index: RwLock<ResourcePoolIndex>,
    /// Centralized task registry for tracking background operations
    pub task_registry: TaskRegistry,
    /// Cancellation tokens for active launches (instance_id -> token)
    pub launch_tokens: RwLock<HashMap<String, CancellationToken>>,
}

/// Cached version manifest with fetch timestamp
pub struct CachedManifest {
    pub manifest: VersionManifest,
    pub fetched_at: i64,
}

/// Information about a running game instance
pub struct RunningInstance {
    pub instance_id: String,
    pub pid: u32,
    pub started_at: i64,
}

/// Queued modpack installation request
#[derive(Clone)]
pub struct QueuedModpackInstall {
    pub queue_id: String,
    pub platform: ModpackPlatform,
    pub modpack_id: String,
    pub version_id: String,
    pub instance_name: Option<String>,
    pub modpack_name: String,
}

impl AppState {
    /// Create a new AppState with default settings
    pub fn new() -> Self {
        let http_client =
            crate::utils::http::create_client().expect("Failed to create HTTP client");

        // Load the resource pool index if it exists
        let pool_index = crate::services::resource_pool_service::load_pool_index()
            .unwrap_or_else(|_| ResourcePoolIndex::new());

        Self {
            settings: RwLock::new(AppSettings::default()),
            version_manifest: RwLock::new(None),
            running_instances: RwLock::new(HashMap::new()),
            http_client,
            modpack_install_queue: Arc::new(Mutex::new(VecDeque::new())),
            content_download_queue: Arc::new(Mutex::new(VecDeque::new())),
            active_content_downloads: Arc::new(Mutex::new(HashSet::new())),
            content_download_tokens: Arc::new(Mutex::new(HashMap::new())),
            api_cache: ApiCache::new(),
            resource_pool_index: RwLock::new(pool_index),
            task_registry: TaskRegistry::new(),
            launch_tokens: RwLock::new(HashMap::new()),
        }
    }

    /// Load settings from disk
    pub fn load_settings(&self) -> Result<(), crate::error::AppError> {
        let config_path = crate::utils::paths::get_config_path();

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let mut settings: AppSettings = serde_json::from_str(&content)?;

            // Fix empty instances path
            if settings.instances_path.is_empty() {
                settings.instances_path = crate::utils::paths::get_instances_dir()
                    .to_string_lossy()
                    .to_string();
                // Save corrected settings
                let content = serde_json::to_string_pretty(&settings)?;
                std::fs::write(&config_path, content)?;
            }

            *self.settings.write() = settings;
        } else {
            // Create default settings with proper paths
            let settings = AppSettings {
                instances_path: crate::utils::paths::get_instances_dir()
                    .to_string_lossy()
                    .to_string(),
                ..Default::default()
            };
            *self.settings.write() = settings;
        }

        Ok(())
    }

    /// Save settings to disk
    pub fn save_settings(&self) -> Result<(), crate::error::AppError> {
        let config_path = crate::utils::paths::get_config_path();

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let settings = self.settings.read();
        let content = serde_json::to_string_pretty(&*settings)?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }

    /// Get a clone of current settings
    pub fn get_settings(&self) -> AppSettings {
        self.settings.read().clone()
    }

    /// Check if version manifest cache is valid
    pub fn is_manifest_cache_valid(&self) -> bool {
        let cache = self.version_manifest.read();
        if let Some(cached) = &*cache {
            let now = chrono::Utc::now().timestamp();
            // Cache valid for 5 minutes
            now - cached.fetched_at < 300
        } else {
            false
        }
    }

    /// Update the cached version manifest
    pub fn set_version_manifest(&self, manifest: VersionManifest) {
        let mut cache = self.version_manifest.write();
        *cache = Some(CachedManifest {
            manifest,
            fetched_at: chrono::Utc::now().timestamp(),
        });
    }

    /// Get cached version manifest if valid
    pub fn get_version_manifest(&self) -> Option<VersionManifest> {
        let cache = self.version_manifest.read();
        cache.as_ref().map(|c| c.manifest.clone())
    }

    /// Register a running instance
    pub fn register_running_instance(&self, instance_id: String, pid: u32) {
        let mut running = self.running_instances.write();
        running.insert(
            instance_id.clone(),
            RunningInstance {
                instance_id,
                pid,
                started_at: chrono::Utc::now().timestamp(),
            },
        );
    }

    /// Unregister a running instance
    pub fn unregister_running_instance(&self, instance_id: &str) {
        let mut running = self.running_instances.write();
        running.remove(instance_id);
    }

    /// Check if an instance is running
    pub fn is_instance_running(&self, instance_id: &str) -> bool {
        let running = self.running_instances.read();
        running.contains_key(instance_id)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
