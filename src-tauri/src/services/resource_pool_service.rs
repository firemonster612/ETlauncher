//! Resource pool service for shared content management
//!
//! This service manages a central pool of mods, shaders, and resource packs,
//! allowing multiple instances to share the same files through hard links or symlinks.

use crate::app_error;
use crate::app_info;
use crate::error::AppError;
use crate::models::resource_pool::{
    GarbageCollectionResult, LinkResult, LinkStrategy, PooledResource, ResourcePoolIndex,
    ResourcePoolStats,
};
use crate::models::ContentType;
use crate::state::AppState;
use crate::utils::link::{link_with_fallback, same_filesystem, symlinks_available};
use crate::utils::paths::{
    get_pool_index_path, get_pooled_resource_path, get_resource_pool_dir,
    get_resource_pool_dir_for_type,
};
use std::fs;
use std::path::{Path, PathBuf};

/// Minimum age (in seconds) for a resource to be eligible for garbage collection
/// This prevents race conditions where a resource is added but not yet linked
const GC_MIN_AGE_SECS: i64 = 86400; // 24 hours

/// Minimum time (in seconds) between automatic garbage collection runs
const GC_INTERVAL_SECS: i64 = 86400; // 24 hours

/// Initialize the resource pool directories
pub fn init_pool() -> Result<(), AppError> {
    fs::create_dir_all(get_resource_pool_dir_for_type(&ContentType::Mod))?;
    fs::create_dir_all(get_resource_pool_dir_for_type(&ContentType::Shader))?;
    fs::create_dir_all(get_resource_pool_dir_for_type(&ContentType::ResourcePack))?;
    Ok(())
}

/// Load the resource pool index from disk
pub fn load_pool_index() -> Result<ResourcePoolIndex, AppError> {
    let index_path = get_pool_index_path();

    if !index_path.exists() {
        return Ok(ResourcePoolIndex::new());
    }

    let content = fs::read_to_string(&index_path)?;
    let index: ResourcePoolIndex = serde_json::from_str(&content)?;
    Ok(index)
}

/// Save the resource pool index to disk
pub fn save_pool_index(index: &ResourcePoolIndex) -> Result<(), AppError> {
    let index_path = get_pool_index_path();

    // Ensure parent directory exists
    if let Some(parent) = index_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(index)?;
    fs::write(&index_path, content)?;
    Ok(())
}

/// Add a resource to the pool from a source file
///
/// If the resource already exists (by hash), updates the usage tracking.
/// Returns the pool path where the resource is stored.
pub fn add_resource(
    state: &AppState,
    source_path: &Path,
    sha512: &str,
    content_type: ContentType,
    original_filename: &str,
) -> Result<PathBuf, AppError> {
    let mut index = state.resource_pool_index.write();

    let pool_path = get_pooled_resource_path(&content_type, sha512, original_filename);

    // If the resource already exists in the pool, just return the path
    if pool_path.exists() && index.contains(sha512) {
        return Ok(pool_path);
    }

    // Ensure pool directory exists
    if let Some(parent) = pool_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Get file size
    let metadata = fs::metadata(source_path)?;
    let size = metadata.len();

    // Move or copy the file to the pool
    // Try to rename first (atomic, fast), fall back to copy + delete
    if fs::rename(source_path, &pool_path).is_err() {
        fs::copy(source_path, &pool_path)?;
        fs::remove_file(source_path)?;
    }

    // Add to index if not already present
    if !index.contains(sha512) {
        let resource = PooledResource::new(
            sha512.to_string(),
            original_filename.to_string(),
            content_type,
            size,
        );
        index.add(resource);
        save_pool_index(&index)?;
    }

    Ok(pool_path)
}

