use crate::error::AppError;
use crate::models::instance::{
    CreateInstanceRequest, DownloadProgress, Instance, InstanceSetupStatus, LoaderType,
    UpdateInstanceRequest,
};
use crate::models::ContentType;
use crate::services::{download_service, manifest_service, resource_pool_service};
use crate::state::AppState;
use crate::utils::paths::{
    get_instance_dir_with_base, get_instance_game_dir_with_base, get_instances_dir_with_base,
};
use chrono::Utc;
use rand::Rng;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

/// All available entity icons for random assignment
const ALL_ENTITY_ICONS: &[&str] = &[
    // Hostile mobs
    "blaze",
    "breeze",
    "creaking",
    "creeper",
    "elder_guardian",
    "endermite",
    "ghast",
    "happy_ghast",
    "guardian",
    "hoglin",
    "zoglin",
    "illager_evoker",
    "illager_illusioner",
    "illager_pillager",
    "illager_ravager",
    "illager_vex",
    "illager_vindicator",
    "phantom",
    "piglin_brute",
    "shulker",
    "silverfish",
    "skeleton",
    "skeleton_stray",
    "skeleton_wither",
    "slime",
    "magma_cube",
    "spider",
    "cave_spider",
    "warden",
    "witch",
    "wither",
    "zombie",
    "zombie_drowned",
    "zombie_husk",
    "zombie_villager",
    // Passive mobs
    "allay",
    "armadillo",
    "axolotl_blue",
    "axolotl_cyan",
    "axolotl_gold",
    "axolotl_pink",
    "axolotl_wild",
    "bat",
    "camel",
    "cat_black",
    "cat_british",
    "cat_calico",
    "cat_jellie",
    "cat_persian",
    "cat_ragdoll",
    "cat_red",
    "cat_siamese",
    "cat_tabby",
    "cat_default",
    "cat_white",
    "chicken",
    "cow",
    "mooshroom",
    "mooshroom_brown",
    "donkey",
    "horse_black",
    "horse_brown",
    "horse_chestnut",
    "horse_creamy",
    "horse_gray",
    "horse_white",
    "mule",
    "skeleton_horse",
    "zombie_horse",
    "llama_brown",
    "llama_cream",
    "llama_gray",
    "llama_white",
    "ocelot",
    "panda",
    "parrot_blue",
    "parrot_red_blue",
    "parrot_gray",
    "parrot_green",
    "parrot_yellow_blue",
    "pig",
    "rabbit_brown",
    "rabbit_white",
    "rabbit_black",
    "rabbit_gold",
    "sheep_white",
    "sheep_black",
    "sheep_brown",
    "sheep_pink",
    "sniffer",
    "strider",
    "villager",
    "wandering_trader",
    // Neutral mobs
    "bee",
    "enderman",
    "fox",
    "fox_snow",
    "goat",
    "iron_golem",
    "piglin",
    "zombified_piglin",
    "polar_bear",
    "snow_golem",
    "wolf",
    "wolf_black",
    "wolf_snowy",
    "wolf_spotted",
    // Aquatic mobs
    "dolphin",
    "fish_cod",
    "fish_salmon",
    "fish_pufferfish",
    "fish_tropical",
    "frog_cold",
    "frog_temperate",
    "frog_warm",
    "squid",
    "glow_squid",
    "tadpole",
    "turtle",
    // Other
    "enderdragon",
    "end_crystal",
    "armorstand",
];

/// Get a random entity icon path, prioritizing icons not already in use
pub fn get_random_entity_icon(used_icons: &[String]) -> String {
    let mut rng = rand::rng();

    // Find icons that haven't been used yet
    let unused_icons: Vec<&str> = ALL_ENTITY_ICONS
        .iter()
        .filter(|icon| !used_icons.contains(&format!("entity:{}", icon)))
        .copied()
        .collect();

    // If there are unused icons, pick from those; otherwise pick from all
    let icon = if !unused_icons.is_empty() {
        let index = rng.random_range(0..unused_icons.len());
        unused_icons[index]
    } else {
        let index = rng.random_range(0..ALL_ENTITY_ICONS.len());
        ALL_ENTITY_ICONS[index]
    };

    format!("entity:{}", icon)
}

