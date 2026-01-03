use parking_lot::RwLock;
use std::collections::HashMap;

use crate::models::{AppSettings, VersionManifest};

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

impl AppState {
    /// Create a new AppState with default settings
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .user_agent("ETLauncher/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            settings: RwLock::new(AppSettings::default()),
            version_manifest: RwLock::new(None),
            running_instances: RwLock::new(HashMap::new()),
            http_client,
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
            let mut settings = AppSettings::default();
            settings.instances_path = crate::utils::paths::get_instances_dir()
                .to_string_lossy()
                .to_string();
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