/// Add a resource to the pool from bytes (used during download)
///
/// This is more efficient than writing to a temp file first.
pub fn add_resource_from_bytes(
    state: &AppState,
    bytes: &[u8],
    sha512: &str,
    content_type: ContentType,
    original_filename: &str,
) -> Result<PathBuf, AppError> {
    let mut index = state.resource_pool_index.write();

    let pool_path = get_pooled_resource_path(&content_type, sha512, original_filename);

    // If the resource already exists in the pool, just return the path
    if pool_path.exists() && index.contains(sha512) {
        return Ok(pool_path);
    }

    // Ensure pool directory exists
    if let Some(parent) = pool_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write bytes to pool
    fs::write(&pool_path, bytes)?;

    // Add to index if not already present
    if !index.contains(sha512) {
        let resource = PooledResource::new(
            sha512.to_string(),
            original_filename.to_string(),
            content_type,
            bytes.len() as u64,
        );
        index.add(resource);
        save_pool_index(&index)?;
    }

    Ok(pool_path)
}

/// Link a pooled resource to an instance directory
///
/// Creates a hard link or symlink from the pool to the instance content directory.
pub fn link_to_instance(
    state: &AppState,
    sha512: &str,
    content_type: &ContentType,
    instance_id: &str,
    dest_filename: &str,
    preferred_strategy: LinkStrategy,
) -> Result<LinkResult, AppError> {
    let mut index = state.resource_pool_index.write();

    // Get the resource from the index
    let resource = index.get(sha512).ok_or_else(|| {
        AppError::ResourceNotFound(format!("Resource {} not found in pool", sha512))
    })?;

    let pool_path = get_pooled_resource_path(content_type, sha512, &resource.original_filename);

    if !pool_path.exists() {
        return Err(AppError::ResourceNotFound(format!(
            "Pool file not found: {}",
            pool_path.display()
        )));
    }

    // Get the destination path in the instance
    let instances_base = state.settings.read().instances_path.clone();
    let game_dir =
        crate::utils::paths::get_instance_game_dir_with_base(&instances_base, instance_id);
    let dest_path = match content_type {
        ContentType::Mod => game_dir.join("mods").join(dest_filename),
        ContentType::Shader => game_dir.join("shaderpacks").join(dest_filename),
        ContentType::ResourcePack => game_dir.join("resourcepacks").join(dest_filename),
        ContentType::Datapack => game_dir.join("datapacks").join(dest_filename),
        ContentType::World => game_dir.join("saves").join(dest_filename),
    };

    // Ensure parent directory exists
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Remove existing file if present
    if dest_path.exists() {
        fs::remove_file(&dest_path)?;
    }

    // Create the link
    let result = link_with_fallback(&pool_path, &dest_path, preferred_strategy);

    match result {
        Ok(link_result) => {
            // Update usage tracking
            if let Some(resource) = index.get_mut(sha512) {
                resource.add_usage(instance_id, dest_filename);
            }
            save_pool_index(&index)?;

            Ok(LinkResult {
                success: true,
                strategy_used: link_result.strategy_used,
                error: None,
            })
        }
        Err(e) => Ok(LinkResult {
            success: false,
            strategy_used: LinkStrategy::Copy,
            error: Some(e.to_string()),
        }),
    }
}

/// Unlink a resource from an instance
///
/// Removes the link and updates the usage tracking.
/// The pool file is kept until garbage collection runs.
pub fn unlink_from_instance(
    state: &AppState,
    sha512: &str,
    content_type: &ContentType,
    instance_id: &str,
    filename: &str,
) -> Result<(), AppError> {
    let mut index = state.resource_pool_index.write();

    // Get the instance path
    let instances_base = state.settings.read().instances_path.clone();
    let game_dir =
        crate::utils::paths::get_instance_game_dir_with_base(&instances_base, instance_id);
    let file_path = match content_type {
        ContentType::Mod => game_dir.join("mods").join(filename),
        ContentType::Shader => game_dir.join("shaderpacks").join(filename),
        ContentType::ResourcePack => game_dir.join("resourcepacks").join(filename),
        ContentType::Datapack => game_dir.join("datapacks").join(filename),
        ContentType::World => game_dir.join("saves").join(filename),
    };

    // Remove the file/link
    if file_path.exists() {
        fs::remove_file(&file_path)?;
    }

    // Update usage tracking
    if let Some(resource) = index.get_mut(sha512) {
        resource.remove_usage(instance_id, filename);
    }

    save_pool_index(&index)?;

    Ok(())
}

