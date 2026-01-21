//! Tauri commands for resource pool management

use serde::Serialize;
use tauri::State;

use crate::error::CommandError;
use crate::models::resource_pool::ResourcePoolStats;
use crate::services::{migration_service, resource_pool_service};
use crate::state::AppState;

/// Get resource pool statistics
#[tauri::command]
pub fn get_pool_stats(state: State<'_, AppState>) -> Result<ResourcePoolStats, CommandError> {
    Ok(resource_pool_service::get_pool_stats(&state))
}

/// Response from garbage collection
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GarbageCollectResult {
    pub resources_removed: usize,
    pub bytes_freed: u64,
    pub failed_count: usize,
}

/// Run garbage collection on the resource pool
#[tauri::command]
pub fn garbage_collect_pool(
    state: State<'_, AppState>,
) -> Result<GarbageCollectResult, CommandError> {
    let result = resource_pool_service::garbage_collect(&state).map_err(CommandError::from)?;

    Ok(GarbageCollectResult {
        resources_removed: result.resources_removed,
        bytes_freed: result.bytes_freed,
        failed_count: result.failed.len(),
    })
}

/// Response from pool integrity verification
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolIntegrityResult {
    pub valid_resources: usize,
    pub missing_files: usize,
    pub orphaned_files: usize,
}

/// Verify resource pool integrity
#[tauri::command]
pub fn verify_pool_integrity(
    state: State<'_, AppState>,
) -> Result<PoolIntegrityResult, CommandError> {
    let result =
        resource_pool_service::verify_pool_integrity(&state).map_err(CommandError::from)?;

    Ok(PoolIntegrityResult {
        valid_resources: result.valid_resources,
        missing_files: result.missing_files.len(),
        orphaned_files: result.orphaned_files.len(),
    })
}

/// Check if an instance needs migration to the resource pool
#[tauri::command]
pub fn check_instance_needs_migration(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<bool, CommandError> {
    migration_service::needs_migration(&state, &instance_id).map_err(CommandError::from)
}

/// Response from instance migration
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceMigrationResult {
    pub instance_id: String,
    pub files_migrated: usize,
    pub space_saved_bytes: u64,
    pub error_count: usize,
}

/// Migrate a single instance to use the resource pool
#[tauri::command]
pub fn migrate_instance_to_pool(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<InstanceMigrationResult, CommandError> {
    let result =
        migration_service::migrate_instance(&state, &instance_id).map_err(CommandError::from)?;

    Ok(InstanceMigrationResult {
        instance_id: result.instance_id,
        files_migrated: result.files_migrated,
        space_saved_bytes: result.space_saved_bytes,
        error_count: result.errors.len(),
    })
}

/// Response from migrating all instances
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrateAllResult {
    pub instances_migrated: usize,
    pub total_files_migrated: usize,
    pub total_space_saved_bytes: u64,
}

/// Migrate all instances to use the resource pool
#[tauri::command]
pub fn migrate_all_instances_to_pool(
    state: State<'_, AppState>,
) -> Result<MigrateAllResult, CommandError> {
    let result = migration_service::migrate_all_instances(&state).map_err(CommandError::from)?;

    Ok(MigrateAllResult {
        instances_migrated: result.instances_migrated,
        total_files_migrated: result.total_files_migrated,
        total_space_saved_bytes: result.total_space_saved_bytes,
    })
}
