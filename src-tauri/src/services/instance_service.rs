use crate::error::AppError;
use crate::models::instance::{CreateInstanceRequest, Instance, LoaderType, UpdateInstanceRequest};
use crate::state::AppState;
use crate::utils::paths::{
    get_instance_dir_with_base, get_instance_game_dir_with_base,
    get_instances_dir_with_base,
};
use chrono::Utc;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Get the configured instances directory from settings
fn get_instances_base_dir(state: &AppState) -> String {
    state.settings.read().instances_path.clone()
}

/// Get the instance.json path for a given instance ID
fn get_instance_config_path(state: &AppState, instance_id: &str) -> PathBuf {
    get_instance_dir_with_base(&get_instances_base_dir(state), instance_id).join("instance.json")
}

/// Load a single instance by ID
pub fn get_instance(state: &AppState, instance_id: &str) -> Result<Instance, AppError> {
    let config_path = get_instance_config_path(state, instance_id);

    if !config_path.exists() {
        return Err(AppError::InstanceNotFound(instance_id.to_string()));
    }

    let content = fs::read_to_string(&config_path)?;
    let instance: Instance = serde_json::from_str(&content)?;
    Ok(instance)
}

/// Load all instances
pub fn get_all_instances(state: &AppState) -> Result<Vec<Instance>, AppError> {
    let instances_dir = get_instances_dir_with_base(&get_instances_base_dir(state));

    if !instances_dir.exists() {
        return Ok(Vec::new());
    }

    let mut instances = Vec::new();

    let entries = fs::read_dir(&instances_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let instance_id = path
                .file_name()
                .and_then(OsStr::to_str)
                .unwrap_or_default();

            // Try to load the instance, skip if it fails (corrupted/incomplete)
            match get_instance(state, instance_id) {
                Ok(instance) => instances.push(instance),
                Err(e) => {
                    eprintln!("Warning: Failed to load instance {}: {}", instance_id, e);
                }
            }
        }
    }

    // Sort by last played (most recent first), then by created date
    instances.sort_by(|a, b| {
        match (a.last_played_at, b.last_played_at) {
            (Some(a_time), Some(b_time)) => b_time.cmp(&a_time),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => b.created_at.cmp(&a.created_at),
        }
    });

    Ok(instances)
}

/// Create a new instance
pub fn create_instance(state: &AppState, request: CreateInstanceRequest) -> Result<Instance, AppError> {
    let id = Uuid::new_v4().to_string();
    let instance_dir = get_instance_dir_with_base(&get_instances_base_dir(state), &id);
    let game_dir = get_instance_game_dir_with_base(&get_instances_base_dir(state), &id);

    // Create directories
    fs::create_dir_all(&instance_dir)?;
    fs::create_dir_all(&game_dir)?;

    // Create standard game subdirectories
    let subdirs = ["mods", "resourcepacks", "saves", "screenshots", "logs", "config"];
    for subdir in subdirs {
        let subdir_path = game_dir.join(subdir);
        fs::create_dir_all(&subdir_path)?;
    }

    let instance = Instance {
        id: id.clone(),
        name: request.name,
        minecraft_version: request.minecraft_version,
        loader_type: request.loader_type.unwrap_or(LoaderType::Vanilla),
        loader_version: request.loader_version,
        created_at: Utc::now().timestamp(),
        last_played_at: None,
        total_play_time: 0,
        icon_path: None,
        java_path: None,
        memory_min_mb: None,
        memory_max_mb: None,
        jvm_args: None,
        game_args: None,
        resolution_width: None,
        resolution_height: None,
    };

    save_instance(state, &instance)?;

    Ok(instance)
}

/// Save an instance to disk
pub fn save_instance(state: &AppState, instance: &Instance) -> Result<(), AppError> {
    let config_path = get_instance_config_path(state, &instance.id);
    let content = serde_json::to_string_pretty(instance)?;
    fs::write(&config_path, content)?;
    Ok(())
}