/// Remove all pool references for an instance
///
/// Called when an instance is deleted.
pub fn remove_instance_from_pool(state: &AppState, instance_id: &str) -> Result<(), AppError> {
    let mut index = state.resource_pool_index.write();

    // Update all resources to remove this instance
    for resource in index.resources.values_mut() {
        resource.remove_instance_usages(instance_id);
    }

    save_pool_index(&index)?;

    Ok(())
}

/// Run garbage collection on the pool
///
/// Removes resources that are not used by any instance and are older than the minimum age.
pub fn garbage_collect(state: &AppState) -> Result<GarbageCollectionResult, AppError> {
    let mut index = state.resource_pool_index.write();

    let candidates: Vec<String> = index
        .get_gc_candidates(GC_MIN_AGE_SECS)
        .iter()
        .map(|r| r.sha512.clone())
        .collect();

    let mut removed = 0;
    let mut bytes_freed: u64 = 0;
    let mut failed = Vec::new();

    for sha512 in candidates {
        if let Some(resource) = index.get(&sha512) {
            let pool_path = get_pooled_resource_path(
                &resource.content_type,
                &sha512,
                &resource.original_filename,
            );

            let size = resource.size;

            if pool_path.exists() {
                match fs::remove_file(&pool_path) {
                    Ok(_) => {
                        index.remove(&sha512);
                        removed += 1;
                        bytes_freed += size;
                    }
                    Err(e) => {
                        failed.push(format!("{}: {}", sha512, e));
                    }
                }
            } else {
                // File doesn't exist, just remove from index
                index.remove(&sha512);
                removed += 1;
            }
        }
    }

    // Update last GC timestamp
    index.last_gc_at = Some(chrono::Utc::now().timestamp());
    save_pool_index(&index)?;

    Ok(GarbageCollectionResult {
        resources_removed: removed,
        bytes_freed,
        failed,
    })
}

/// Get statistics about the resource pool
pub fn get_pool_stats(state: &AppState) -> ResourcePoolStats {
    let index = state.resource_pool_index.read();
    let assets_cache_size = calculate_directory_size(&crate::utils::paths::get_assets_dir());
    let libraries_cache_size = calculate_directory_size(&crate::utils::paths::get_libraries_dir());
    ResourcePoolStats::from_index_with_cache_sizes(&index, assets_cache_size, libraries_cache_size)
}

/// Calculate the total size of a directory recursively
fn calculate_directory_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }

    fn visit_dir(dir: &Path) -> u64 {
        let mut total = 0;
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(meta) = path.metadata() {
                        total += meta.len();
                    }
                } else if path.is_dir() {
                    total += visit_dir(&path);
                }
            }
        }
        total
    }

    visit_dir(path)
}

/// Check if a resource exists in the pool
pub fn resource_exists(state: &AppState, sha512: &str) -> bool {
    let index = state.resource_pool_index.read();
    if let Some(resource) = index.get(sha512) {
        let pool_path =
            get_pooled_resource_path(&resource.content_type, sha512, &resource.original_filename);
        pool_path.exists()
    } else {
        false
    }
}

/// Get the pool path for a resource
pub fn get_resource_path(
    state: &AppState,
    sha512: &str,
    content_type: &ContentType,
) -> Option<PathBuf> {
    let index = state.resource_pool_index.read();
    index
        .get(sha512)
        .map(|r| get_pooled_resource_path(content_type, sha512, &r.original_filename))
}

/// Check if linking is available for the given paths
///
/// Returns information about what link strategies are available.
pub fn check_link_availability(instance_path: &Path) -> Result<LinkAvailability, AppError> {
    let pool_dir = get_resource_pool_dir();

    let same_fs = same_filesystem(&pool_dir, instance_path).unwrap_or(false);
    let symlinks = symlinks_available();

    Ok(LinkAvailability {
        hard_links_available: same_fs,
        symlinks_available: symlinks,
        recommended_strategy: if same_fs {
            LinkStrategy::HardLink
        } else if symlinks {
            LinkStrategy::Symlink
        } else {
            LinkStrategy::Copy
        },
    })
}

