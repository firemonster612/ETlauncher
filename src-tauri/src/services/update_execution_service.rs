//! Update execution service
//!
//! Handles the actual execution of updates including backup, content updates,
//! loader installation, and rollback on failure.

use crate::error::AppError;
use crate::models::content::{
    ContentPlatform, ContentType, InstalledContent, InstanceUpdatePlan, ModpackUpdatePlan,
    UpdatePlan, UserContentDecision,
};
use crate::models::instance::{Instance, LoaderType, ModpackPlatform};
use crate::models::modpack::ModpackVersion;
use crate::models::ContentSource;
use crate::services::{
    atlauncher_service, content_install_service, curseforge_service, ftb_service, instance_service,
    loader_service, manifest_service, modpack_install_service, modrinth_service, technic_service,
};
use crate::state::AppState;
use crate::utils::paths::get_instance_game_dir_with_base;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

/// Progress event for update execution
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProgress {
    pub stage: String,
    pub progress: u32,
    pub current_item: Option<String>,
    pub total_items: u32,
    pub completed_items: u32,
}

/// Execute content updates for an instance (same MC version)
pub async fn execute_content_update(
    state: &AppState,
    instance_id: &str,
    plan: &UpdatePlan,
    app_handle: Option<&AppHandle>,
) -> Result<(), AppError> {
    let instance = instance_service::get_instance(state, instance_id)?;
    let instances_base = state.settings.read().instances_path.clone();
    let game_dir = get_instance_game_dir_with_base(&instances_base, instance_id);

    // Create backup before making changes
    emit_progress(app_handle, "Creating backup", 0, None, 0, 0);
    let backup_id = format!("backup_{}", Utc::now().timestamp());
    let backup_path = create_backup(&game_dir, &backup_id)?;

    let result =
        execute_content_update_inner(state, instance_id, &instance, &game_dir, plan, app_handle)
            .await;

    match result {
        Ok(()) => {
            // Success - cleanup backup
            emit_progress(app_handle, "Cleaning up", 98, None, 0, 0);
            cleanup_backup(&backup_path)?;
            emit_progress(app_handle, "Update complete", 100, None, 0, 0);
            Ok(())
        }
        Err(e) => {
            // Failure - restore from backup
            emit_progress(app_handle, "Restoring from backup", 0, None, 0, 0);
            if let Err(restore_err) = restore_backup(&backup_path, &game_dir) {
                eprintln!("Failed to restore backup: {}", restore_err);
            }
            cleanup_backup(&backup_path)?;
            Err(e)
        }
    }
}

/// Inner function for content update execution
async fn execute_content_update_inner(
    state: &AppState,
    instance_id: &str,
    instance: &Instance,
    game_dir: &PathBuf,
    plan: &UpdatePlan,
    app_handle: Option<&AppHandle>,
) -> Result<(), AppError> {
    let manifest = manifest_service::load_manifest(state, instance_id)?;
    let mods_dir = game_dir.join("mods");

    let total_items = plan.content_to_update.len() + plan.content_to_remove.len();
    let mut completed = 0;

    // Remove content marked for removal
    for filename in &plan.content_to_remove {
        emit_progress(
            app_handle,
            "Removing content",
            10 + (completed * 40 / total_items.max(1)) as u32,
            Some(filename.clone()),
            total_items as u32,
            completed as u32,
        );

        let file_path = mods_dir.join(filename);
        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }

        // Remove from manifest
        manifest_service::remove_content(state, instance_id, filename, &ContentType::Mod)?;
        completed += 1;
    }

    // Update content
    for filename in &plan.content_to_update {
        emit_progress(
            app_handle,
            "Updating content",
            50 + (completed * 45 / total_items.max(1)) as u32,
            Some(filename.clone()),
            total_items as u32,
            completed as u32,
        );

        // Find the content in manifest
        if let Some(content) = manifest.mods.iter().find(|c| c.filename == *filename) {
            // Download the new version
            let updated = update_single_content(
                state,
                instance_id,
                content,
                &instance.minecraft_version,
                Some(&instance.loader_type),
                app_handle,
            )
            .await?;

            if let Some(new_content) = updated {
                // Remove old file
                let old_path = mods_dir.join(&content.filename);
                if old_path.exists() && content.filename != new_content.filename {
                    fs::remove_file(&old_path)?;
                }
            }
        }

        completed += 1;
    }

    Ok(())
}