/// Collect all icon paths currently in use by instances
pub fn get_used_icons(state: &AppState) -> Vec<String> {
    match get_all_instances(state) {
        Ok(instances) => instances.into_iter().filter_map(|i| i.icon_path).collect(),
        Err(_) => Vec::new(),
    }
}

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
            let instance_id = path.file_name().and_then(OsStr::to_str).unwrap_or_default();

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
    instances.sort_by(|a, b| match (a.last_played_at, b.last_played_at) {
        (Some(a_time), Some(b_time)) => b_time.cmp(&a_time),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => b.created_at.cmp(&a.created_at),
    });

    Ok(instances)
}

/// Create a new instance
pub fn create_instance(
    state: &AppState,
    request: CreateInstanceRequest,
) -> Result<Instance, AppError> {
    let id = Uuid::new_v4().to_string();
    let instance_dir = get_instance_dir_with_base(&get_instances_base_dir(state), &id);
    let game_dir = get_instance_game_dir_with_base(&get_instances_base_dir(state), &id);

    // Create directories
    fs::create_dir_all(&instance_dir)?;
    fs::create_dir_all(&game_dir)?;

    // Create standard game subdirectories
    let subdirs = [
        "mods",
        "resourcepacks",
        "saves",
        "screenshots",
        "logs",
        "config",
    ];
    for subdir in subdirs {
        let subdir_path = game_dir.join(subdir);
        fs::create_dir_all(&subdir_path)?;
    }

    // Get icons already in use to prioritize unused ones
    let used_icons = get_used_icons(state);

    let instance = Instance {
        id: id.clone(),
        name: request.name,
        minecraft_version: request.minecraft_version,
        loader_type: request.loader_type.unwrap_or(LoaderType::Vanilla),
        loader_version: request.loader_version,
        created_at: Utc::now().timestamp(),
        last_played_at: None,
        total_play_time: 0,
        icon_path: Some(get_random_entity_icon(&used_icons)),
        java_path: None,
        memory_min_mb: None,
        memory_max_mb: None,
        jvm_args: None,
        game_args: None,
        resolution_width: None,
        resolution_height: None,
        modpack_platform: None,
        modpack_id: None,
        modpack_version_id: None,
        description: None,
        author: None,
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
pub fn update_instance(
    state: &AppState,
    instance_id: &str,
    updates: UpdateInstanceRequest,
) -> Result<Instance, AppError> {
    let mut instance = get_instance(state, instance_id)?;

    // Apply updates
    if let Some(name) = updates.name {
        instance.name = name;
    }
    if let Some(loader_type) = updates.loader_type {
        instance.loader_type = loader_type;
    }
    if updates.loader_version.is_some() {
        instance.loader_version = updates.loader_version;
    }
    if updates.icon_path.is_some() {
        instance.icon_path = updates.icon_path;
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
    if updates.description.is_some() {
        instance.description = updates.description;
    }
    if updates.author.is_some() {
        instance.author = updates.author;
    }

    save_instance(state, &instance)?;

    Ok(instance)
}

/// Delete an instance
pub fn delete_instance(
    state: &AppState,
    instance_id: &str,
    delete_files: bool,
) -> Result<(), AppError> {
    let instance_dir = get_instance_dir_with_base(&get_instances_base_dir(state), instance_id);

    if !instance_dir.exists() {
        return Err(AppError::InstanceNotFound(instance_id.to_string()));
    }

    // Remove instance from resource pool tracking before deleting files
    // This updates the "space saved" calculation by removing usage references
    if let Err(e) = resource_pool_service::remove_instance_from_pool(state, instance_id) {
        eprintln!(
            "Warning: Failed to remove instance {} from resource pool: {}",
            instance_id, e
        );
        // Continue with deletion even if pool update fails
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
///
/// When resource pool is enabled, pooled content is linked instead of copied.
pub fn duplicate_instance(
    state: &AppState,
    instance_id: &str,
    new_name: String,
) -> Result<Instance, AppError> {
    let source = get_instance(state, instance_id)?;
    let new_id = Uuid::new_v4().to_string();
    let source_dir = get_instance_dir_with_base(&get_instances_base_dir(state), instance_id);
    let dest_dir = get_instance_dir_with_base(&get_instances_base_dir(state), &new_id);

    let settings = state.get_settings();
    let pool_enabled = settings.resource_pool.enabled;
    let link_strategy = settings.resource_pool.link_strategy;

    // Load source manifest to identify pooled content
    let source_manifest = manifest_service::load_manifest(state, instance_id).ok();

    // Create destination directory
    fs::create_dir_all(&dest_dir)?;

    // Copy the directory, but handle pooled content specially
    if pool_enabled {
        if let Some(ref manifest) = source_manifest {
            // Copy directory but skip pooled content files (we'll link them instead)
            copy_dir_with_pool_awareness(
                state,
                &source_dir,
                &dest_dir,
                manifest,
                instance_id,
                &new_id,
                link_strategy,
            )?;
        } else {
            // No manifest, just copy everything
            copy_dir_recursive(&source_dir, &dest_dir)?;
        }
    } else {
        // Pool disabled, just copy everything
        copy_dir_recursive(&source_dir, &dest_dir)?;
    }

    // Create new instance with updated metadata
    let new_instance = Instance {
        id: new_id.clone(),
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
        modpack_platform: source.modpack_platform,
        modpack_id: source.modpack_id,
        modpack_version_id: source.modpack_version_id,
        description: source.description,
        author: source.author,
    };

    save_instance(state, &new_instance)?;

    Ok(new_instance)
}

/// Copy a directory with pool awareness
///
/// For pooled content, creates links from the pool instead of copying.
fn copy_dir_with_pool_awareness(
    state: &AppState,
    src: &PathBuf,
    dst: &PathBuf,
    manifest: &crate::models::InstalledContentManifest,
    _source_instance_id: &str,
    new_instance_id: &str,
    link_strategy: crate::models::LinkStrategy,
) -> Result<(), AppError> {
    fs::create_dir_all(dst)?;

    let entries = fs::read_dir(src)?;

    for entry in entries {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            let dir_name = entry.file_name().to_string_lossy().to_string();

            // Check if this is a content directory we need to handle specially
            if dir_name == ".minecraft" {
                // Recursively handle .minecraft directory
                copy_dir_with_pool_awareness(
                    state,
                    &src_path,
                    &dst_path,
                    manifest,
                    _source_instance_id,
                    new_instance_id,
                    link_strategy,
                )?;
            } else if dir_name == "mods" || dir_name == "shaderpacks" || dir_name == "resourcepacks"
            {
                // Handle content directories with pool awareness
                copy_content_dir_with_pool(
                    state,
                    &src_path,
                    &dst_path,
                    manifest,
                    new_instance_id,
                    &dir_name,
                    link_strategy,
                )?;
            } else {
                // Regular directory, copy recursively
                copy_dir_recursive(&src_path, &dst_path)?;
            }
        } else {
            // Regular file, copy
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Copy a content directory (mods/shaderpacks/resourcepacks) with pool awareness
fn copy_content_dir_with_pool(
    state: &AppState,
    src: &PathBuf,
    dst: &PathBuf,
    manifest: &crate::models::InstalledContentManifest,
    new_instance_id: &str,
    dir_name: &str,
    link_strategy: crate::models::LinkStrategy,
) -> Result<(), AppError> {
    use crate::models::resource_pool::LinkStrategy;

    fs::create_dir_all(dst)?;

    // Determine content type from directory name
    let content_type = match dir_name {
        "mods" => ContentType::Mod,
        "shaderpacks" => ContentType::Shader,
        "resourcepacks" => ContentType::ResourcePack,
        _ => return copy_dir_recursive(src, dst),
    };

    // Get the appropriate content list from manifest
    let content_list = match content_type {
        ContentType::Mod => &manifest.mods,
        ContentType::Shader => &manifest.shaders,
        ContentType::ResourcePack => &manifest.resource_packs,
        ContentType::Datapack => &manifest.datapacks,
        ContentType::World => &manifest.worlds,
    };

    let entries = fs::read_dir(src)?;

    for entry in entries {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let filename = entry.file_name().to_string_lossy().to_string();

        if src_path.is_dir() {
            // Could be "disabled" subdirectory
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            // Check if this file is pooled
            let pooled_content = content_list
                .iter()
                .find(|c| c.filename == filename && c.is_pooled);

            if let Some(content) = pooled_content {
                if let Some(ref sha512) = content.sha512_hash {
                    // Create link from pool instead of copying
                    let link_strat = match link_strategy {
                        crate::models::LinkStrategy::Auto => LinkStrategy::Auto,
                        crate::models::LinkStrategy::HardLink => LinkStrategy::HardLink,
                        crate::models::LinkStrategy::Symlink => LinkStrategy::Symlink,
                        crate::models::LinkStrategy::Copy => LinkStrategy::Copy,
                    };

                    let result = resource_pool_service::link_to_instance(
                        state,
                        sha512,
                        &content_type,
                        new_instance_id,
                        &filename,
                        link_strat,
                    );

                    if result.is_err() {
                        // Fallback to copy if linking fails
                        fs::copy(&src_path, &dst_path)?;
                    }
                } else {
                    // No hash, copy the file
                    fs::copy(&src_path, &dst_path)?;
                }
            } else {
                // Not pooled, copy the file
                fs::copy(&src_path, &dst_path)?;
            }
        }
    }

    Ok(())
}

/// Update instance's last played time and increment play time
pub fn update_play_time(
    state: &AppState,
    instance_id: &str,
    session_duration_secs: u64,
) -> Result<Instance, AppError> {
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

/// Emit instance setup status event
fn emit_setup_status(app_handle: &AppHandle, instance_id: &str, status: InstanceSetupStatus) {
    #[derive(serde::Serialize, Clone)]
    struct SetupStatusEvent {
        instance_id: String,
        status: InstanceSetupStatus,
    }

    let _ = app_handle.emit(
        "instance_setup_status",
        SetupStatusEvent {
            instance_id: instance_id.to_string(),
            status,
        },
    );
}

/// Setup an instance by downloading game files
/// This should be called after instance creation (and loader installation if applicable)
pub async fn setup_instance(
    state: &AppState,
    instance_id: &str,
    app_handle: &AppHandle,
) -> Result<(), AppError> {
    // Load instance
    let instance = get_instance(state, instance_id)?;
    let game_dir = get_game_directory(state, instance_id);

    // Emit preparing status
    emit_setup_status(
        app_handle,
        instance_id,
        InstanceSetupStatus::Preparing {
            message: "Loading version info...".to_string(),
        },
    );

    // Get version info (with loader support if applicable)
    let version_info = download_service::get_version_info_with_loader(
        &state.http_client,
        &instance.minecraft_version,
        &instance.loader_type,
        instance.loader_version.as_deref(),
        &game_dir,
    )
    .await?;

    // Emit downloading status - initial state
    emit_setup_status(
        app_handle,
        instance_id,
        InstanceSetupStatus::DownloadingGameFiles {
            progress: DownloadProgress::default(),
        },
    );

    // Download game files using the merged version info
    // The download_service will emit download_progress events, but we also want
    // to emit instance_setup_status events. We'll listen for download_progress
    // events in the frontend and update the setup status accordingly.
    download_service::download_game_files_with_version(
        instance_id,
        &instance.minecraft_version,
        &version_info,
        Some(app_handle),
    )
    .await?;

    // Emit complete status
    emit_setup_status(app_handle, instance_id, InstanceSetupStatus::Complete);

    Ok(())
}