/// Update an existing instance
pub fn update_instance(state: &AppState, instance_id: &str, updates: UpdateInstanceRequest) -> Result<Instance, AppError> {
    let mut instance = get_instance(state, instance_id)?;

    // Apply updates
    if let Some(name) = updates.name {
        instance.name = name;
    }
    if updates.java_path.is_some() {
        instance.java_path = updates.java_path;
    }
    if updates.memory_min_mb.is_some() {
        instance.memory_min_mb = updates.memory_min_mb;
    }
    if updates.memory_max_mb.is_some() {
        instance.memory_max_mb = updates.memory_max_mb;
    }
    if updates.jvm_args.is_some() {
        instance.jvm_args = updates.jvm_args;
    }
    if updates.game_args.is_some() {
        instance.game_args = updates.game_args;
    }
    if updates.resolution_width.is_some() {
        instance.resolution_width = updates.resolution_width;
    }
    if updates.resolution_height.is_some() {
        instance.resolution_height = updates.resolution_height;
    }

    save_instance(state, &instance)?;

    Ok(instance)
}

/// Delete an instance
pub fn delete_instance(state: &AppState, instance_id: &str, delete_files: bool) -> Result<(), AppError> {
    let instance_dir = get_instance_dir_with_base(&get_instances_base_dir(state), instance_id);

    if !instance_dir.exists() {
        return Err(AppError::InstanceNotFound(instance_id.to_string()));
    }

    if delete_files {
        // Completely remove the instance directory
        fs::remove_dir_all(&instance_dir)?;
    } else {
        // Only remove the instance.json config file
        let config_path = get_instance_config_path(state, instance_id);
        fs::remove_file(&config_path)?;
    }

    Ok(())
}

/// Duplicate an instance with a new name
pub fn duplicate_instance(state: &AppState, instance_id: &str, new_name: String) -> Result<Instance, AppError> {
    let source = get_instance(state, instance_id)?;
    let new_id = Uuid::new_v4().to_string();
    let source_dir = get_instance_dir_with_base(&get_instances_base_dir(state), instance_id);
    let dest_dir = get_instance_dir_with_base(&get_instances_base_dir(state), &new_id);

    // Copy the entire directory
    copy_dir_recursive(&source_dir, &dest_dir)?;

    // Create new instance with updated metadata
    let new_instance = Instance {
        id: new_id,
        name: new_name,
        minecraft_version: source.minecraft_version,
        loader_type: source.loader_type,
        loader_version: source.loader_version,
        created_at: Utc::now().timestamp(),
        last_played_at: None,
        total_play_time: 0,
        icon_path: source.icon_path,
        java_path: source.java_path,
        memory_min_mb: source.memory_min_mb,
        memory_max_mb: source.memory_max_mb,
        jvm_args: source.jvm_args,
        game_args: source.game_args,
        resolution_width: source.resolution_width,
        resolution_height: source.resolution_height,
    };

    save_instance(state, &new_instance)?;

    Ok(new_instance)
}

/// Update instance's last played time and increment play time
pub fn update_play_time(state: &AppState, instance_id: &str, session_duration_secs: u64) -> Result<Instance, AppError> {
    let mut instance = get_instance(state, instance_id)?;
    instance.last_played_at = Some(Utc::now().timestamp());
    instance.total_play_time += session_duration_secs;
    save_instance(state, &instance)?;
    Ok(instance)
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> Result<(), AppError> {
    fs::create_dir_all(dst)?;

    let entries = fs::read_dir(src)?;

    for entry in entries {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Get the game directory for an instance (for use by launch service)
pub fn get_game_directory(state: &AppState, instance_id: &str) -> PathBuf {
    get_instance_game_dir_with_base(&get_instances_base_dir(state), instance_id)
}

/// Check if an instance exists
pub fn instance_exists(state: &AppState, instance_id: &str) -> bool {
    get_instance_config_path(state, instance_id).exists()
}