/// Execute a version migration (change MC version)
pub async fn execute_version_migration(
    state: &AppState,
    instance_id: &str,
    target_mc_version: &str,
    target_loader: &LoaderType,
    target_loader_version: &str,
    plan: &UpdatePlan,
    app_handle: Option<&AppHandle>,
) -> Result<Instance, AppError> {
    let mut instance = instance_service::get_instance(state, instance_id)?;
    let instances_base = state.settings.read().instances_path.clone();
    let game_dir = get_instance_game_dir_with_base(&instances_base, instance_id);

    // Create backup
    emit_progress(app_handle, "Creating backup", 0, None, 0, 0);
    let backup_id = format!("backup_{}", Utc::now().timestamp());
    let backup_path = create_backup(&game_dir, &backup_id)?;

    let result = execute_version_migration_inner(
        state,
        instance_id,
        &mut instance,
        &game_dir,
        target_mc_version,
        target_loader,
        target_loader_version,
        plan,
        app_handle,
    )
    .await;

    match result {
        Ok(updated_instance) => {
            emit_progress(app_handle, "Cleaning up", 98, None, 0, 0);
            cleanup_backup(&backup_path)?;
            emit_progress(app_handle, "Migration complete", 100, None, 0, 0);
            Ok(updated_instance)
        }
        Err(e) => {
            emit_progress(app_handle, "Restoring from backup", 0, None, 0, 0);
            if let Err(restore_err) = restore_backup(&backup_path, &game_dir) {
                eprintln!("Failed to restore backup: {}", restore_err);
            }
            cleanup_backup(&backup_path)?;
            Err(e)
        }
    }
}

/// Inner function for version migration execution
async fn execute_version_migration_inner(
    state: &AppState,
    instance_id: &str,
    instance: &mut Instance,
    game_dir: &PathBuf,
    target_mc_version: &str,
    target_loader: &LoaderType,
    target_loader_version: &str,
    plan: &UpdatePlan,
    app_handle: Option<&AppHandle>,
) -> Result<Instance, AppError> {
    let manifest = manifest_service::load_manifest(state, instance_id)?;
    let mods_dir = game_dir.join("mods");

    // Install new loader version
    if *target_loader != LoaderType::Vanilla {
        emit_progress(
            app_handle,
            "Installing mod loader",
            5,
            Some(format!("{:?} {}", target_loader, target_loader_version)),
            0,
            0,
        );

        loader_service::install_loader(
            game_dir,
            *target_loader,
            target_mc_version,
            target_loader_version,
            |msg, pct| {
                emit_progress(
                    app_handle,
                    &format!("Loader: {}", msg),
                    5 + (pct / 5),
                    None,
                    0,
                    0,
                );
            },
        )
        .await?;
    }

    let total_items = plan.content_to_update.len() + plan.content_to_remove.len();
    let mut completed = 0;

    // Remove content marked for removal
    for filename in &plan.content_to_remove {
        emit_progress(
            app_handle,
            "Removing incompatible content",
            25 + (completed * 30 / total_items.max(1)) as u32,
            Some(filename.clone()),
            total_items as u32,
            completed as u32,
        );

        let file_path = mods_dir.join(filename);
        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }
        manifest_service::remove_content(state, instance_id, filename, &ContentType::Mod)?;
        completed += 1;
    }

    // Update content to new MC version compatible versions
    for filename in &plan.content_to_update {
        emit_progress(
            app_handle,
            "Updating content",
            55 + (completed * 40 / total_items.max(1)) as u32,
            Some(filename.clone()),
            total_items as u32,
            completed as u32,
        );

        if let Some(content) = manifest.mods.iter().find(|c| c.filename == *filename) {
            let updated = update_single_content(
                state,
                instance_id,
                content,
                target_mc_version,
                Some(target_loader),
                app_handle,
            )
            .await?;

            if let Some(new_content) = updated {
                let old_path = mods_dir.join(&content.filename);
                if old_path.exists() && content.filename != new_content.filename {
                    fs::remove_file(&old_path)?;
                }
            }
        }

        completed += 1;
    }

    // Update instance metadata
    instance.minecraft_version = target_mc_version.to_string();
    instance.loader_type = *target_loader;
    instance.loader_version = Some(target_loader_version.to_string());

    instance_service::save_instance(state, instance)?;

    Ok(instance.clone())
}

