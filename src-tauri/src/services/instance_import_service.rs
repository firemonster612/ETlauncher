use crate::app_error;
use crate::error::AppError;
use crate::models::{Instance, LoaderType};
use crate::services::{
    content_scan_service,
    instance_service::{self, get_random_entity_icon, get_used_icons},
    loader_service,
};
use crate::state::AppState;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;
use zip::ZipArchive;

/// Type of import source
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ImportSourceType {
    /// Vanilla .minecraft folder
    VanillaMinecraft,
    /// MultiMC instance folder
    MultiMC,
    /// Prism Launcher instance folder (same format as MultiMC)
    PrismLauncher,
    /// CurseForge modpack .zip file
    CurseForgeZip,
}

/// Analysis result from examining an import source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportAnalysis {
    /// Detected Minecraft version
    pub minecraft_version: Option<String>,
    /// Detected loader type
    pub loader_type: LoaderType,
    /// Detected loader version
    pub loader_version: Option<String>,
    /// Number of mods found
    pub mod_count: usize,
    /// Whether resourcepacks folder has content
    pub has_resourcepacks: bool,
    /// Whether shaderpacks folder has content
    pub has_shaderpacks: bool,
    /// Whether config folder has content
    pub has_config: bool,
    /// The detected source type
    pub source_type: ImportSourceType,
    /// Suggested name for the instance
    pub suggested_name: Option<String>,
}

/// Progress event for import operations
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportProgress {
    pub stage: String,
    pub progress: u32,
    pub current_item: Option<String>,
}

fn emit_progress(
    app_handle: Option<&AppHandle>,
    stage: &str,
    progress: u32,
    current_item: Option<String>,
) {
    if let Some(handle) = app_handle {
        let _ = handle.emit(
            "import_progress",
            ImportProgress {
                stage: stage.to_string(),
                progress,
                current_item,
            },
        );
    }
}

/// Detect the type of import source from a path
pub fn detect_source_type(path: &Path) -> Option<ImportSourceType> {
    // Check if it's a zip file (CurseForge modpack)
    if path.is_file() {
        if let Some(ext) = path.extension() {
            if ext == "zip" {
                // Verify it contains manifest.json
                if let Ok(file) = File::open(path) {
                    if let Ok(mut archive) = ZipArchive::new(file) {
                        if archive.by_name("manifest.json").is_ok() {
                            return Some(ImportSourceType::CurseForgeZip);
                        }
                    }
                }
            }
        }
        return None;
    }

    // It's a directory - check for various markers
    if !path.is_dir() {
        return None;
    }

    // Check for MultiMC/Prism instance markers
    if path.join("mmc-pack.json").exists() || path.join("instance.cfg").exists() {
        // Distinguish between MultiMC and Prism by checking for specific files
        // Both use the same format, so we'll just return MultiMC for now
        // (PrismLauncher is a fork with same format)
        if path.join(".minecraft").exists() {
            return Some(ImportSourceType::MultiMC);
        }
        // Some instances have the minecraft dir directly
        if path.join("minecraft").exists() {
            return Some(ImportSourceType::MultiMC);
        }
        // Prism sometimes uses different folder structure
        return Some(ImportSourceType::PrismLauncher);
    }

    // Check for vanilla .minecraft markers
    if path.join("versions").exists() || path.join("launcher_profiles.json").exists() {
        return Some(ImportSourceType::VanillaMinecraft);
    }

    // Check if it's a .minecraft folder that might just have mods
    if path.join("mods").exists()
        && (path.join("config").exists() || path.join("options.txt").exists())
    {
        return Some(ImportSourceType::VanillaMinecraft);
    }

    None
}

