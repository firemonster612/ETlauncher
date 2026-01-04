//! Manifest service for persisting installed content tracking
//!
//! This service manages the InstalledContentManifest which tracks all content
//! (mods, shaders, resource packs) installed in an instance, including their
//! source (modpack-original vs user-added) and version information.

use crate::error::AppError;
use crate::models::content::{
    ContentSource, ContentType, InstalledContent, InstalledContentManifest, MANIFEST_VERSION,
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
pub fn load_manifest(state: &AppState, instance_id: &str) -> Result<InstalledContentManifest, AppError> {
    let manifest_path = get_manifest_path(state, instance_id);

    if !manifest_path.exists() {
        // Return empty manifest with current version
        return Ok(InstalledContentManifest {
            manifest_version: MANIFEST_VERSION,
            mods: Vec::new(),
            shaders: Vec::new(),
            resource_packs: Vec::new(),
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
pub fn add_content(
    state: &AppState,
    instance_id: &str,
    content: InstalledContent,
) -> Result<(), AppError> {
    let mut manifest = load_manifest(state, instance_id)?;

    // Get the appropriate list based on content type
    let list = match content.content_type {
        ContentType::Mod => &mut manifest.mods,
        ContentType::Shader => &mut manifest.shaders,
        ContentType::ResourcePack => &mut manifest.resource_packs,
    };

    // Remove existing entry with same filename (if updating)
    list.retain(|c| c.filename != content.filename);

    // Add the new content
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
        if content.source == ContentSource::UserAdded || content.source == ContentSource::UserDependency {
            user_content.push(content);
        }
    }
    for content in manifest.shaders {
        if content.source == ContentSource::UserAdded || content.source == ContentSource::UserDependency {
            user_content.push(content);
        }
    }
    for content in manifest.resource_packs {
        if content.source == ContentSource::UserAdded || content.source == ContentSource::UserDependency {
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

    Ok(modpack_content)
}

/// Clear all content from manifest (used before modpack reinstall)
pub fn clear_manifest(state: &AppState, instance_id: &str) -> Result<(), AppError> {
    let manifest = InstalledContentManifest {
        manifest_version: MANIFEST_VERSION,
        mods: Vec::new(),
        shaders: Vec::new(),
        resource_packs: Vec::new(),
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