/// Update a single content item to the latest compatible version
async fn update_single_content(
    state: &AppState,
    instance_id: &str,
    content: &InstalledContent,
    mc_version: &str,
    loader: Option<&LoaderType>,
    app_handle: Option<&AppHandle>,
) -> Result<Option<InstalledContent>, AppError> {
    // Try Modrinth first
    if let Some(ref modrinth_id) = content.modrinth_id {
        let versions = modrinth_service::get_content_versions(
            &state.http_client,
            modrinth_id,
            Some(mc_version),
            loader,
        )
        .await?;

        if let Some(latest) = versions.first() {
            // Get content info
            let content_info =
                modrinth_service::get_content(&state.http_client, modrinth_id).await?;

            // Install the new version
            let installed = content_install_service::install_content(
                state,
                instance_id,
                ContentPlatform::Modrinth,
                modrinth_id,
                &content_info.name,
                &content_info.slug,
                content.content_type,
                latest,
                content.is_dependency,
                None, // parent_filename: preserve existing dependency_of from manifest
                Some(content.source.clone()),
                app_handle,
                None, // cancel_token: not used in updates
                None, // queue_id: not used in updates
            )
            .await?;

            return Ok(Some(installed));
        }
    }

    // Try CurseForge
    if let Some(curseforge_id) = content.curseforge_id {
        let api_key = state
            .get_settings()
            .curseforge_api_key
            .ok_or_else(|| AppError::ApiError("CurseForge API key not configured".to_string()))?;

        let versions = curseforge_service::get_content_versions(
            &state.http_client,
            &api_key,
            &curseforge_id.to_string(),
            Some(mc_version),
            loader,
        )
        .await?;

        if let Some(latest) = versions.first() {
            let content_info = curseforge_service::get_content(
                &state.http_client,
                &api_key,
                &curseforge_id.to_string(),
            )
            .await?;

            let installed = content_install_service::install_content(
                state,
                instance_id,
                ContentPlatform::CurseForge,
                &curseforge_id.to_string(),
                &content_info.name,
                &content_info.slug,
                content.content_type,
                latest,
                content.is_dependency,
                None, // parent_filename: preserve existing dependency_of from manifest
                Some(content.source.clone()),
                app_handle,
                None, // cancel_token: not used in updates
                None, // queue_id: not used in updates
            )
            .await?;

            return Ok(Some(installed));
        }
    }

    Ok(None)
}

// === Backup/Restore Functions ===

/// Create a backup of the content folders
fn create_backup(game_dir: &PathBuf, backup_id: &str) -> Result<PathBuf, AppError> {
    let backup_dir = game_dir.join(".etlauncher_backups").join(backup_id);
    fs::create_dir_all(&backup_dir)?;

    // Backup mods folder
    let mods_dir = game_dir.join("mods");
    if mods_dir.exists() {
        let backup_mods = backup_dir.join("mods");
        copy_dir_all(&mods_dir, &backup_mods)?;
    }

    // Backup shaders folder
    let shaders_dir = game_dir.join("shaderpacks");
    if shaders_dir.exists() {
        let backup_shaders = backup_dir.join("shaderpacks");
        copy_dir_all(&shaders_dir, &backup_shaders)?;
    }

    // Backup resource packs folder
    let resourcepacks_dir = game_dir.join("resourcepacks");
    if resourcepacks_dir.exists() {
        let backup_resourcepacks = backup_dir.join("resourcepacks");
        copy_dir_all(&resourcepacks_dir, &backup_resourcepacks)?;
    }

    Ok(backup_dir)
}

