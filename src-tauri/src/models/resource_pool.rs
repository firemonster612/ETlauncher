//! Resource pool models for shared content management
//!
//! This module defines data structures for the resource pool system that stores
//! mods, shaders, and resource packs once and links them to instances.

use crate::models::content::ContentType;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Strategy for linking resources from the pool to instances
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LinkStrategy {
    /// Automatically choose the best strategy (hard link > symlink > copy)
    #[default]
    Auto,
    /// Always use hard links (fails if cross-filesystem)
    HardLink,
    /// Always use symbolic links
    Symlink,
    /// Always copy files (no space savings, but always works)
    Copy,
}

impl std::fmt::Display for LinkStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkStrategy::Auto => write!(f, "auto"),
            LinkStrategy::HardLink => write!(f, "hardlink"),
            LinkStrategy::Symlink => write!(f, "symlink"),
            LinkStrategy::Copy => write!(f, "copy"),
        }
    }
}

/// Configuration for the resource pool
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcePoolConfig {
    /// Whether the resource pool is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Preferred link strategy
    #[serde(default)]
    pub link_strategy: LinkStrategy,
}

fn default_true() -> bool {
    true
}

impl Default for ResourcePoolConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            link_strategy: LinkStrategy::Auto,
        }
    }
}

/// A resource stored in the pool
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PooledResource {
    /// SHA512 hash of the file (used as filename in pool)
    pub sha512: String,
    /// Original filename (for display and extension detection)
    pub original_filename: String,
    /// Content type (mod, shader, resourcepack)
    pub content_type: ContentType,
    /// File size in bytes
    pub size: u64,
    /// When the resource was added to the pool (Unix timestamp)
    pub added_at: i64,
    /// Set of instance IDs that reference this resource
    /// Format: "instance_id:filename" to allow same resource with different names
    #[serde(default)]
    pub used_by: HashSet<String>,
}

impl PooledResource {
    /// Create a new pooled resource
    pub fn new(
        sha512: String,
        original_filename: String,
        content_type: ContentType,
        size: u64,
    ) -> Self {
        Self {
            sha512,
            original_filename,
            content_type,
            size,
            added_at: chrono::Utc::now().timestamp(),
            used_by: HashSet::new(),
        }
    }

    /// Add an instance usage reference
    pub fn add_usage(&mut self, instance_id: &str, filename: &str) {
        self.used_by.insert(format!("{}:{}", instance_id, filename));
    }

    /// Remove an instance usage reference
    pub fn remove_usage(&mut self, instance_id: &str, filename: &str) {
        self.used_by
            .remove(&format!("{}:{}", instance_id, filename));
    }

    /// Remove all usages for an instance (when instance is deleted)
    pub fn remove_instance_usages(&mut self, instance_id: &str) {
        self.used_by
            .retain(|usage| !usage.starts_with(&format!("{}:", instance_id)));
    }

    /// Check if this resource is unused
    pub fn is_unused(&self) -> bool {
        self.used_by.is_empty()
    }

    /// Get the number of instances using this resource
    pub fn usage_count(&self) -> usize {
        self.used_by.len()
    }
}

/// Index of all resources in the pool
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResourcePoolIndex {
    /// Version of the index format for future migrations
    #[serde(default = "default_index_version")]
    pub version: u32,
    /// Map of SHA512 hash -> pooled resource
    #[serde(default)]
    pub resources: HashMap<String, PooledResource>,
    /// Last garbage collection timestamp
    pub last_gc_at: Option<i64>,
}

fn default_index_version() -> u32 {
    1
}

impl ResourcePoolIndex {
    /// Create a new empty index
    pub fn new() -> Self {
        Self {
            version: 1,
            resources: HashMap::new(),
            last_gc_at: None,
        }
    }

    /// Get a resource by hash
    pub fn get(&self, sha512: &str) -> Option<&PooledResource> {
        self.resources.get(sha512)
    }

    /// Get a mutable resource by hash
    pub fn get_mut(&mut self, sha512: &str) -> Option<&mut PooledResource> {
        self.resources.get_mut(sha512)
    }

    /// Check if a resource exists in the pool
    pub fn contains(&self, sha512: &str) -> bool {
        self.resources.contains_key(sha512)
    }