/// Analyze an import source to extract metadata
pub fn analyze_import_source(path: &Path) -> Result<ImportAnalysis, AppError> {
    let source_type = detect_source_type(path)
        .ok_or_else(|| {
            // Provide detailed error message about what we checked
            let is_file = path.is_file();
            let is_dir = path.is_dir();
            let exists = path.exists();

            if !exists {
                return AppError::InvalidInput(format!(
                    "Path does not exist: {}",
                    path.display()
                ));
            }

            if is_file {
                let ext = path.extension().map(|e| e.to_string_lossy().to_string());
                return AppError::InvalidInput(format!(
                    "File '{}' is not a recognized modpack format. Extension: {:?}. Expected .zip with manifest.json inside.",
                    path.display(),
                    ext
                ));
            }

            if is_dir {
                let has_mmc_pack = path.join("mmc-pack.json").exists();
                let has_instance_cfg = path.join("instance.cfg").exists();
                let has_versions = path.join("versions").exists();
                let has_launcher_profiles = path.join("launcher_profiles.json").exists();
                let has_mods = path.join("mods").exists();
                let has_dot_minecraft = path.join(".minecraft").exists();
                let has_minecraft = path.join("minecraft").exists();

                return AppError::InvalidInput(format!(
                    "Folder '{}' is not a recognized instance format. Found: mmc-pack.json={}, instance.cfg={}, versions={}, launcher_profiles.json={}, mods={}, .minecraft={}, minecraft={}",
                    path.display(),
                    has_mmc_pack, has_instance_cfg, has_versions, has_launcher_profiles, has_mods, has_dot_minecraft, has_minecraft
                ));
            }

            AppError::InvalidInput(format!(
                "Could not determine import source type for: {}",
                path.display()
            ))
        })?;

    match source_type {
        ImportSourceType::VanillaMinecraft => analyze_vanilla_minecraft(path),
        ImportSourceType::MultiMC | ImportSourceType::PrismLauncher => {
            analyze_multimc_prism(path, source_type)
        }
        ImportSourceType::CurseForgeZip => analyze_curseforge_zip(path),
    }
}

/// Analyze a vanilla .minecraft folder
fn analyze_vanilla_minecraft(path: &Path) -> Result<ImportAnalysis, AppError> {
    let mut analysis = ImportAnalysis {
        minecraft_version: None,
        loader_type: LoaderType::Vanilla,
        loader_version: None,
        mod_count: 0,
        has_resourcepacks: false,
        has_shaderpacks: false,
        has_config: false,
        source_type: ImportSourceType::VanillaMinecraft,
        suggested_name: None,
    };

    // Try to detect MC version from launcher_profiles.json
    let launcher_profiles_path = path.join("launcher_profiles.json");
    if launcher_profiles_path.exists() {
        if let Ok(content) = fs::read_to_string(&launcher_profiles_path) {
            if let Ok(profiles) = serde_json::from_str::<LauncherProfiles>(&content) {
                // Try to get the selected profile's version
                if let Some(selected_profile) = profiles.selected_profile {
                    if let Some(profile) = profiles.profiles.get(&selected_profile) {
                        if let Some(ref version) = profile.last_version_id {
                            // Parse the version string - it might contain loader info
                            let (mc_ver, loader, loader_ver) = parse_version_id(version);
                            analysis.minecraft_version = mc_ver;
                            analysis.loader_type = loader;
                            analysis.loader_version = loader_ver;
                        }
                    }
                }
                // If no selected profile, try the first one
                if analysis.minecraft_version.is_none() {
                    for profile in profiles.profiles.values() {
                        if let Some(ref version) = profile.last_version_id {
                            let (mc_ver, loader, loader_ver) = parse_version_id(version);
                            analysis.minecraft_version = mc_ver;
                            analysis.loader_type = loader;
                            analysis.loader_version = loader_ver;
                            break;
                        }
                    }
                }
            }
        }
    }

    // If we couldn't get version from profiles, check versions folder
    if analysis.minecraft_version.is_none() {
        let versions_dir = path.join("versions");
        if versions_dir.exists() {
            // Look for installed versions
            if let Ok(entries) = fs::read_dir(&versions_dir) {
                for entry in entries.flatten() {
                    let version_name = entry.file_name().to_string_lossy().to_string();
                    let (mc_ver, loader, loader_ver) = parse_version_id(&version_name);
                    if mc_ver.is_some() {
                        analysis.minecraft_version = mc_ver;
                        analysis.loader_type = loader;
                        analysis.loader_version = loader_ver;
                        break;
                    }
                }
            }
        }
    }

    // Count mods
    let mods_dir = path.join("mods");
    if mods_dir.exists() {
        analysis.mod_count = count_files_with_extension(&mods_dir, &["jar"]);
    }

    // Check for resourcepacks
    let resourcepacks_dir = path.join("resourcepacks");
    analysis.has_resourcepacks = resourcepacks_dir.exists() && has_files(&resourcepacks_dir);

    // Check for shaderpacks
    let shaderpacks_dir = path.join("shaderpacks");
    analysis.has_shaderpacks = shaderpacks_dir.exists() && has_files(&shaderpacks_dir);

    // Check for config
    let config_dir = path.join("config");
    analysis.has_config = config_dir.exists() && has_files(&config_dir);

    // Suggest a name based on the folder name
    analysis.suggested_name = path.file_name().map(|n| n.to_string_lossy().to_string());

    Ok(analysis)
}