/// Restore from a backup
fn restore_backup(backup_dir: &PathBuf, game_dir: &PathBuf) -> Result<(), AppError> {
    // Restore mods
    let backup_mods = backup_dir.join("mods");
    if backup_mods.exists() {
        let mods_dir = game_dir.join("mods");
        if mods_dir.exists() {
            fs::remove_dir_all(&mods_dir)?;
        }
        copy_dir_all(&backup_mods, &mods_dir)?;
    }

    // Restore shaders
    let backup_shaders = backup_dir.join("shaderpacks");
    if backup_shaders.exists() {
        let shaders_dir = game_dir.join("shaderpacks");
        if shaders_dir.exists() {
            fs::remove_dir_all(&shaders_dir)?;
        }
        copy_dir_all(&backup_shaders, &shaders_dir)?;
    }

    // Restore resource packs
    let backup_resourcepacks = backup_dir.join("resourcepacks");
    if backup_resourcepacks.exists() {
        let resourcepacks_dir = game_dir.join("resourcepacks");
        if resourcepacks_dir.exists() {
            fs::remove_dir_all(&resourcepacks_dir)?;
        }
        copy_dir_all(&backup_resourcepacks, &resourcepacks_dir)?;
    }

    Ok(())
}

/// Clean up a backup directory
fn cleanup_backup(backup_dir: &PathBuf) -> Result<(), AppError> {
    if backup_dir.exists() {
        fs::remove_dir_all(backup_dir)?;
    }
    Ok(())
}

/// Recursively copy a directory
fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> Result<(), AppError> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Emit progress event
fn emit_progress(
    app_handle: Option<&AppHandle>,
    stage: &str,
    progress: u32,
    current_item: Option<String>,
    total_items: u32,
    completed_items: u32,
) {
    if let Some(handle) = app_handle {
        let _ = handle.emit(
            "update_progress",
            UpdateProgress {
                stage: stage.to_string(),
                progress,
                current_item,
                total_items,
                completed_items,
            },
        );
    }
}

// =============================================================================
// NEW UPDATE SYSTEM EXECUTION FUNCTIONS
// =============================================================================

/// Execute a modpack update
///
/// Updates a modpack instance to a new version, handling user-added content based on decisions.
pub async fn execute_modpack_update(
    state: &AppState,
    instance_id: &str,
    plan: &ModpackUpdatePlan,
    app_handle: Option<&AppHandle>,
) -> Result<Instance, AppError> {
    // Validate all user content decisions are not Pending
    for (filename, decision) in &plan.user_content_decisions {
        if *decision == UserContentDecision::Pending {
            return Err(AppError::InvalidInput(format!(
                "No decision made for user-added content: {}",
                filename
            )));
        }
    }

    let instance = instance_service::get_instance(state, instance_id)?;
    let instances_base = state.settings.read().instances_path.clone();
    let game_dir = get_instance_game_dir_with_base(&instances_base, instance_id);

    // Create backup
    emit_progress(app_handle, "Creating backup", 0, None, 0, 0);
    let backup_id = format!("backup_{}", Utc::now().timestamp());
    let backup_path = create_backup(&game_dir, &backup_id)?;

    let result =
        execute_modpack_update_inner(state, instance_id, &instance, &game_dir, plan, app_handle)
            .await;

    match result {
        Ok(updated_instance) => {
            emit_progress(app_handle, "Cleaning up", 98, None, 0, 0);
            cleanup_backup(&backup_path)?;
            emit_progress(app_handle, "Update complete", 100, None, 0, 0);
            Ok(updated_instance)
        }
        Err(e) => {
            emit_progress(app_handle, "Restoring from backup", 0, None, 0, 0);
            if let Err(restore_err) = restore_backup(&backup_path, &game_dir) {
                eprintln!("Failed to restore backup: {}", restore_err);
            }
            cleanup_backup(&backup_path)?;
            Err(e)
        }
    }
}

