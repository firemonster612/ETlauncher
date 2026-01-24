//! Manifest service for persisting installed content tracking
//!
//! This service manages the InstalledContentManifest which tracks all content
//! (mods, shaders, resource packs) installed in an instance, including their
//! source (modpack-original vs user-added) and version information.

use crate::error::AppError;
use crate::models::content::{
    ContentPlatform, ContentSource, ContentType, InstalledContent, InstalledContentManifest,
    MANIFEST_VERSION,
};
use crate::state::AppState;
use crate::utils::paths::get_instance_dir_with_base;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

/// Manifest filename
const MANIFEST_FILENAME: &str = "etlauncher_manifest.json";

/// Get the configured instances directory from settings
fn get_instances_base_dir(state: &AppState) -> String {
    state.settings.read().instances_path.clone()
}

/// Get the manifest file path for a given instance
fn get_manifest_path(state: &AppState, instance_id: &str) -> PathBuf {
    get_instance_dir_with_base(&get_instances_base_dir(state), instance_id).join(MANIFEST_FILENAME)
}

/// Load manifest from disk (returns empty manifest if not exists)
pub fn load_manifest(
    state: &AppState,
    instance_id: &str,
) -> Result<InstalledContentManifest, AppError> {
    let manifest_path = get_manifest_path(state, instance_id);

    if !manifest_path.exists() {
        // Return empty manifest with current version
        return Ok(InstalledContentManifest {
            manifest_version: MANIFEST_VERSION,
            mods: Vec::new(),
            shaders: Vec::new(),
            resource_packs: Vec::new(),
            datapacks: Vec::new(),
            worlds: Vec::new(),
            last_synced_at: None,
        });
    }

    let content = fs::read_to_string(&manifest_path)?;
    let manifest: InstalledContentManifest = serde_json::from_str(&content)?;

    // TODO: Handle manifest migration if version differs
    Ok(manifest)
}

/// Save manifest to disk
pub fn save_manifest(
    state: &AppState,
    instance_id: &str,
    manifest: &InstalledContentManifest,
) -> Result<(), AppError> {
    let manifest_path = get_manifest_path(state, instance_id);

    // Ensure parent directory exists
    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(manifest)?;
    fs::write(&manifest_path, content)?;

    Ok(())
}

/// Add content to manifest
/// If the content already exists (by filename), merges dependency_of lists
pub fn add_content(
    state: &AppState,
    instance_id: &str,
    mut content: InstalledContent,
) -> Result<(), AppError> {
    let mut manifest = load_manifest(state, instance_id)?;

    // Get the appropriate list based on content type
    let list = match content.content_type {
        ContentType::Mod => &mut manifest.mods,
        ContentType::Shader => &mut manifest.shaders,
        ContentType::ResourcePack => &mut manifest.resource_packs,
        ContentType::Datapack => &mut manifest.datapacks,
        ContentType::World => &mut manifest.worlds,
    };

    // Check if content already exists and merge dependency_of if so
    if let Some(existing) = list.iter().find(|c| c.filename == content.filename) {
        // Merge dependency_of: combine existing parents with new parent
        let mut merged_deps = existing.dependency_of.clone();
        for dep in content.dependency_of.iter() {
            if !merged_deps.contains(dep) {
                merged_deps.push(dep.clone());
            }
        }
        content.dependency_of = merged_deps;

        // If existing was a dependency, keep that status
        if existing.is_dependency {
            content.is_dependency = true;
        }
    }

    // Remove existing entry with same filename (if updating)
    list.retain(|c| c.filename != content.filename);

    // Add the new/merged content
    list.push(content);

    save_manifest(state, instance_id, &manifest)?;

    Ok(())
}

/// Remove content from manifest by filename
pub fn remove_content(
    state: &AppState,
    instance_id: &str,
    filename: &str,
    content_type: &ContentType,
) -> Result<(), AppError> {
    let mut manifest = load_manifest(state, instance_id)?;

    let list = match content_type {
        ContentType::Mod => &mut manifest.mods,
        ContentType::Shader => &mut manifest.shaders,
        ContentType::ResourcePack => &mut manifest.resource_packs,
        ContentType::Datapack => &mut manifest.datapacks,
        ContentType::World => &mut manifest.worlds,
    };

    list.retain(|c| c.filename != filename);

    save_manifest(state, instance_id, &manifest)?;

    Ok(())
}