/// Analyze a MultiMC/Prism instance folder
fn analyze_multimc_prism(
    path: &Path,
    source_type: ImportSourceType,
) -> Result<ImportAnalysis, AppError> {
    let mut analysis = ImportAnalysis {
        minecraft_version: None,
        loader_type: LoaderType::Vanilla,
        loader_version: None,
        mod_count: 0,
        has_resourcepacks: false,
        has_shaderpacks: false,
        has_config: false,
        source_type,
        suggested_name: None,
    };

    // Parse mmc-pack.json for version and loader info
    let mmc_pack_path = path.join("mmc-pack.json");
    if mmc_pack_path.exists() {
        let content = fs::read_to_string(&mmc_pack_path)
            .map_err(|e| AppError::InvalidInput(format!("Could not read mmc-pack.json: {}", e)))?;

        let mmc_pack: MmcPack = serde_json::from_str(&content)
            .map_err(|e| AppError::InvalidInput(format!("Could not parse mmc-pack.json: {}", e)))?;

        for component in &mmc_pack.components {
            match component.uid.as_str() {
                "net.minecraft" => {
                    analysis.minecraft_version = Some(component.version.clone());
                }
                "net.fabricmc.fabric-loader" => {
                    analysis.loader_type = LoaderType::Fabric;
                    analysis.loader_version = Some(component.version.clone());
                }
                "net.minecraftforge" => {
                    analysis.loader_type = LoaderType::Forge;
                    analysis.loader_version = Some(component.version.clone());
                }
                "net.neoforged.neoforge" => {
                    analysis.loader_type = LoaderType::NeoForge;
                    analysis.loader_version = Some(component.version.clone());
                }
                "org.quiltmc.quilt-loader" => {
                    analysis.loader_type = LoaderType::Quilt;
                    analysis.loader_version = Some(component.version.clone());
                }
                _ => {}
            }
        }

        // If no Minecraft version found in components, return an error with debug info
        if analysis.minecraft_version.is_none() {
            let component_uids: Vec<&str> =
                mmc_pack.components.iter().map(|c| c.uid.as_str()).collect();
            return Err(AppError::InvalidInput(format!(
                "mmc-pack.json does not contain net.minecraft component. Found components: {:?}",
                component_uids
            )));
        }
    } else {
        return Err(AppError::InvalidInput(format!(
            "No mmc-pack.json found at {}. This may not be a valid MultiMC/Prism instance folder.",
            path.display()
        )));
    }

    // Parse instance.cfg for the instance name
    let instance_cfg_path = path.join("instance.cfg");
    if instance_cfg_path.exists() {
        if let Ok(content) = fs::read_to_string(&instance_cfg_path) {
            for line in content.lines() {
                if let Some(name) = line.strip_prefix("name=") {
                    analysis.suggested_name = Some(name.to_string());
                    break;
                }
            }
        }
    }

    // Find the minecraft directory (could be .minecraft or minecraft)
    let game_dir = if path.join(".minecraft").exists() {
        path.join(".minecraft")
    } else if path.join("minecraft").exists() {
        path.join("minecraft")
    } else {
        path.to_path_buf()
    };

    // Count mods
    let mods_dir = game_dir.join("mods");
    if mods_dir.exists() {
        analysis.mod_count = count_files_with_extension(&mods_dir, &["jar"]);
    }

    // Check for resourcepacks
    let resourcepacks_dir = game_dir.join("resourcepacks");
    analysis.has_resourcepacks = resourcepacks_dir.exists() && has_files(&resourcepacks_dir);

    // Check for shaderpacks
    let shaderpacks_dir = game_dir.join("shaderpacks");
    analysis.has_shaderpacks = shaderpacks_dir.exists() && has_files(&shaderpacks_dir);

    // Check for config
    let config_dir = game_dir.join("config");
    analysis.has_config = config_dir.exists() && has_files(&config_dir);

    // Fall back to folder name for suggested name
    if analysis.suggested_name.is_none() {
        analysis.suggested_name = path.file_name().map(|n| n.to_string_lossy().to_string());
    }

    Ok(analysis)
}