/// Inner function for modpack update execution
async fn execute_modpack_update_inner(
    state: &AppState,
    instance_id: &str,
    instance: &Instance,
    game_dir: &PathBuf,
    plan: &ModpackUpdatePlan,
    app_handle: Option<&AppHandle>,
) -> Result<Instance, AppError> {
    let platform = instance
        .modpack_platform
        .as_ref()
        .ok_or_else(|| AppError::InvalidInput("Not a modpack instance".to_string()))?;
    let modpack_id = instance
        .modpack_id
        .as_ref()
        .ok_or_else(|| AppError::InvalidInput("Modpack ID missing".to_string()))?;

    // Identify content to keep
    let content_to_keep: Vec<String> = plan
        .user_content_decisions
        .iter()
        .filter(|(_, decision)| **decision == UserContentDecision::Keep)
        .map(|(filename, _)| filename.clone())
        .collect();

    // Save user-added content marked as Keep to temp folder
    emit_progress(app_handle, "Preserving user content", 5, None, 0, 0);
    let temp_dir = game_dir.join(".etlauncher_user_content_temp");
    preserve_user_content(game_dir, &temp_dir, &content_to_keep)?;

    // Clear content folders
    emit_progress(app_handle, "Clearing content folders", 10, None, 0, 0);
    clear_content_folders(game_dir)?;

    // Fetch the target version info
    emit_progress(app_handle, "Fetching modpack version", 15, None, 0, 0);
    let target_version =
        get_modpack_version(state, platform, modpack_id, &plan.target_version_id).await?;

    // Install the new modpack version
    emit_progress(
        app_handle,
        "Downloading modpack",
        20,
        Some(target_version.name.clone()),
        0,
        0,
    );

    // Use the appropriate platform installer
    match platform {
        ModpackPlatform::Modrinth => {
            modpack_install_service::install_modrinth_modpack_version(
                state,
                instance_id,
                game_dir,
                modpack_id,
                &plan.target_version_id,
                |stage, progress| {
                    emit_progress(app_handle, stage, 20 + (progress * 60 / 100), None, 0, 0);
                },
            )
            .await?;
        }
        ModpackPlatform::CurseForge => {
            let api_key = state.get_settings().curseforge_api_key.ok_or_else(|| {
                AppError::ApiError("CurseForge API key not configured".to_string())
            })?;
            modpack_install_service::install_curseforge_modpack_version(
                state,
                instance_id,
                game_dir,
                &api_key,
                modpack_id,
                &plan.target_version_id,
                |stage, progress| {
                    emit_progress(app_handle, stage, 20 + (progress * 60 / 100), None, 0, 0);
                },
            )
            .await?;
        }
        ModpackPlatform::FTB => {
            modpack_install_service::install_ftb_modpack_version(
                state,
                instance_id,
                game_dir,
                modpack_id,
                &plan.target_version_id,
                |stage, progress| {
                    emit_progress(app_handle, stage, 20 + (progress * 60 / 100), None, 0, 0);
                },
            )
            .await?;
        }
        ModpackPlatform::Technic => {
            modpack_install_service::install_technic_modpack_version(
                state,
                instance_id,
                game_dir,
                modpack_id,
                &plan.target_version_id,
                |stage, progress| {
                    emit_progress(app_handle, stage, 20 + (progress * 60 / 100), None, 0, 0);
                },
            )
            .await?;
        }
        ModpackPlatform::ATLauncher => {
            modpack_install_service::install_atlauncher_modpack_version(
                state,
                instance_id,
                game_dir,
                modpack_id,
                &plan.target_version_id,
                |stage, progress| {
                    emit_progress(app_handle, stage, 20 + (progress * 60 / 100), None, 0, 0);
                },
            )
            .await?;
        }
    }

    // Try to update user-added content that was kept, fall back to old version if no update found
    emit_progress(app_handle, "Updating user content", 85, None, 0, 0);
    let manifest = manifest_service::load_manifest(state, instance_id)?;
    let kept_content_count = content_to_keep.len();

    for (kept_completed, filename) in content_to_keep.iter().enumerate() {
        emit_progress(
            app_handle,
            "Updating user content",
            85 + (kept_completed * 10 / kept_content_count.max(1)) as u32,
            Some(filename.clone()),
            kept_content_count as u32,
            kept_completed as u32,
        );

        // Find the content info in manifest
        let content = manifest
            .mods
            .iter()
            .chain(manifest.shaders.iter())
            .chain(manifest.resource_packs.iter())
            .find(|c| c.filename == *filename);

        let mut updated = false;

        if let Some(content_info) = content {
            // Only try to update if we have platform IDs
            if content_info.modrinth_id.is_some() || content_info.curseforge_id.is_some() {
                // Try to find and install a compatible version
                let update_result = update_single_content(
                    state,
                    instance_id,
                    content_info,
                    &target_version.mc_version,
                    Some(&target_version.loader_type),
                    app_handle,
                )
                .await;

                if let Ok(Some(_new_content)) = update_result {
                    // Successfully updated - don't restore old file
                    updated = true;
                }
            }
        }

        // If update failed or not possible, restore the old file from temp
        if !updated {
            let temp_file = temp_dir.join(filename);
            if temp_file.exists() {
                let mods_dir = game_dir.join("mods");
                fs::create_dir_all(&mods_dir)?;
                let dest = mods_dir.join(filename);
                fs::copy(&temp_file, &dest)?;
            }
        }
    }

    let _ = fs::remove_dir_all(&temp_dir);

    // Update instance metadata
    emit_progress(app_handle, "Updating instance", 90, None, 0, 0);

    // Resolve loader version if missing (the install functions resolve it internally but don't return it)
    let (final_loader_type, final_loader_version) = if target_version.loader_version.is_none()
        && target_version.loader_type != LoaderType::Vanilla
    {
        // Check if the instance has mods
        let mods_dir = game_dir.join("mods");
        let has_mods = mods_dir.exists()
            && fs::read_dir(&mods_dir)
                .map(|d| d.count() > 0)
                .unwrap_or(false);

        modpack_install_service::resolve_loader_for_pack(
            &target_version.mc_version,
            target_version.loader_type,
            target_version.loader_version.clone(),
            has_mods,
        )
        .await?
    } else {
        (
            target_version.loader_type,
            target_version.loader_version.clone(),
        )
    };

    let mut updated_instance = instance.clone();
    updated_instance.minecraft_version = target_version.mc_version.clone();
    updated_instance.loader_type = final_loader_type;
    updated_instance.loader_version = final_loader_version;
    updated_instance.modpack_version_id = Some(plan.target_version_id.clone());

    instance_service::save_instance(state, &updated_instance)?;

    // Rebuild manifest
    emit_progress(app_handle, "Rebuilding manifest", 95, None, 0, 0);
    modpack_install_service::create_modpack_manifest(state, instance_id, game_dir)?;

    // Re-mark kept user content as UserAdded so it's detected in future updates
    if !content_to_keep.is_empty() {
        let mut new_manifest = manifest_service::load_manifest(state, instance_id)?;
        for filename in &content_to_keep {
            // Find the content in the manifest and mark as UserAdded
            for content in new_manifest.mods.iter_mut() {
                if content.filename == *filename {
                    content.source = ContentSource::UserAdded;
                }
            }
            for content in new_manifest.shaders.iter_mut() {
                if content.filename == *filename {
                    content.source = ContentSource::UserAdded;
                }
            }
            for content in new_manifest.resource_packs.iter_mut() {
                if content.filename == *filename {
                    content.source = ContentSource::UserAdded;
                }
            }
        }
        manifest_service::save_manifest(state, instance_id, &new_manifest)?;
    }

    Ok(updated_instance)
}