/// Get content by filename from manifest
pub fn get_content(
    state: &AppState,
    instance_id: &str,
    filename: &str,
    content_type: &ContentType,
) -> Result<Option<InstalledContent>, AppError> {
    let manifest = load_manifest(state, instance_id)?;

    let list = match content_type {
        ContentType::Mod => &manifest.mods,
        ContentType::Shader => &manifest.shaders,
        ContentType::ResourcePack => &manifest.resource_packs,
        ContentType::Datapack => &manifest.datapacks,
        ContentType::World => &manifest.worlds,
    };

    Ok(list.iter().find(|c| c.filename == filename).cloned())
}

/// Mark all content in manifest as modpack-original
/// Called after modpack installation to mark all content as from the modpack
pub fn mark_all_as_modpack_content(state: &AppState, instance_id: &str) -> Result<(), AppError> {
    let mut manifest = load_manifest(state, instance_id)?;

    for content in manifest.mods.iter_mut() {
        content.source = ContentSource::ModpackOriginal;
    }
    for content in manifest.shaders.iter_mut() {
        content.source = ContentSource::ModpackOriginal;
    }
    for content in manifest.resource_packs.iter_mut() {
        content.source = ContentSource::ModpackOriginal;
    }
    for content in manifest.datapacks.iter_mut() {
        content.source = ContentSource::ModpackOriginal;
    }
    for content in manifest.worlds.iter_mut() {
        content.source = ContentSource::ModpackOriginal;
    }

    save_manifest(state, instance_id, &manifest)?;

    Ok(())
}

/// Update the last_synced_at timestamp
pub fn update_sync_timestamp(state: &AppState, instance_id: &str) -> Result<(), AppError> {
    let mut manifest = load_manifest(state, instance_id)?;
    manifest.last_synced_at = Some(Utc::now().timestamp());
    save_manifest(state, instance_id, &manifest)?;
    Ok(())
}

/// Get all user-added content (for backup during modpack updates)
pub fn get_user_added_content(
    state: &AppState,
    instance_id: &str,
) -> Result<Vec<InstalledContent>, AppError> {
    let manifest = load_manifest(state, instance_id)?;

    let mut user_content = Vec::new();

    for content in manifest.mods {
        if content.source == ContentSource::UserAdded
            || content.source == ContentSource::UserDependency
        {
            user_content.push(content);
        }
    }
    for content in manifest.shaders {
        if content.source == ContentSource::UserAdded
            || content.source == ContentSource::UserDependency
        {
            user_content.push(content);
        }
    }
    for content in manifest.resource_packs {
        if content.source == ContentSource::UserAdded
            || content.source == ContentSource::UserDependency
        {
            user_content.push(content);
        }
    }
    for content in manifest.datapacks {
        if content.source == ContentSource::UserAdded
            || content.source == ContentSource::UserDependency
        {
            user_content.push(content);
        }
    }
    for content in manifest.worlds {
        if content.source == ContentSource::UserAdded
            || content.source == ContentSource::UserDependency
        {
            user_content.push(content);
        }
    }

    Ok(user_content)
}

/// Get all modpack-original content
pub fn get_modpack_content(
    state: &AppState,
    instance_id: &str,
) -> Result<Vec<InstalledContent>, AppError> {
    let manifest = load_manifest(state, instance_id)?;

    let mut modpack_content = Vec::new();

    for content in manifest.mods {
        if content.source == ContentSource::ModpackOriginal {
            modpack_content.push(content);
        }
    }
    for content in manifest.shaders {
        if content.source == ContentSource::ModpackOriginal {
            modpack_content.push(content);
        }
    }
    for content in manifest.resource_packs {
        if content.source == ContentSource::ModpackOriginal {
            modpack_content.push(content);
        }
    }
    for content in manifest.datapacks {
        if content.source == ContentSource::ModpackOriginal {
            modpack_content.push(content);
        }
    }
    for content in manifest.worlds {
        if content.source == ContentSource::ModpackOriginal {
            modpack_content.push(content);
        }
    }

    Ok(modpack_content)
}

/// Clear all content from manifest (used before modpack reinstall)
pub fn clear_manifest(state: &AppState, instance_id: &str) -> Result<(), AppError> {
    let manifest = InstalledContentManifest {
        manifest_version: MANIFEST_VERSION,
        mods: Vec::new(),
        shaders: Vec::new(),
        resource_packs: Vec::new(),
        datapacks: Vec::new(),
        worlds: Vec::new(),
        last_synced_at: Some(Utc::now().timestamp()),
    };
    save_manifest(state, instance_id, &manifest)?;
    Ok(())
}