    /// Add a resource to the index
    pub fn add(&mut self, resource: PooledResource) {
        self.resources.insert(resource.sha512.clone(), resource);
    }

    /// Remove a resource from the index
    pub fn remove(&mut self, sha512: &str) -> Option<PooledResource> {
        self.resources.remove(sha512)
    }

    /// Get all unused resources eligible for garbage collection
    /// A resource is eligible if it's unused and older than the min_age
    pub fn get_gc_candidates(&self, min_age_secs: i64) -> Vec<&PooledResource> {
        let now = chrono::Utc::now().timestamp();
        self.resources
            .values()
            .filter(|r| r.is_unused() && (now - r.added_at) > min_age_secs)
            .collect()
    }

    /// Get total pool size in bytes
    pub fn total_size(&self) -> u64 {
        self.resources.values().map(|r| r.size).sum()
    }

    /// Get count of resources by type
    pub fn count_by_type(&self, content_type: ContentType) -> usize {
        self.resources
            .values()
            .filter(|r| r.content_type == content_type)
            .count()
    }

    /// Calculate space savings (sum of sizes * (usage_count - 1) for multi-use resources)
    pub fn calculate_space_savings(&self) -> u64 {
        self.resources
            .values()
            .filter(|r| r.usage_count() > 1)
            .map(|r| r.size * (r.usage_count() as u64 - 1))
            .sum()
    }
}

/// Statistics about the resource pool
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcePoolStats {
    /// Total number of resources in the pool
    pub total_resources: usize,
    /// Number of mods in the pool
    pub mod_count: usize,
    /// Number of shaders in the pool
    pub shader_count: usize,
    /// Number of resource packs in the pool
    pub resource_pack_count: usize,
    /// Total size of the pool in bytes
    pub total_size_bytes: u64,
    /// Estimated space saved by deduplication (bytes)
    pub space_saved_bytes: u64,
    /// Number of unused resources (candidates for GC)
    pub unused_count: usize,
    /// Last garbage collection timestamp
    pub last_gc_at: Option<i64>,
    /// Size of cached assets (textures, sounds, etc.) in bytes
    pub assets_cache_size: u64,
    /// Size of cached libraries (Java libraries) in bytes
    pub libraries_cache_size: u64,
}

impl ResourcePoolStats {
    /// Create stats from a pool index (without cache sizes - use from_index_with_cache_sizes instead)
    pub fn from_index(index: &ResourcePoolIndex) -> Self {
        Self {
            total_resources: index.resources.len(),
            mod_count: index.count_by_type(ContentType::Mod),
            shader_count: index.count_by_type(ContentType::Shader),
            resource_pack_count: index.count_by_type(ContentType::ResourcePack),
            total_size_bytes: index.total_size(),
            space_saved_bytes: index.calculate_space_savings(),
            unused_count: index.resources.values().filter(|r| r.is_unused()).count(),
            last_gc_at: index.last_gc_at,
            assets_cache_size: 0,
            libraries_cache_size: 0,
        }
    }

    /// Create stats from a pool index with cache sizes
    pub fn from_index_with_cache_sizes(
        index: &ResourcePoolIndex,
        assets_cache_size: u64,
        libraries_cache_size: u64,
    ) -> Self {
        Self {
            total_resources: index.resources.len(),
            mod_count: index.count_by_type(ContentType::Mod),
            shader_count: index.count_by_type(ContentType::Shader),
            resource_pack_count: index.count_by_type(ContentType::ResourcePack),
            total_size_bytes: index.total_size(),
            space_saved_bytes: index.calculate_space_savings(),
            unused_count: index.resources.values().filter(|r| r.is_unused()).count(),
            last_gc_at: index.last_gc_at,
            assets_cache_size,
            libraries_cache_size,
        }
    }
}

/// Result of a link operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// The strategy that was actually used
    pub strategy_used: LinkStrategy,
    /// Error message if failed
    pub error: Option<String>,
}

/// Result of garbage collection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GarbageCollectionResult {
    /// Number of resources removed
    pub resources_removed: usize,
    /// Bytes freed
    pub bytes_freed: u64,
    /// Resources that failed to remove
    pub failed: Vec<String>,
}