/// Execute a non-modpack instance update
///
/// Updates an instance to a new Minecraft version, handling incompatible content based on decisions.
pub async fn execute_instance_update(
    state: &AppState,
    instance_id: &str,
    plan: &InstanceUpdatePlan,
    app_handle: Option<&AppHandle>,
) -> Result<Instance, AppError> {
    // Validate all incompatible content decisions are not Pending
    for (filename, decision) in &plan.incompatible_decisions {
        if *decision == UserContentDecision::Pending {
            return Err(AppError::InvalidInput(format!(
                "No decision made for incompatible content: {}",
                filename
            )));
        }
    }

    let instance = instance_service::get_instance(state, instance_id)?;
    let instances_base = state.settings.read().instances_path.clone();
    let game_dir = get_instance_game_dir_with_base(&instances_base, instance_id);

    // Create backup
    emit_progress(app_handle, "Creating backup", 0, None, 0, 0);
    let backup_id = format!("backup_{}", Utc::now().timestamp());
    let backup_path = create_backup(&game_dir, &backup_id)?;

    let result =
        execute_instance_update_inner(state, instance_id, &instance, &game_dir, plan, app_handle)
            .await;

    match result {
        Ok(updated_instance) => {
            emit_progress(app_handle, "Cleaning up", 98, None, 0, 0);
            cleanup_backup(&backup_path)?;
            emit_progress(app_handle, "Update complete", 100, None, 0, 0);
            Ok(updated_instance)
        }
        Err(e) => {
            emit_progress(app_handle, "Restoring from backup", 0, None, 0, 0);
            if let Err(restore_err) = restore_backup(&backup_path, &game_dir) {
                eprintln!("Failed to restore backup: {}", restore_err);
            }
            cleanup_backup(&backup_path)?;
            Err(e)
        }
    }
}