/// Delete the manifest file entirely
pub fn delete_manifest(state: &AppState, instance_id: &str) -> Result<(), AppError> {
    let manifest_path = get_manifest_path(state, instance_id);
    if manifest_path.exists() {
        fs::remove_file(&manifest_path)?;
    }
    Ok(())
}

/// Update reverse dependencies for newly installed content
/// Scans the manifest to find mods that depend on this content and updates dependency_of
pub fn update_reverse_dependencies(
    state: &AppState,
    instance_id: &str,
    installed: &InstalledContent,
) -> Result<(), AppError> {
    let mut manifest = load_manifest(state, instance_id)?;

    // Build the ID to search for in other mods' dependency_ids
    let this_id = match installed.installed_from {
        ContentPlatform::Modrinth => installed
            .modrinth_id
            .as_ref()
            .map(|id| format!("modrinth:{}", id)),
        ContentPlatform::CurseForge => installed
            .curseforge_id
            .map(|id| format!("curseforge:{}", id)),
    };

    let Some(search_id) = this_id else {
        return Ok(());
    };

    // Find all content that has this content in their dependency_ids
    let mut dependents: Vec<String> = Vec::new();

    let list = match installed.content_type {
        ContentType::Mod => &manifest.mods,
        ContentType::Shader => &manifest.shaders,
        ContentType::ResourcePack => &manifest.resource_packs,
        ContentType::Datapack => &manifest.datapacks,
        ContentType::World => &manifest.worlds,
    };

    for content in list {
        if content.filename != installed.filename && content.dependency_ids.contains(&search_id) {
            dependents.push(content.filename.clone());
        }
    }

    // If we found dependents, update this content's dependency_of
    if !dependents.is_empty() {
        let list_mut = match installed.content_type {
            ContentType::Mod => &mut manifest.mods,
            ContentType::Shader => &mut manifest.shaders,
            ContentType::ResourcePack => &mut manifest.resource_packs,
            ContentType::Datapack => &mut manifest.datapacks,
            ContentType::World => &mut manifest.worlds,
        };

        if let Some(content) = list_mut
            .iter_mut()
            .find(|c| c.filename == installed.filename)
        {
            for dep in dependents {
                if !content.dependency_of.contains(&dep) {
                    content.dependency_of.push(dep);
                }
            }
            content.is_dependency = true;
            save_manifest(state, instance_id, &manifest)?;
        }
    }

    Ok(())
}

/// Add a parent to an existing content's dependency_of list
/// Used when a new mod depends on already-installed content
pub fn add_dependent(
    state: &AppState,
    instance_id: &str,
    filename: &str,
    content_type: &ContentType,
    parent_filename: &str,
) -> Result<(), AppError> {
    let mut manifest = load_manifest(state, instance_id)?;

    let list = match content_type {
        ContentType::Mod => &mut manifest.mods,
        ContentType::Shader => &mut manifest.shaders,
        ContentType::ResourcePack => &mut manifest.resource_packs,
        ContentType::Datapack => &mut manifest.datapacks,
        ContentType::World => &mut manifest.worlds,
    };

    // Find the content and add the parent if not already present
    if let Some(content) = list.iter_mut().find(|c| c.filename == filename) {
        if !content.dependency_of.contains(&parent_filename.to_string()) {
            content.dependency_of.push(parent_filename.to_string());
            // Also mark as dependency if it wasn't already
            content.is_dependency = true;
            save_manifest(state, instance_id, &manifest)?;
        }
    }

    Ok(())
}

/// Update dependency_ids for an existing content entry
/// Used during scan to backfill dependency info for existing instances
pub fn update_dependency_ids(
    state: &AppState,
    instance_id: &str,
    filename: &str,
    content_type: &ContentType,
    dependency_ids: Vec<String>,
) -> Result<bool, AppError> {
    let mut manifest = load_manifest(state, instance_id)?;

    let list = match content_type {
        ContentType::Mod => &mut manifest.mods,
        ContentType::Shader => &mut manifest.shaders,
        ContentType::ResourcePack => &mut manifest.resource_packs,
        ContentType::Datapack => &mut manifest.datapacks,
        ContentType::World => &mut manifest.worlds,
    };

    // Find the content and update dependency_ids if empty
    if let Some(content) = list.iter_mut().find(|c| c.filename == filename) {
        if content.dependency_ids.is_empty() && !dependency_ids.is_empty() {
            content.dependency_ids = dependency_ids;
            save_manifest(state, instance_id, &manifest)?;
            return Ok(true);
        }
    }

    Ok(false)
}