/// Analyze a CurseForge modpack zip file
fn analyze_curseforge_zip(path: &Path) -> Result<ImportAnalysis, AppError> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    // Read manifest.json
    let manifest: CfManifest = {
        let mut manifest_file = archive.by_name("manifest.json")?;
        let mut contents = String::new();
        manifest_file.read_to_string(&mut contents)?;
        serde_json::from_str(&contents)?
    };

    // Parse loader from manifest
    let (loader_type, loader_version) = if let Some(loader) = manifest.minecraft.mod_loaders.first()
    {
        parse_curseforge_loader_id(&loader.id)
    } else {
        (LoaderType::Vanilla, None)
    };

    let mut analysis = ImportAnalysis {
        minecraft_version: Some(manifest.minecraft.version),
        loader_type,
        loader_version,
        mod_count: manifest.files.len(),
        has_resourcepacks: false,
        has_shaderpacks: false,
        has_config: false,
        source_type: ImportSourceType::CurseForgeZip,
        suggested_name: Some(manifest.name),
    };

    // Check overrides for resourcepacks, shaderpacks, config
    let overrides_prefix = manifest.overrides.as_deref().unwrap_or("overrides");
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            let name = file.name();
            if name.starts_with(&format!("{}/resourcepacks/", overrides_prefix)) {
                analysis.has_resourcepacks = true;
            } else if name.starts_with(&format!("{}/shaderpacks/", overrides_prefix)) {
                analysis.has_shaderpacks = true;
            } else if name.starts_with(&format!("{}/config/", overrides_prefix)) {
                analysis.has_config = true;
            }
        }
    }

    Ok(analysis)
}

