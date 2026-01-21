//! Migration service for converting existing instances to use the resource pool
//!
//! This service handles migrating existing instance content to the resource pool,
//! replacing duplicate files with links.

use crate::error::AppError;
use crate::models::{ContentType, InstalledContentManifest};
use crate::services::{manifest_service, resource_pool_service};
use crate::state::AppState;
use crate::utils::paths::get_instance_game_dir_with_base;
use sha2::{Digest, Sha512};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

/// Result of migrating a single instance
#[derive(Debug, Clone)]
pub struct InstanceMigrationResult {
    pub instance_id: String,
    pub files_migrated: usize,
    pub space_saved_bytes: u64,
    pub errors: Vec<String>,
}

/// Result of migrating all instances
#[derive(Debug, Clone)]
pub struct MigrationResult {
    pub instances_migrated: usize,
    pub total_files_migrated: usize,
    pub total_space_saved_bytes: u64,
    pub instance_results: Vec<InstanceMigrationResult>,
}

/// Check if an instance needs migration to the resource pool
pub fn needs_migration(state: &AppState, instance_id: &str) -> Result<bool, AppError> {
    let manifest = match manifest_service::load_manifest(state, instance_id) {
        Ok(m) => m,
        Err(_) => return Ok(false), // No manifest means no managed content
    };

    // Check if any content is not pooled
    let has_unpooled = manifest.mods.iter().any(|c| !c.is_pooled)
        || manifest.shaders.iter().any(|c| !c.is_pooled)
        || manifest.resource_packs.iter().any(|c| !c.is_pooled);

    Ok(has_unpooled)
}

/// Migrate a single instance's content to the resource pool
pub fn migrate_instance(
    state: &AppState,
    instance_id: &str,
) -> Result<InstanceMigrationResult, AppError> {
    let settings = state.get_settings();
    let instances_base = settings.instances_path.clone();
    let game_dir = get_instance_game_dir_with_base(&instances_base, instance_id);

    let mut files_migrated = 0;
    let mut space_saved: u64 = 0;
    let mut errors = Vec::new();

    // Get or create manifest
    let mut manifest = manifest_service::load_manifest(state, instance_id)
        .unwrap_or_else(|_| InstalledContentManifest::default());

    // Migrate mods
    let (migrated, saved, errs) = migrate_content_type(
        state,
        instance_id,
        &game_dir.join("mods"),
        ContentType::Mod,
        &mut manifest.mods,
        &settings.resource_pool.link_strategy,
    )?;
    files_migrated += migrated;
    space_saved += saved;
    errors.extend(errs);

    // Migrate shaders
    let (migrated, saved, errs) = migrate_content_type(
        state,
        instance_id,
        &game_dir.join("shaderpacks"),
        ContentType::Shader,
        &mut manifest.shaders,
        &settings.resource_pool.link_strategy,
    )?;
    files_migrated += migrated;
    space_saved += saved;
    errors.extend(errs);

    // Migrate resource packs
    let (migrated, saved, errs) = migrate_content_type(
        state,
        instance_id,
        &game_dir.join("resourcepacks"),
        ContentType::ResourcePack,
        &mut manifest.resource_packs,
        &settings.resource_pool.link_strategy,
    )?;
    files_migrated += migrated;
    space_saved += saved;
    errors.extend(errs);

    // Save updated manifest
    manifest_service::save_manifest(state, instance_id, &manifest)?;

    Ok(InstanceMigrationResult {
        instance_id: instance_id.to_string(),
        files_migrated,
        space_saved_bytes: space_saved,
        errors,
    })
}

/// Migrate a single content type for an instance
fn migrate_content_type(
    state: &AppState,
    instance_id: &str,
    content_dir: &PathBuf,
    content_type: ContentType,
    manifest_entries: &mut Vec<crate::models::InstalledContent>,
    link_strategy: &crate::models::LinkStrategy,
) -> Result<(usize, u64, Vec<String>), AppError> {
    use crate::models::resource_pool::LinkStrategy;

    let mut migrated = 0;
    let mut space_saved: u64 = 0;
    let mut errors = Vec::new();

    if !content_dir.exists() {
        return Ok((0, 0, vec![]));
    }

    // Process each file in the content directory
    for entry in manifest_entries.iter_mut() {
        if entry.is_pooled {
            continue; // Already pooled
        }

        let file_path = content_dir.join(&entry.filename);
        if !file_path.exists() {
            continue;
        }

        // Compute hash if not present
        let sha512 = if let Some(hash) = &entry.sha512_hash {
            hash.clone()
        } else {
            match compute_file_sha512(&file_path) {
                Ok(h) => h,
                Err(e) => {
                    errors.push(format!(
                        "{}: failed to compute hash - {}",
                        entry.filename, e
                    ));
                    continue;
                }
            }
        };

        // Get file size before migration
        let file_size = fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);

        // Add to pool (this moves the file)
        let _pool_path = match resource_pool_service::add_resource(
            state,
            &file_path,
            &sha512,
            content_type,
            &entry.filename,
        ) {
            Ok(p) => p,
            Err(e) => {
                errors.push(format!("{}: failed to add to pool - {}", entry.filename, e));
                continue;
            }
        };

        // Create link back to instance
        let link_strat = match link_strategy {
            crate::models::LinkStrategy::Auto => LinkStrategy::Auto,
            crate::models::LinkStrategy::HardLink => LinkStrategy::HardLink,
            crate::models::LinkStrategy::Symlink => LinkStrategy::Symlink,
            crate::models::LinkStrategy::Copy => LinkStrategy::Copy,
        };

        match resource_pool_service::link_to_instance(
            state,
            &sha512,
            &content_type,
            instance_id,
            &entry.filename,
            link_strat,
        ) {
            Ok(result) => {
                if result.success {
                    // Update manifest entry
                    entry.sha512_hash = Some(sha512.clone());
                    entry.is_pooled = true;
                    migrated += 1;

                    // If we created a link (not copy), we saved space
                    if result.strategy_used != LinkStrategy::Copy {
                        space_saved += file_size;
                    }
                } else {
                    errors.push(format!(
                        "{}: link failed - {:?}",
                        entry.filename, result.error
                    ));
                }
            }
            Err(e) => {
                errors.push(format!("{}: link failed - {}", entry.filename, e));
            }
        }
    }

    Ok((migrated, space_saved, errors))
}

/// Compute SHA512 hash of a file
fn compute_file_sha512(path: &PathBuf) -> Result<String, AppError> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha512::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Migrate all instances to use the resource pool
pub fn migrate_all_instances(state: &AppState) -> Result<MigrationResult, AppError> {
    use crate::services::instance_service;

    let instances = instance_service::get_all_instances(state)?;
    let mut results = Vec::new();
    let mut total_files = 0;
    let mut total_space: u64 = 0;

    for instance in instances {
        if needs_migration(state, &instance.id)? {
            match migrate_instance(state, &instance.id) {
                Ok(result) => {
                    total_files += result.files_migrated;
                    total_space += result.space_saved_bytes;
                    results.push(result);
                }
                Err(e) => {
                    results.push(InstanceMigrationResult {
                        instance_id: instance.id.clone(),
                        files_migrated: 0,
                        space_saved_bytes: 0,
                        errors: vec![e.to_string()],
                    });
                }
            }
        }
    }

    Ok(MigrationResult {
        instances_migrated: results.len(),
        total_files_migrated: total_files,
        total_space_saved_bytes: total_space,
        instance_results: results,
    })
}