/// Inner function for non-modpack instance update execution
async fn execute_instance_update_inner(
    state: &AppState,
    instance_id: &str,
    instance: &Instance,
    game_dir: &PathBuf,
    plan: &InstanceUpdatePlan,
    app_handle: Option<&AppHandle>,
) -> Result<Instance, AppError> {
    let manifest = manifest_service::load_manifest(state, instance_id)?;
    let mods_dir = game_dir.join("mods");

    // Install new loader version if needed
    if plan.target_loader_type != LoaderType::Vanilla {
        if let Some(ref loader_version) = plan.target_loader_version {
            emit_progress(
                app_handle,
                "Installing mod loader",
                5,
                Some(format!("{:?} {}", plan.target_loader_type, loader_version)),
                0,
                0,
            );

            loader_service::install_loader(
                game_dir,
                plan.target_loader_type,
                &plan.target_mc_version,
                loader_version,
                |msg, pct| {
                    emit_progress(
                        app_handle,
                        &format!("Loader: {}", msg),
                        5 + (pct / 5),
                        None,
                        0,
                        0,
                    );
                },
            )
            .await?;
        }
    }

    // Collect content to remove
    let content_to_remove: Vec<String> = plan
        .incompatible_decisions
        .iter()
        .filter(|(_, decision)| **decision == UserContentDecision::Remove)
        .map(|(filename, _)| filename.clone())
        .collect();

    // Collect all content filenames that need updating (excluding those being removed)
    let all_content: Vec<InstalledContent> = manifest
        .mods
        .iter()
        .chain(manifest.shaders.iter())
        .chain(manifest.resource_packs.iter())
        .filter(|c| !content_to_remove.contains(&c.filename))
        .cloned()
        .collect();

    let total_items = content_to_remove.len() + all_content.len();
    let mut completed = 0;

    // Remove incompatible content marked for removal
    for filename in &content_to_remove {
        emit_progress(
            app_handle,
            "Removing incompatible content",
            25 + (completed * 20 / total_items.max(1)) as u32,
            Some(filename.clone()),
            total_items as u32,
            completed as u32,
        );

        let file_path = mods_dir.join(filename);
        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }
        manifest_service::remove_content(state, instance_id, filename, &ContentType::Mod)?;
        completed += 1;
    }

    // Update remaining content to new MC version compatible versions
    for content in &all_content {
        emit_progress(
            app_handle,
            "Updating content",
            45 + (completed * 50 / total_items.max(1)) as u32,
            Some(content.filename.clone()),
            total_items as u32,
            completed as u32,
        );

        // Skip unidentified content (keep as-is)
        if content.modrinth_id.is_none() && content.curseforge_id.is_none() {
            completed += 1;
            continue;
        }

        let updated = update_single_content(
            state,
            instance_id,
            content,
            &plan.target_mc_version,
            Some(&plan.target_loader_type),
            app_handle,
        )
        .await?;

        if let Some(new_content) = updated {
            let old_path = mods_dir.join(&content.filename);
            if old_path.exists() && content.filename != new_content.filename {
                fs::remove_file(&old_path)?;
            }
        }

        completed += 1;
    }

    // Update instance metadata
    emit_progress(app_handle, "Updating instance", 95, None, 0, 0);
    let mut updated_instance = instance.clone();
    updated_instance.minecraft_version = plan.target_mc_version.clone();
    updated_instance.loader_type = plan.target_loader_type;
    updated_instance.loader_version = plan.target_loader_version.clone();

    instance_service::save_instance(state, &updated_instance)?;

    Ok(updated_instance)
}