/// Import a vanilla .minecraft folder as a new instance
pub async fn import_vanilla_minecraft(
    state: &AppState,
    source_path: &Path,
    instance_name: String,
    app_handle: Option<&AppHandle>,
) -> Result<Instance, AppError> {
    // Register task in the task registry
    let task_id = Uuid::new_v4().to_string();
    state.task_registry.register(
        task_id.clone(),
        crate::task_registry::TaskType::InstanceImport,
        "Importing from vanilla Minecraft".to_string(),
        None,
        None,
    );
    state.task_registry.start(&task_id);

    let result: Result<Instance, AppError> = async {
        emit_progress(app_handle, "Analyzing source", 0, None);
        state
            .task_registry
            .update_stage(&task_id, "Analyzing source".to_string());

        // Analyze the source to get version info
        let analysis = analyze_vanilla_minecraft(source_path)?;

        let mc_version = analysis.minecraft_version.ok_or_else(|| {
            AppError::InvalidInput("Could not detect Minecraft version".to_string())
        })?;

        emit_progress(app_handle, "Creating instance", 10, None);
        state
            .task_registry
            .update_stage(&task_id, "Creating instance".to_string());

        // Create instance directories
        let instance_id = Uuid::new_v4().to_string();
        let instances_base = state.settings.read().instances_path.clone();
        let instance_dir =
            crate::utils::paths::get_instance_dir_with_base(&instances_base, &instance_id);
        let game_dir =
            crate::utils::paths::get_instance_game_dir_with_base(&instances_base, &instance_id);

        fs::create_dir_all(&instance_dir)?;
        fs::create_dir_all(&game_dir)?;

        // Create standard game subdirectories
        for subdir in [
            "mods",
            "resourcepacks",
            "saves",
            "screenshots",
            "logs",
            "config",
            "shaderpacks",
        ] {
            fs::create_dir_all(game_dir.join(subdir))?;
        }

        // Copy game content
        emit_progress(app_handle, "Copying mods", 20, None);
        state
            .task_registry
            .update_stage(&task_id, "Copying files".to_string());
        copy_directory_if_exists(&source_path.join("mods"), &game_dir.join("mods"))?;

        emit_progress(app_handle, "Copying resourcepacks", 40, None);
        copy_directory_if_exists(
            &source_path.join("resourcepacks"),
            &game_dir.join("resourcepacks"),
        )?;

        emit_progress(app_handle, "Copying shaderpacks", 50, None);
        copy_directory_if_exists(
            &source_path.join("shaderpacks"),
            &game_dir.join("shaderpacks"),
        )?;

        emit_progress(app_handle, "Copying config", 60, None);
        copy_directory_if_exists(&source_path.join("config"), &game_dir.join("config"))?;

        emit_progress(app_handle, "Copying saves", 70, None);
        copy_directory_if_exists(&source_path.join("saves"), &game_dir.join("saves"))?;

        // Install loader if needed
        if analysis.loader_type != LoaderType::Vanilla {
            if let Some(ref loader_version) = analysis.loader_version {
                emit_progress(
                    app_handle,
                    "Installing loader",
                    80,
                    Some(format!("{:?} {}", analysis.loader_type, loader_version)),
                );
                state
                    .task_registry
                    .update_stage(&task_id, "Installing loader".to_string());
                loader_service::install_loader(
                    &game_dir,
                    analysis.loader_type,
                    &mc_version,
                    loader_version,
                    |_, _| {},
                )
                .await?;
            }
        }

        emit_progress(app_handle, "Finalizing", 90, None);
        state
            .task_registry
            .update_stage(&task_id, "Finalizing".to_string());

        // Create instance
        let used_icons = get_used_icons(state);
        let instance = Instance {
            id: instance_id.clone(),
            name: instance_name,
            minecraft_version: mc_version,
            loader_type: analysis.loader_type,
            loader_version: analysis.loader_version,
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

        instance_service::save_instance(state, &instance)?;

        // Scan and identify imported content so the upgrade system can track it
        state
            .task_registry
            .update_stage(&task_id, "Scanning content".to_string());
        if let Err(e) = scan_and_identify_imported_content(state, &instance_id, app_handle).await {
            app_error!(
                "[import] Warning: Failed to identify imported content: {}",
                e
            );
            // Non-fatal: instance is still usable, content just won't be tracked for upgrades
        }

        emit_progress(app_handle, "Import complete", 100, None);

        Ok(instance)
    }
    .await;

    match result {
        Ok(instance) => {
            state.task_registry.complete(&task_id);
            Ok(instance)
        }
        Err(e) => {
            state.task_registry.fail(&task_id, e.to_string());
            Err(e)
        }
    }
}

/// Import a MultiMC/Prism instance as a new instance
pub async fn import_multimc_prism(
    state: &AppState,
    source_path: &Path,
    instance_name: String,
    app_handle: Option<&AppHandle>,
) -> Result<Instance, AppError> {
    // Register task in the task registry
    let task_id = Uuid::new_v4().to_string();
    state.task_registry.register(
        task_id.clone(),
        crate::task_registry::TaskType::InstanceImport,
        "Importing from MultiMC/Prism".to_string(),
        None,
        None,
    );
    state.task_registry.start(&task_id);

    let result: Result<Instance, AppError> = async {
        emit_progress(app_handle, "Analyzing source", 0, None);
        state
            .task_registry
            .update_stage(&task_id, "Analyzing source".to_string());

        // Analyze the source to get version info
        let source_type = detect_source_type(source_path).ok_or_else(|| {
            AppError::InvalidInput("Not a valid MultiMC/Prism instance".to_string())
        })?;

        let analysis = analyze_multimc_prism(source_path, source_type)?;

        let mc_version = analysis.minecraft_version.ok_or_else(|| {
            AppError::InvalidInput("Could not detect Minecraft version".to_string())
        })?;

        emit_progress(app_handle, "Creating instance", 10, None);
        state
            .task_registry
            .update_stage(&task_id, "Creating instance".to_string());

        // Create instance directories
        let instance_id = Uuid::new_v4().to_string();
        let instances_base = state.settings.read().instances_path.clone();
        let instance_dir =
            crate::utils::paths::get_instance_dir_with_base(&instances_base, &instance_id);
        let game_dir =
            crate::utils::paths::get_instance_game_dir_with_base(&instances_base, &instance_id);

        fs::create_dir_all(&instance_dir)?;
        fs::create_dir_all(&game_dir)?;

        // Find the source game directory
        let source_game_dir = if source_path.join(".minecraft").exists() {
            source_path.join(".minecraft")
        } else if source_path.join("minecraft").exists() {
            source_path.join("minecraft")
        } else {
            source_path.to_path_buf()
        };

        // Create standard game subdirectories
        for subdir in [
            "mods",
            "resourcepacks",
            "saves",
            "screenshots",
            "logs",
            "config",
            "shaderpacks",
        ] {
            fs::create_dir_all(game_dir.join(subdir))?;
        }

        // Copy game content
        emit_progress(app_handle, "Copying mods", 20, None);
        state
            .task_registry
            .update_stage(&task_id, "Copying files".to_string());
        copy_directory_if_exists(&source_game_dir.join("mods"), &game_dir.join("mods"))?;

        emit_progress(app_handle, "Copying resourcepacks", 40, None);
        copy_directory_if_exists(
            &source_game_dir.join("resourcepacks"),
            &game_dir.join("resourcepacks"),
        )?;

        emit_progress(app_handle, "Copying shaderpacks", 50, None);
        copy_directory_if_exists(
            &source_game_dir.join("shaderpacks"),
            &game_dir.join("shaderpacks"),
        )?;

        emit_progress(app_handle, "Copying config", 60, None);
        copy_directory_if_exists(&source_game_dir.join("config"), &game_dir.join("config"))?;

        emit_progress(app_handle, "Copying saves", 70, None);
        copy_directory_if_exists(&source_game_dir.join("saves"), &game_dir.join("saves"))?;

        // Install loader if needed
        if analysis.loader_type != LoaderType::Vanilla {
            if let Some(ref loader_version) = analysis.loader_version {
                emit_progress(
                    app_handle,
                    "Installing loader",
                    80,
                    Some(format!("{:?} {}", analysis.loader_type, loader_version)),
                );
                state
                    .task_registry
                    .update_stage(&task_id, "Installing loader".to_string());
                loader_service::install_loader(
                    &game_dir,
                    analysis.loader_type,
                    &mc_version,
                    loader_version,
                    |_, _| {},
                )
                .await?;
            }
        }

        emit_progress(app_handle, "Finalizing", 90, None);
        state
            .task_registry
            .update_stage(&task_id, "Finalizing".to_string());

        // Create instance
        let used_icons = get_used_icons(state);
        let instance = Instance {
            id: instance_id.clone(),
            name: instance_name,
            minecraft_version: mc_version,
            loader_type: analysis.loader_type,
            loader_version: analysis.loader_version,
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

        instance_service::save_instance(state, &instance)?;

        // Scan and identify imported content so the upgrade system can track it
        state
            .task_registry
            .update_stage(&task_id, "Scanning content".to_string());
        if let Err(e) = scan_and_identify_imported_content(state, &instance_id, app_handle).await {
            app_error!(
                "[import] Warning: Failed to identify imported content: {}",
                e
            );
            // Non-fatal: instance is still usable, content just won't be tracked for upgrades
        }

        emit_progress(app_handle, "Import complete", 100, None);

        Ok(instance)
    }
    .await;

    match result {
        Ok(instance) => {
            state.task_registry.complete(&task_id);
            Ok(instance)
        }
        Err(e) => {
            state.task_registry.fail(&task_id, e.to_string());
            Err(e)
        }
    }
}

/// Scan and identify imported content via the shared rescan service, then populate the manifest.
async fn scan_and_identify_imported_content(
    state: &AppState,
    instance_id: &str,
    _app_handle: Option<&AppHandle>,
) -> Result<(), AppError> {
    content_scan_service::rescan_and_rebuild_manifest(state, instance_id).await?;
    Ok(())
}

/// Import a CurseForge modpack .zip file
/// This function delegates to the existing modpack_install_service for the actual work
pub async fn import_curseforge_zip(
    state: &AppState,
    file_path: &Path,
    instance_name: Option<String>,
    app_handle: Option<&AppHandle>,
) -> Result<Instance, AppError> {
    use crate::services::modpack_install_service;

    // Register task in the task registry
    let task_id = Uuid::new_v4().to_string();
    state.task_registry.register(
        task_id.clone(),
        crate::task_registry::TaskType::InstanceImport,
        "Importing CurseForge modpack".to_string(),
        None,
        None,
    );
    state.task_registry.start(&task_id);

    let result: Result<Instance, AppError> = async {
        emit_progress(app_handle, "Analyzing modpack", 0, None);
        state
            .task_registry
            .update_stage(&task_id, "Analyzing modpack".to_string());

        // First, analyze to get the suggested name if not provided
        let analysis = analyze_curseforge_zip(file_path)?;
        let final_name = instance_name.unwrap_or_else(|| {
            analysis
                .suggested_name
                .unwrap_or_else(|| "Imported Modpack".to_string())
        });

        state
            .task_registry
            .update_stage(&task_id, "Installing modpack".to_string());

        // Use the existing import function from modpack_install_service
        modpack_install_service::import_curseforge_zip_file(
            state,
            file_path,
            Some(final_name),
            app_handle,
            None, // No cancel token for now
        )
        .await
    }
    .await;

    match result {
        Ok(instance) => {
            state.task_registry.complete(&task_id);
            Ok(instance)
        }
        Err(e) => {
            state.task_registry.fail(&task_id, e.to_string());
            Err(e)
        }
    }
}

// === Helper Types ===

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LauncherProfiles {
    #[serde(default)]
    selected_profile: Option<String>,
    #[serde(default)]
    profiles: std::collections::HashMap<String, LauncherProfile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LauncherProfile {
    #[serde(default)]
    last_version_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MmcPack {
    #[serde(default)]
    components: Vec<MmcComponent>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MmcComponent {
    uid: String,
    version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfManifest {
    minecraft: CfMinecraft,
    name: String,
    #[serde(default)]
    files: Vec<CfFile>,
    overrides: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfMinecraft {
    version: String,
    #[serde(default)]
    mod_loaders: Vec<CfModLoader>,
}

#[derive(Debug, Deserialize)]
struct CfModLoader {
    id: String,
}

#[derive(Debug, Deserialize)]
struct CfFile {
    #[allow(dead_code)]
    #[serde(rename = "projectID")]
    project_id: u32,
    #[allow(dead_code)]
    #[serde(rename = "fileID")]
    file_id: u32,
}

// === Helper Functions ===

/// Parse a version ID string to extract MC version, loader type, and loader version
fn parse_version_id(version_id: &str) -> (Option<String>, LoaderType, Option<String>) {
    // Common patterns:
    // fabric-loader-0.14.21-1.20.1
    // 1.20.1-forge-47.1.0
    // neoforge-20.4.0
    // quilt-loader-0.19.0-1.20.1
    // 1.20.1

    let lower = version_id.to_lowercase();

    // Fabric pattern: fabric-loader-{version}-{mc_version}
    if lower.contains("fabric-loader-") {
        let parts: Vec<&str> = version_id.split('-').collect();
        if parts.len() >= 4 {
            // fabric-loader-0.14.21-1.20.1
            let loader_version = parts[2].to_string();
            let mc_version = parts[3..].join("-");
            return (Some(mc_version), LoaderType::Fabric, Some(loader_version));
        }
    }

    // Quilt pattern: quilt-loader-{version}-{mc_version}
    if lower.contains("quilt-loader-") {
        let parts: Vec<&str> = version_id.split('-').collect();
        if parts.len() >= 4 {
            let loader_version = parts[2].to_string();
            let mc_version = parts[3..].join("-");
            return (Some(mc_version), LoaderType::Quilt, Some(loader_version));
        }
    }

    // Forge pattern: {mc_version}-forge-{loader_version}
    if lower.contains("-forge-") {
        let parts: Vec<&str> = version_id.split("-forge-").collect();
        if parts.len() == 2 {
            return (
                Some(parts[0].to_string()),
                LoaderType::Forge,
                Some(parts[1].to_string()),
            );
        }
    }

    // NeoForge pattern: {mc_version}-neoforge-{loader_version}
    if lower.contains("-neoforge-") {
        let parts: Vec<&str> = version_id.split("-neoforge-").collect();
        if parts.len() == 2 {
            return (
                Some(parts[0].to_string()),
                LoaderType::NeoForge,
                Some(parts[1].to_string()),
            );
        }
    }

    // Plain MC version (e.g., 1.20.1)
    if version_id
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        // Looks like it starts with a number, probably MC version
        return (Some(version_id.to_string()), LoaderType::Vanilla, None);
    }

    (None, LoaderType::Vanilla, None)
}

/// Parse a CurseForge loader ID (e.g., "forge-47.1.0", "fabric-0.14.21")
fn parse_curseforge_loader_id(loader_id: &str) -> (LoaderType, Option<String>) {
    let lower = loader_id.to_lowercase();

    if lower.starts_with("forge-") {
        return (LoaderType::Forge, Some(loader_id[6..].to_string()));
    }
    if lower.starts_with("fabric-") {
        return (LoaderType::Fabric, Some(loader_id[7..].to_string()));
    }
    if lower.starts_with("neoforge-") {
        return (LoaderType::NeoForge, Some(loader_id[9..].to_string()));
    }
    if lower.starts_with("quilt-") {
        return (LoaderType::Quilt, Some(loader_id[6..].to_string()));
    }

    (LoaderType::Vanilla, None)
}

/// Count files with specific extensions in a directory
fn count_files_with_extension(dir: &Path, extensions: &[&str]) -> usize {
    if !dir.exists() {
        return 0;
    }

    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if extensions.iter().any(|e| e.to_lowercase() == ext_str) {
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

/// Check if a directory has any files (not counting hidden files)
fn has_files(dir: &Path) -> bool {
    if !dir.exists() {
        return false;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            // Skip hidden files
            if !name_str.starts_with('.') {
                return true;
            }
        }
    }
    false
}

/// Copy a directory if it exists, otherwise do nothing
fn copy_directory_if_exists(src: &Path, dst: &Path) -> Result<(), AppError> {
    if !src.exists() {
        return Ok(());
    }

    copy_dir_recursive(src, dst)
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), AppError> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        // Skip hidden files
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