/// Information about available linking strategies
#[derive(Debug, Clone)]
pub struct LinkAvailability {
    pub hard_links_available: bool,
    pub symlinks_available: bool,
    pub recommended_strategy: LinkStrategy,
}

/// Verify the integrity of the pool
///
/// Checks that all indexed resources exist on disk and removes orphaned entries.
pub fn verify_pool_integrity(state: &AppState) -> Result<PoolIntegrityReport, AppError> {
    let mut index = state.resource_pool_index.write();

    let mut missing_files = Vec::new();
    let mut orphaned_files = Vec::new();
    let mut valid_count = 0;

    // Check that all indexed resources exist
    let hashes: Vec<String> = index.resources.keys().cloned().collect();
    for sha512 in hashes {
        if let Some(resource) = index.get(&sha512) {
            let pool_path = get_pooled_resource_path(
                &resource.content_type,
                &sha512,
                &resource.original_filename,
            );

            if pool_path.exists() {
                valid_count += 1;
            } else {
                missing_files.push(sha512.clone());
            }
        }
    }

    // Check for orphaned files not in the index
    for content_type in &[
        ContentType::Mod,
        ContentType::Shader,
        ContentType::ResourcePack,
    ] {
        let type_dir = get_resource_pool_dir_for_type(content_type);
        if type_dir.exists() {
            if let Ok(entries) = fs::read_dir(&type_dir) {
                for entry in entries.flatten() {
                    if let Some(filename) = entry.file_name().to_str() {
                        // Extract hash from filename (format: hash.extension)
                        if let Some(hash) = filename.split('.').next() {
                            if !index.contains(hash) {
                                orphaned_files.push(entry.path());
                            }
                        }
                    }
                }
            }
        }
    }

    // Remove missing entries from index
    for sha512 in &missing_files {
        index.remove(sha512);
    }

    if !missing_files.is_empty() {
        save_pool_index(&index)?;
    }

    Ok(PoolIntegrityReport {
        valid_resources: valid_count,
        missing_files,
        orphaned_files,
    })
}

/// Report from pool integrity verification
#[derive(Debug, Clone)]
pub struct PoolIntegrityReport {
    pub valid_resources: usize,
    pub missing_files: Vec<String>,
    pub orphaned_files: Vec<PathBuf>,
}

/// Check if garbage collection should run (24+ hours since last GC or never run)
pub fn should_run_gc(state: &AppState) -> bool {
    let index = state.resource_pool_index.read();

    // Check if there are any unused resources to clean
    let has_candidates = index
        .resources
        .values()
        .any(|r| r.is_unused() && (chrono::Utc::now().timestamp() - r.added_at) > GC_MIN_AGE_SECS);

    if !has_candidates {
        return false;
    }

    // Check if GC has never run or it's been more than 24 hours
    match index.last_gc_at {
        None => true, // Never run before
        Some(last_gc) => {
            let now = chrono::Utc::now().timestamp();
            now - last_gc >= GC_INTERVAL_SECS
        }
    }
}

/// Run garbage collection in the background if conditions are met
/// Returns true if GC was started, false if skipped
pub fn maybe_run_gc_background(state: &AppState) {
    if !should_run_gc(state) {
        return;
    }

    // Run GC - since this is called from startup, we can just run it directly
    // The GC operation is not CPU-intensive, just file I/O
    match garbage_collect(state) {
        Ok(result) => {
            if result.resources_removed > 0 {
                app_info!(
                    "[resource_pool] Auto GC: removed {} unused resources, freed {} bytes",
                    result.resources_removed,
                    result.bytes_freed
                );
            }
        }
        Err(e) => {
            app_error!("[resource_pool] Auto GC failed: {}", e);
        }
    }
}