// === Helper Functions for New Update System ===

/// Get modpack version info from platform
async fn get_modpack_version(
    state: &AppState,
    platform: &ModpackPlatform,
    modpack_id: &str,
    version_id: &str,
) -> Result<ModpackVersion, AppError> {
    let versions = match platform {
        ModpackPlatform::Modrinth => {
            modrinth_service::get_modpack_versions(&state.http_client, modpack_id).await?
        }
        ModpackPlatform::CurseForge => {
            let api_key = state.get_settings().curseforge_api_key.ok_or_else(|| {
                AppError::ApiError("CurseForge API key not configured".to_string())
            })?;
            curseforge_service::get_modpack_versions(&state.http_client, &api_key, modpack_id)
                .await?
        }
        ModpackPlatform::FTB => {
            ftb_service::get_modpack_versions(&state.http_client, modpack_id).await?
        }
        ModpackPlatform::Technic => {
            technic_service::get_modpack_versions(&state.http_client, modpack_id).await?
        }
        ModpackPlatform::ATLauncher => {
            atlauncher_service::get_modpack_versions(&state.http_client, modpack_id).await?
        }
    };

    versions
        .into_iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| AppError::InvalidInput(format!("Version {} not found", version_id)))
}

/// Preserve user content to a temp folder
fn preserve_user_content(
    game_dir: &PathBuf,
    temp_dir: &PathBuf,
    filenames: &[String],
) -> Result<(), AppError> {
    if filenames.is_empty() {
        return Ok(());
    }

    fs::create_dir_all(temp_dir)?;

    // Check mods folder
    let mods_dir = game_dir.join("mods");
    for filename in filenames {
        let src = mods_dir.join(filename);
        if src.exists() {
            let dst = temp_dir.join(filename);
            fs::copy(&src, &dst)?;
        }
    }

    // Check shaders folder
    let shaders_dir = game_dir.join("shaderpacks");
    for filename in filenames {
        let src = shaders_dir.join(filename);
        if src.exists() {
            let dst = temp_dir.join(filename);
            fs::copy(&src, &dst)?;
        }
    }

    // Check resourcepacks folder
    let resourcepacks_dir = game_dir.join("resourcepacks");
    for filename in filenames {
        let src = resourcepacks_dir.join(filename);
        if src.exists() {
            let dst = temp_dir.join(filename);
            fs::copy(&src, &dst)?;
        }
    }

    Ok(())
}

/// Clear content folders (mods, shaders, resourcepacks)
fn clear_content_folders(game_dir: &PathBuf) -> Result<(), AppError> {
    let folders = ["mods", "shaderpacks", "resourcepacks"];

    for folder in folders {
        let dir = game_dir.join(folder);
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        fs::create_dir_all(&dir)?;
    }

    Ok(())
}
