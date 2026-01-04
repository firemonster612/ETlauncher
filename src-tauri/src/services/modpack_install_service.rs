use crate::error::AppError;
use crate::models::{
    ContentPlatform, ContentSource, ContentType, InstalledContent, InstalledContentManifest,
    Instance, LoaderType, ModpackPlatform, MANIFEST_VERSION,
};
use crate::services::{
    atlauncher_service, curseforge_service, ftb_service, instance_service, loader_service,
    manifest_service, modrinth_service, technic_service,
};
use crate::state::AppState;
use crate::utils::hash::{murmur2_bytes, sha512_file};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;
use zip::ZipArchive;

/// Progress event for modpack installation
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModpackInstallProgress {
    pub stage: String,
    pub progress: u32,
    pub current_item: Option<String>,
    pub total_items: u32,
    pub completed_items: u32,
}

/// Install a Modrinth modpack
pub async fn install_modrinth_modpack(
    state: &AppState,
    modpack_id: &str,
    version_id: &str,
    instance_name: Option<String>,
    app_handle: Option<&AppHandle>,
) -> Result<Instance, AppError> {
    println!("[modpack_install] install_modrinth_modpack: modpack_id={}, version_id={}", modpack_id, version_id);

    // Get modpack info
    println!("[modpack_install] Fetching modpack info...");
    let modpack = modrinth_service::get_modpack(&state.http_client, modpack_id).await?;
    println!("[modpack_install] Got modpack: {}", modpack.name);

    println!("[modpack_install] Fetching version info...");
    let version = modrinth_service::get_modpack_version(&state.http_client, version_id).await?;
    println!("[modpack_install] Got version: {}", version.name);

    emit_progress(app_handle, "Downloading modpack", 0, None, 0, 0);

    // Get the mrpack file URL
    let mrpack_file = version
        .files
        .iter()
        .find(|f| f.path.ends_with(".mrpack"))
        .or_else(|| version.files.first())
        .ok_or_else(|| AppError::ContentNotFound("No modpack file found".to_string()))?;

    // Download the mrpack file
    println!("[modpack_install] Downloading mrpack from: {}", mrpack_file.url);
    let mrpack_bytes = download_bytes(&state.http_client, &mrpack_file.url).await?;
    println!("[modpack_install] Downloaded {} bytes", mrpack_bytes.len());

    emit_progress(app_handle, "Extracting modpack", 10, None, 0, 0);

    // Parse the mrpack (it's a zip file with modrinth.index.json)
    let cursor = std::io::Cursor::new(&mrpack_bytes);
    let mut archive = ZipArchive::new(cursor)?;

    // Read modrinth.index.json
    let index: ModrinthIndex = {
        let mut index_file = archive.by_name("modrinth.index.json")?;
        let mut contents = String::new();
        index_file.read_to_string(&mut contents)?;
        serde_json::from_str(&contents)?
    };

    // Determine Minecraft version and loader
    let mc_version = index
        .dependencies
        .get("minecraft")
        .cloned()
        .unwrap_or_else(|| "1.20.1".to_string());

    let (loader_type, loader_version) = determine_loader(&index.dependencies);
    println!("[modpack_install] Parsed index: mc_version={}, loader={:?}, loader_version={:?}, files={}",
        mc_version, loader_type, loader_version, index.files.len());

    // Create instance
    let instance_name = instance_name.unwrap_or_else(|| modpack.name.clone());
    let instance_id = Uuid::new_v4().to_string();
    println!("[modpack_install] Creating instance: id={}, name={}", instance_id, instance_name);

    emit_progress(app_handle, "Creating instance", 15, None, 0, 0);

    // Get instance directories
    let instances_base = state.settings.read().instances_path.clone();
    let instance_dir = crate::utils::paths::get_instance_dir_with_base(&instances_base, &instance_id);
    let game_dir = crate::utils::paths::get_instance_game_dir_with_base(&instances_base, &instance_id);

    // Create directories
    fs::create_dir_all(&instance_dir)?;
    fs::create_dir_all(&game_dir)?;

    // Create standard game subdirectories
    for subdir in ["mods", "resourcepacks", "saves", "screenshots", "logs", "config", "shaderpacks"] {
        fs::create_dir_all(game_dir.join(subdir))?;
    }

    let instance = Instance {
        id: instance_id.clone(),
        name: instance_name,
        minecraft_version: mc_version.clone(),
        loader_type: loader_type.unwrap_or(LoaderType::Vanilla),
        loader_version: loader_version,
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
        modpack_platform: Some(ModpackPlatform::Modrinth),
        modpack_id: Some(modpack_id.to_string()),
        modpack_version_id: Some(version_id.to_string()),
    };

    instance_service::save_instance(state, &instance)?;

    // Extract overrides folder
    emit_progress(app_handle, "Extracting overrides", 20, None, 0, 0);
    extract_overrides(&mut archive, &game_dir, "overrides")?;
    extract_overrides(&mut archive, &game_dir, "client-overrides")?;

    // Download mod files
    let total_files = index.files.len() as u32;
    emit_progress(app_handle, "Downloading mods", 25, None, total_files, 0);

    for (i, file_entry) in index.files.iter().enumerate() {
        let filename = file_entry
            .path
            .split('/')
            .last()
            .unwrap_or(&file_entry.path);

        emit_progress(
            app_handle,
            "Downloading mods",
            25 + ((i as u32 * 70) / total_files.max(1)),
            Some(filename.to_string()),
            total_files,
            i as u32,
        );

        // Determine file destination
        let dest_path = game_dir.join(&file_entry.path);
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Try each download URL
        let mut downloaded = false;
        for url in &file_entry.downloads {
            match download_file_with_hash(
                &state.http_client,
                url,
                &dest_path,
                file_entry.hashes.get("sha1").map(|s| s.as_str()),
            )
            .await
            {
                Ok(_) => {
                    downloaded = true;
                    break;
                }
                Err(_) => continue,
            }
        }

        if !downloaded && !file_entry.downloads.is_empty() {
            eprintln!("Failed to download: {}", file_entry.path);
        }
    }

    // Install mod loader if not Vanilla
    if instance.loader_type != LoaderType::Vanilla {
        if let Some(ref lv) = instance.loader_version {
            emit_progress(
                app_handle,
                "Installing mod loader",
                95,
                Some(format!("{:?} {}", instance.loader_type, lv)),
                0,
                0,
            );

            loader_service::install_loader(
                &game_dir,
                instance.loader_type.clone(),
                &mc_version,
                lv,
                |msg, pct| {
                    emit_progress(
                        app_handle,
                        &format!("Loader: {}", msg),
                        95 + (pct / 20),
                        None,
                        0,
                        0,
                    );
                },
            )
            .await?;
        }
    }

    emit_progress(app_handle, "Installation complete", 100, None, total_files, total_files);

    // Create manifest and mark all content as modpack-original
    if let Err(e) = create_modpack_manifest(state, &instance.id, &game_dir) {
        eprintln!("[modpack_install] Warning: Failed to create manifest: {}", e);
    }

    println!("[modpack_install] Modrinth modpack installed successfully: instance_id={}", instance.id);
    Ok(instance)
}

/// Install a CurseForge modpack
pub async fn install_curseforge_modpack(
    state: &AppState,
    modpack_id: &str,
    version_id: &str,
    instance_name: Option<String>,
    app_handle: Option<&AppHandle>,
) -> Result<Instance, AppError> {
    println!("[modpack_install] install_curseforge_modpack: modpack_id={}, version_id={}", modpack_id, version_id);

    let api_key = state
        .get_settings()
        .curseforge_api_key
        .ok_or_else(|| {
            println!("[modpack_install] CurseForge API key not configured");
            AppError::ApiError("CurseForge API key not configured".to_string())
        })?;

    // Get modpack info
    println!("[modpack_install] Fetching CurseForge modpack info...");
    let modpack = curseforge_service::get_modpack(&state.http_client, &api_key, modpack_id).await?;
    println!("[modpack_install] Got modpack: {}", modpack.name);

    // Get versions and find the specific one
    println!("[modpack_install] Fetching CurseForge versions...");
    let versions = curseforge_service::get_modpack_versions(&state.http_client, &api_key, modpack_id).await?;
    println!("[modpack_install] Found {} versions", versions.len());
    let version = versions
        .into_iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| AppError::ContentNotFound(format!("Version {} not found", version_id)))?;

    emit_progress(app_handle, "Downloading modpack", 0, None, 0, 0);

    // Get the modpack zip URL
    let zip_file = version
        .files
        .first()
        .ok_or_else(|| AppError::ContentNotFound("No modpack file found".to_string()))?;

    // Download the modpack zip
    let zip_bytes = download_bytes(&state.http_client, &zip_file.url).await?;

    emit_progress(app_handle, "Extracting modpack", 10, None, 0, 0);

    // Parse the zip (contains manifest.json)
    let cursor = std::io::Cursor::new(&zip_bytes);
    let mut archive = ZipArchive::new(cursor)?;

    // Read manifest.json
    let manifest: CurseForgeManifest = {
        let mut manifest_file = archive.by_name("manifest.json")?;
        let mut contents = String::new();
        manifest_file.read_to_string(&mut contents)?;
        serde_json::from_str(&contents)?
    };

    // Determine Minecraft version and loader
    let mc_version = manifest.minecraft.version.clone();
    let (loader_type, loader_version) = if let Some(loader) = manifest.minecraft.mod_loaders.first()
    {
        parse_curseforge_loader(&loader.id)
    } else {
        (None, None)
    };

    // Create instance
    let instance_name = instance_name.unwrap_or_else(|| modpack.name.clone());
    let instance_id = Uuid::new_v4().to_string();

    emit_progress(app_handle, "Creating instance", 15, None, 0, 0);

    // Get instance directories
    let instances_base = state.settings.read().instances_path.clone();
    let instance_dir = crate::utils::paths::get_instance_dir_with_base(&instances_base, &instance_id);
    let game_dir = crate::utils::paths::get_instance_game_dir_with_base(&instances_base, &instance_id);

    // Create directories
    fs::create_dir_all(&instance_dir)?;
    fs::create_dir_all(&game_dir)?;

    // Create standard game subdirectories
    for subdir in ["mods", "resourcepacks", "saves", "screenshots", "logs", "config", "shaderpacks"] {
        fs::create_dir_all(game_dir.join(subdir))?;
    }

    let instance = Instance {
        id: instance_id.clone(),
        name: instance_name,
        minecraft_version: mc_version.clone(),
        loader_type: loader_type.unwrap_or(LoaderType::Vanilla),
        loader_version: loader_version,
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
        modpack_platform: Some(ModpackPlatform::CurseForge),
        modpack_id: Some(modpack_id.to_string()),
        modpack_version_id: Some(version_id.to_string()),
    };

    instance_service::save_instance(state, &instance)?;

    // Extract overrides folder
    emit_progress(app_handle, "Extracting overrides", 20, None, 0, 0);
    let overrides_folder = manifest.overrides.as_deref().unwrap_or("overrides");
    extract_overrides(&mut archive, &game_dir, overrides_folder)?;

    // Download mod files from CurseForge
    let total_files = manifest.files.len() as u32;
    emit_progress(app_handle, "Downloading mods", 25, None, total_files, 0);

    let mods_dir = game_dir.join("mods");
    fs::create_dir_all(&mods_dir)?;

    for (i, cf_file) in manifest.files.iter().enumerate() {
        emit_progress(
            app_handle,
            "Downloading mods",
            25 + ((i as u32 * 70) / total_files.max(1)),
            Some(format!("Mod {}/{}", i + 1, total_files)),
            total_files,
            i as u32,
        );

        // Get file info from CurseForge API
        let file_info = curseforge_service::get_mod_file(
            &state.http_client,
            &api_key,
            cf_file.project_id,
            cf_file.file_id,
        )
        .await;

        if let Ok(info) = file_info {
            let dest_path = mods_dir.join(&info.filename);
            let _ = download_bytes_to_file(&state.http_client, &info.download_url, &dest_path).await;
        }
    }

    // Install mod loader if not Vanilla
    if instance.loader_type != LoaderType::Vanilla {
        if let Some(ref lv) = instance.loader_version {
            emit_progress(
                app_handle,
                "Installing mod loader",
                95,
                Some(format!("{:?} {}", instance.loader_type, lv)),
                0,
                0,
            );

            loader_service::install_loader(
                &game_dir,
                instance.loader_type.clone(),
                &mc_version,
                lv,
                |msg, pct| {
                    emit_progress(
                        app_handle,
                        &format!("Loader: {}", msg),
                        95 + (pct / 20),
                        None,
                        0,
                        0,
                    );
                },
            )
            .await?;
        }
    }

    emit_progress(app_handle, "Installation complete", 100, None, total_files, total_files);

    // Create manifest and mark all content as modpack-original
    if let Err(e) = create_modpack_manifest(state, &instance.id, &game_dir) {
        eprintln!("[modpack_install] Warning: Failed to create manifest: {}", e);
    }

    println!("[modpack_install] CurseForge modpack installed successfully: instance_id={}", instance.id);
    Ok(instance)
}

// === Helper Types ===

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModrinthIndex {
    #[allow(dead_code)]
    format_version: u32,
    #[allow(dead_code)]
    game: String,
    #[allow(dead_code)]
    version_id: String,
    #[allow(dead_code)]
    name: String,
    #[serde(default)]
    dependencies: std::collections::HashMap<String, String>,
    #[serde(default)]
    files: Vec<ModrinthFileEntry>,
}

#[derive(Debug, Deserialize)]
struct ModrinthFileEntry {
    path: String,
    #[serde(default)]
    hashes: std::collections::HashMap<String, String>,
    #[serde(default)]
    downloads: Vec<String>,
    #[allow(dead_code)]
    #[serde(default)]
    file_size: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseForgeManifest {
    minecraft: CurseForgeMinecraft,
    #[allow(dead_code)]
    manifest_type: String,
    #[allow(dead_code)]
    manifest_version: u32,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    version: String,
    #[allow(dead_code)]
    author: String,
    #[serde(default)]
    files: Vec<CurseForgeFileEntry>,
    overrides: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseForgeMinecraft {
    version: String,
    #[serde(default)]
    mod_loaders: Vec<CurseForgeModLoader>,
}

#[derive(Debug, Deserialize)]
struct CurseForgeModLoader {
    id: String,
    #[allow(dead_code)]
    primary: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseForgeFileEntry {
    project_id: u32,
    file_id: u32,
    #[allow(dead_code)]
    required: bool,
}

// === Helper Functions ===

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
            "modpack_install_progress",
            ModpackInstallProgress {
                stage: stage.to_string(),
                progress,
                current_item,
                total_items,
                completed_items,
            },
        );
    }
}

fn determine_loader(
    dependencies: &std::collections::HashMap<String, String>,
) -> (Option<LoaderType>, Option<String>) {
    if let Some(version) = dependencies.get("fabric-loader") {
        return (Some(LoaderType::Fabric), Some(version.clone()));
    }
    if let Some(version) = dependencies.get("quilt-loader") {
        return (Some(LoaderType::Quilt), Some(version.clone()));
    }
    if let Some(version) = dependencies.get("forge") {
        return (Some(LoaderType::Forge), Some(version.clone()));
    }
    if let Some(version) = dependencies.get("neoforge") {
        return (Some(LoaderType::NeoForge), Some(version.clone()));
    }
    (None, None)
}

fn parse_curseforge_loader(loader_id: &str) -> (Option<LoaderType>, Option<String>) {
    if loader_id.starts_with("forge-") {
        let version = loader_id.strip_prefix("forge-").map(|s| s.to_string());
        return (Some(LoaderType::Forge), version);
    }
    if loader_id.starts_with("fabric-") {
        let version = loader_id.strip_prefix("fabric-").map(|s| s.to_string());
        return (Some(LoaderType::Fabric), version);
    }
    if loader_id.starts_with("neoforge-") {
        let version = loader_id.strip_prefix("neoforge-").map(|s| s.to_string());
        return (Some(LoaderType::NeoForge), version);
    }
    if loader_id.starts_with("quilt-") {
        let version = loader_id.strip_prefix("quilt-").map(|s| s.to_string());
        return (Some(LoaderType::Quilt), version);
    }
    (None, None)
}

fn extract_overrides<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    game_dir: &PathBuf,
    overrides_prefix: &str,
) -> Result<(), AppError> {
    let prefix_with_slash = format!("{}/", overrides_prefix);

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        if name.starts_with(&prefix_with_slash) {
            let relative_path = name.strip_prefix(&prefix_with_slash).unwrap_or(&name);
            if relative_path.is_empty() {
                continue;
            }

            let dest_path = game_dir.join(relative_path);

            if name.ends_with('/') {
                fs::create_dir_all(&dest_path)?;
            } else {
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = File::create(&dest_path)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }
    }

    Ok(())
}

async fn download_bytes(client: &Client, url: &str) -> Result<Vec<u8>, AppError> {
    let response = client.get(url).send().await.map_err(|e| {
        AppError::DownloadError(format!("Failed to fetch {}: {}", url, e))
    })?;

    if !response.status().is_success() {
        return Err(AppError::DownloadError(format!(
            "HTTP {} for {}",
            response.status(),
            url
        )));
    }

    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}

async fn download_bytes_to_file(
    client: &Client,
    url: &str,
    path: &PathBuf,
) -> Result<(), AppError> {
    let bytes = download_bytes(client, url).await?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = File::create(path)?;
    file.write_all(&bytes)?;

    Ok(())
}

async fn download_file_with_hash(
    client: &Client,
    url: &str,
    path: &PathBuf,
    expected_hash: Option<&str>,
) -> Result<(), AppError> {
    download_file_with_hash_algo(client, url, path, expected_hash, "sha1").await
}

async fn download_file_with_hash_algo(
    client: &Client,
    url: &str,
    path: &PathBuf,
    expected_hash: Option<&str>,
    hash_algo: &str,
) -> Result<(), AppError> {
    let bytes = download_bytes(client, url).await?;

    // Verify hash if provided
    if let Some(expected) = expected_hash {
        if !expected.is_empty() {
            let hash = match hash_algo {
                "md5" => {
                    let digest = md5::compute(&bytes);
                    format!("{:x}", digest)
                }
                _ => {
                    // Default to SHA1
                    let mut hasher = Sha1::new();
                    hasher.update(&bytes);
                    format!("{:x}", hasher.finalize())
                }
            };

            if hash != expected {
                eprintln!("[download] Hash mismatch for {}: expected={}, got={}", path.display(), expected, hash);
                return Err(AppError::HashMismatch(path.display().to_string()));
            }
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = File::create(path)?;
    file.write_all(&bytes)?;

    Ok(())
}

/// Import an instance from a local .mrpack file
pub async fn import_from_mrpack_file(
    state: &AppState,
    file_path: &str,
    instance_name: Option<String>,
    app_handle: Option<&AppHandle>,
) -> Result<Instance, AppError> {
    println!("[modpack_install] import_from_mrpack_file: file_path={}", file_path);

    emit_progress(app_handle, "Reading modpack file", 0, None, 0, 0);

    // Read the mrpack file from disk
    let mrpack_path = std::path::Path::new(file_path);
    if !mrpack_path.exists() {
        return Err(AppError::ContentNotFound(format!(
            "File not found: {}",
            file_path
        )));
    }

    let mrpack_file = File::open(mrpack_path)?;
    let mut archive = ZipArchive::new(mrpack_file)?;

    emit_progress(app_handle, "Extracting modpack", 10, None, 0, 0);

    // Read modrinth.index.json
    let index: ModrinthIndex = {
        let mut index_file = archive.by_name("modrinth.index.json")?;
        let mut contents = String::new();
        index_file.read_to_string(&mut contents)?;
        serde_json::from_str(&contents)?
    };

    // Determine Minecraft version and loader
    let mc_version = index
        .dependencies
        .get("minecraft")
        .cloned()
        .unwrap_or_else(|| "1.20.1".to_string());

    let (loader_type, loader_version) = determine_loader(&index.dependencies);
    println!(
        "[modpack_install] Parsed index: mc_version={}, loader={:?}, loader_version={:?}, files={}",
        mc_version, loader_type, loader_version, index.files.len()
    );

    // Create instance
    let pack_name = index.name.clone();
    let instance_name = instance_name.unwrap_or(pack_name);
    let instance_id = Uuid::new_v4().to_string();
    println!(
        "[modpack_install] Creating instance: id={}, name={}",
        instance_id, instance_name
    );

    emit_progress(app_handle, "Creating instance", 15, None, 0, 0);

    // Get instance directories
    let instances_base = state.settings.read().instances_path.clone();
    let instance_dir =
        crate::utils::paths::get_instance_dir_with_base(&instances_base, &instance_id);
    let game_dir =
        crate::utils::paths::get_instance_game_dir_with_base(&instances_base, &instance_id);

    // Create directories
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

    let instance = Instance {
        id: instance_id.clone(),
        name: instance_name,
        minecraft_version: mc_version.clone(),
        loader_type: loader_type.unwrap_or(LoaderType::Vanilla),
        loader_version,
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
        modpack_platform: None,
        modpack_id: None,
        modpack_version_id: None,
    };

    instance_service::save_instance(state, &instance)?;

    // Extract overrides folder
    emit_progress(app_handle, "Extracting overrides", 20, None, 0, 0);
    extract_overrides(&mut archive, &game_dir, "overrides")?;
    extract_overrides(&mut archive, &game_dir, "client-overrides")?;

    // Download mod files
    let total_files = index.files.len() as u32;
    emit_progress(app_handle, "Downloading mods", 25, None, total_files, 0);

    for (i, file_entry) in index.files.iter().enumerate() {
        let filename = file_entry
            .path
            .split('/')
            .last()
            .unwrap_or(&file_entry.path);

        emit_progress(
            app_handle,
            "Downloading mods",
            25 + ((i as u32 * 70) / total_files.max(1)),
            Some(filename.to_string()),
            total_files,
            i as u32,
        );

        // Determine file destination
        let dest_path = game_dir.join(&file_entry.path);
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Try each download URL
        let mut downloaded = false;
        for url in &file_entry.downloads {
            match download_file_with_hash(
                &state.http_client,
                url,
                &dest_path,
                file_entry.hashes.get("sha1").map(|s| s.as_str()),
            )
            .await
            {
                Ok(_) => {
                    downloaded = true;
                    break;
                }
                Err(_) => continue,
            }
        }

        if !downloaded && !file_entry.downloads.is_empty() {
            eprintln!("Failed to download: {}", file_entry.path);
        }
    }

    // Install mod loader if not Vanilla
    if instance.loader_type != LoaderType::Vanilla {
        if let Some(ref lv) = instance.loader_version {
            emit_progress(
                app_handle,
                "Installing mod loader",
                95,
                Some(format!("{:?} {}", instance.loader_type, lv)),
                0,
                0,
            );

            loader_service::install_loader(
                &game_dir,
                instance.loader_type.clone(),
                &mc_version,
                lv,
                |msg, pct| {
                    emit_progress(
                        app_handle,
                        &format!("Loader: {}", msg),
                        95 + (pct / 20),
                        None,
                        0,
                        0,
                    );
                },
            )
            .await?;
        }
    }

    emit_progress(
        app_handle,
        "Installation complete",
        100,
        None,
        total_files,
        total_files,
    );

    // Create manifest for imported modpack (no platform tracking since it's a local file)
    if let Err(e) = create_modpack_manifest(state, &instance.id, &game_dir) {
        eprintln!("[modpack_install] Warning: Failed to create manifest: {}", e);
    }

    println!(
        "[modpack_install] Imported mrpack successfully: instance_id={}",
        instance.id
    );
    Ok(instance)
}

/// Create a manifest for a modpack installation by scanning the content folders
/// All content is marked as ModpackOriginal since it came from the modpack
pub fn create_modpack_manifest(
    state: &AppState,
    instance_id: &str,
    game_dir: &PathBuf,
) -> Result<(), AppError> {
    let mut manifest = InstalledContentManifest {
        manifest_version: MANIFEST_VERSION,
        mods: Vec::new(),
        shaders: Vec::new(),
        resource_packs: Vec::new(),
        last_synced_at: Some(Utc::now().timestamp()),
    };

    // Scan mods folder
    let mods_dir = game_dir.join("mods");
    if mods_dir.exists() {
        manifest.mods = scan_content_folder(&mods_dir, ContentType::Mod)?;
    }

    // Scan shaders folder
    let shaders_dir = game_dir.join("shaderpacks");
    if shaders_dir.exists() {
        manifest.shaders = scan_content_folder(&shaders_dir, ContentType::Shader)?;
    }

    // Scan resource packs folder
    let resourcepacks_dir = game_dir.join("resourcepacks");
    if resourcepacks_dir.exists() {
        manifest.resource_packs = scan_content_folder(&resourcepacks_dir, ContentType::ResourcePack)?;
    }

    // Save the manifest
    manifest_service::save_manifest(state, instance_id, &manifest)?;

    println!(
        "[modpack_install] Created manifest: {} mods, {} shaders, {} resource packs",
        manifest.mods.len(),
        manifest.shaders.len(),
        manifest.resource_packs.len()
    );

    Ok(())
}

/// Scan a content folder and create InstalledContent entries for all files
/// All entries are marked as ModpackOriginal
fn scan_content_folder(folder: &PathBuf, content_type: ContentType) -> Result<Vec<InstalledContent>, AppError> {
    let mut content = Vec::new();

    let entries = fs::read_dir(folder)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Skip directories and non-jar/zip files
        if path.is_dir() {
            continue;
        }

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();

        // Skip hidden files and cache files
        if filename.starts_with('.') || filename.ends_with(".cache") {
            continue;
        }

        // For mods/shaders, only process .jar files
        // For resource packs, process .zip files
        let valid_extension = match content_type {
            ContentType::Mod | ContentType::Shader => filename.ends_with(".jar"),
            ContentType::ResourcePack => filename.ends_with(".zip"),
        };

        if !valid_extension {
            continue;
        }

        // Compute hashes
        let sha512_hash = sha512_file(&path).ok();
        let murmur2_fingerprint = if let Ok(bytes) = fs::read(&path) {
            Some(murmur2_bytes(&bytes))
        } else {
            None
        };

        // Create basic entry - no platform tracking since we don't know the source
        let installed = InstalledContent {
            name: filename.trim_end_matches(".jar").trim_end_matches(".zip").to_string(),
            slug: filename.trim_end_matches(".jar").trim_end_matches(".zip").to_lowercase().replace(' ', "-"),
            modrinth_id: None,
            curseforge_id: None,
            installed_from: ContentPlatform::Modrinth, // Default, will be updated if identified later
            version: "unknown".to_string(),
            version_id: "unknown".to_string(),
            filename: filename.clone(),
            content_type: content_type.clone(),
            installed_at: Utc::now().timestamp(),
            is_dependency: false,
            source: ContentSource::ModpackOriginal,
            sha512_hash,
            murmur2_fingerprint,
        };

        content.push(installed);
    }

    Ok(content)
}

// =============================================================================
// VERSION-SPECIFIC INSTALL FUNCTIONS (for updating existing instances)
// =============================================================================

/// Install a specific Modrinth modpack version to an existing instance
pub async fn install_modrinth_modpack_version<F>(
    state: &AppState,
    _instance_id: &str,
    game_dir: &PathBuf,
    modpack_id: &str,
    version_id: &str,
    progress_callback: F,
) -> Result<(), AppError>
where
    F: Fn(&str, u32),
{
    progress_callback("Fetching version info", 0);

    // Get version info
    let version = modrinth_service::get_modpack_version(&state.http_client, version_id).await?;

    // Get the mrpack file URL
    let mrpack_file = version
        .files
        .iter()
        .find(|f| f.path.ends_with(".mrpack"))
        .or_else(|| version.files.first())
        .ok_or_else(|| AppError::ContentNotFound("No modpack file found".to_string()))?;

    progress_callback("Downloading modpack", 10);

    // Download the mrpack file
    let mrpack_bytes = download_bytes(&state.http_client, &mrpack_file.url).await?;

    progress_callback("Extracting modpack", 20);

    // Parse the mrpack
    let cursor = std::io::Cursor::new(&mrpack_bytes);
    let mut archive = ZipArchive::new(cursor)?;

    // Read modrinth.index.json
    let index: ModrinthIndex = {
        let mut index_file = archive.by_name("modrinth.index.json")?;
        let mut contents = String::new();
        index_file.read_to_string(&mut contents)?;
        serde_json::from_str(&contents)?
    };

    // Extract overrides
    progress_callback("Extracting overrides", 25);
    extract_overrides(&mut archive, game_dir, "overrides")?;
    extract_overrides(&mut archive, game_dir, "client-overrides")?;

    // Download mod files
    let total_files = index.files.len();
    for (i, file_entry) in index.files.iter().enumerate() {
        let filename = file_entry
            .path
            .split('/')
            .last()
            .unwrap_or(&file_entry.path);

        let progress = 30 + ((i * 60) / total_files.max(1));
        progress_callback(&format!("Downloading: {}", filename), progress as u32);

        let dest_path = game_dir.join(&file_entry.path);
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        for url in &file_entry.downloads {
            if download_file_with_hash(
                &state.http_client,
                url,
                &dest_path,
                file_entry.hashes.get("sha1").map(|s| s.as_str()),
            )
            .await
            .is_ok()
            {
                break;
            }
        }
    }

    // Get loader info and install
    let mc_version = index
        .dependencies
        .get("minecraft")
        .cloned()
        .unwrap_or_else(|| "1.20.1".to_string());
    let (loader_type, loader_version) = determine_loader(&index.dependencies);

    if let (Some(lt), Some(lv)) = (loader_type, loader_version) {
        if lt != LoaderType::Vanilla {
            progress_callback("Installing mod loader", 90);
            loader_service::install_loader(game_dir, lt, &mc_version, &lv, |_, _| {}).await?;
        }
    }

    progress_callback("Complete", 100);
    Ok(())
}

/// Install a specific CurseForge modpack version to an existing instance
pub async fn install_curseforge_modpack_version<F>(
    state: &AppState,
    _instance_id: &str,
    game_dir: &PathBuf,
    api_key: &str,
    modpack_id: &str,
    version_id: &str,
    progress_callback: F,
) -> Result<(), AppError>
where
    F: Fn(&str, u32),
{
    progress_callback("Fetching version info", 0);

    // Get versions and find the specific one
    let versions =
        curseforge_service::get_modpack_versions(&state.http_client, api_key, modpack_id).await?;
    let version = versions
        .into_iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| AppError::ContentNotFound(format!("Version {} not found", version_id)))?;

    // Get the modpack zip URL
    let zip_file = version
        .files
        .first()
        .ok_or_else(|| AppError::ContentNotFound("No modpack file found".to_string()))?;

    progress_callback("Downloading modpack", 10);

    // Download the modpack zip
    let zip_bytes = download_bytes(&state.http_client, &zip_file.url).await?;

    progress_callback("Extracting modpack", 20);

    // Parse the zip
    let cursor = std::io::Cursor::new(&zip_bytes);
    let mut archive = ZipArchive::new(cursor)?;

    // Read manifest.json
    let manifest: CurseForgeManifest = {
        let mut manifest_file = archive.by_name("manifest.json")?;
        let mut contents = String::new();
        manifest_file.read_to_string(&mut contents)?;
        serde_json::from_str(&contents)?
    };

    // Extract overrides
    progress_callback("Extracting overrides", 25);
    let overrides_folder = manifest.overrides.as_deref().unwrap_or("overrides");
    extract_overrides(&mut archive, game_dir, overrides_folder)?;

    // Download mod files
    let mods_dir = game_dir.join("mods");
    fs::create_dir_all(&mods_dir)?;

    let total_files = manifest.files.len();
    for (i, file_ref) in manifest.files.iter().enumerate() {
        let progress = 30 + ((i * 60) / total_files.max(1));
        progress_callback(&format!("Downloading mod {}", file_ref.project_id), progress as u32);

        // Get file info from CurseForge
        if let Ok(file_info) = curseforge_service::get_mod_file(
            &state.http_client,
            api_key,
            file_ref.project_id,
            file_ref.file_id,
        )
        .await
        {
            let dest_path = mods_dir.join(&file_info.filename);
            let _ = download_bytes_to_file(&state.http_client, &file_info.download_url, &dest_path).await;
        }
    }

    // Get loader info and install
    let mc_version = manifest.minecraft.version.clone();
    if let Some(loader) = manifest.minecraft.mod_loaders.first() {
        let (loader_type, loader_version) = parse_curseforge_loader(&loader.id);
        if let (Some(lt), Some(lv)) = (loader_type, loader_version) {
            if lt != LoaderType::Vanilla {
                progress_callback("Installing mod loader", 90);
                loader_service::install_loader(game_dir, lt, &mc_version, &lv, |_, _| {}).await?;
            }
        }
    }

    progress_callback("Complete", 100);
    Ok(())
}

/// Install a specific FTB modpack version to an existing instance
pub async fn install_ftb_modpack_version<F>(
    state: &AppState,
    _instance_id: &str,
    game_dir: &PathBuf,
    modpack_id: &str,
    version_id: &str,
    progress_callback: F,
) -> Result<(), AppError>
where
    F: Fn(&str, u32),
{
    use crate::services::ftb_service;

    progress_callback("Fetching version info", 0);

    // Get versions and find the specific one
    let versions = ftb_service::get_modpack_versions(&state.http_client, modpack_id).await?;
    let version = versions
        .into_iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| AppError::ContentNotFound(format!("Version {} not found", version_id)))?;

    // Download mod files
    let total_files = version.files.len();
    for (i, file) in version.files.iter().enumerate() {
        let filename = file.path.split('/').last().unwrap_or(&file.path);
        let progress = 10 + ((i * 80) / total_files.max(1));
        progress_callback(&format!("Downloading: {}", filename), progress as u32);

        let dest_path = game_dir.join(&file.path);
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let hash = file.hash.as_deref();
        let _ = download_file_with_hash(&state.http_client, &file.url, &dest_path, hash).await;
    }

    // Install loader
    if version.loader_type != LoaderType::Vanilla {
        if let Some(ref lv) = version.loader_version {
            progress_callback("Installing mod loader", 90);
            loader_service::install_loader(
                game_dir,
                version.loader_type.clone(),
                &version.mc_version,
                lv,
                |_, _| {},
            )
            .await?;
        }
    }

    progress_callback("Complete", 100);
    Ok(())
}

/// Install a specific Technic modpack version to an existing instance
pub async fn install_technic_modpack_version<F>(
    state: &AppState,
    _instance_id: &str,
    game_dir: &PathBuf,
    modpack_id: &str,
    version_id: &str,
    progress_callback: F,
) -> Result<(), AppError>
where
    F: Fn(&str, u32),
{
    use crate::services::technic_service;

    progress_callback("Fetching version info", 0);

    // Get versions and find the specific one
    let versions = technic_service::get_modpack_versions(&state.http_client, modpack_id).await?;
    let version = versions
        .into_iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| AppError::ContentNotFound(format!("Version {} not found", version_id)))?;

    // Download mod files
    let total_files = version.files.len();
    for (i, file) in version.files.iter().enumerate() {
        let filename = file.path.split('/').last().unwrap_or(&file.path);
        let progress = 10 + ((i * 80) / total_files.max(1));
        progress_callback(&format!("Downloading: {}", filename), progress as u32);

        let dest_path = game_dir.join(&file.path);
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let hash = file.hash.as_deref();
        let _ = download_file_with_hash(&state.http_client, &file.url, &dest_path, hash).await;
    }

    // Install loader
    if version.loader_type != LoaderType::Vanilla {
        if let Some(ref lv) = version.loader_version {
            progress_callback("Installing mod loader", 90);
            loader_service::install_loader(
                game_dir,
                version.loader_type.clone(),
                &version.mc_version,
                lv,
                |_, _| {},
            )
            .await?;
        }
    }

    progress_callback("Complete", 100);
    Ok(())
}

/// Install a specific ATLauncher modpack version to an existing instance
pub async fn install_atlauncher_modpack_version<F>(
    state: &AppState,
    _instance_id: &str,
    game_dir: &PathBuf,
    modpack_id: &str,
    version_id: &str,
    progress_callback: F,
) -> Result<(), AppError>
where
    F: Fn(&str, u32),
{
    progress_callback("Fetching version info", 0);

    // Get versions and find the specific one
    let versions = atlauncher_service::get_modpack_versions(&state.http_client, modpack_id).await?;
    let version = versions
        .into_iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| AppError::ContentNotFound(format!("Version {} not found", version_id)))?;

    // Download mod files
    let total_files = version.files.len();
    for (i, file) in version.files.iter().enumerate() {
        let filename = file.path.split('/').last().unwrap_or(&file.path);
        let progress = 10 + ((i * 80) / total_files.max(1));
        progress_callback(&format!("Downloading: {}", filename), progress as u32);

        let dest_path = game_dir.join(&file.path);
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let hash = file.hash.as_deref();
        let _ = download_file_with_hash(&state.http_client, &file.url, &dest_path, hash).await;
    }

    // Install loader
    if version.loader_type != LoaderType::Vanilla {
        if let Some(ref lv) = version.loader_version {
            progress_callback("Installing mod loader", 90);
            loader_service::install_loader(
                game_dir,
                version.loader_type.clone(),
                &version.mc_version,
                lv,
                |_, _| {},
            )
            .await?;
        }
    }

    progress_callback("Complete", 100);
    Ok(())
}

// =============================================================================
// FULL INSTALLATION FUNCTIONS (for creating new instances)
// =============================================================================

/// Install an FTB modpack and create a new instance
pub async fn install_ftb_modpack(
    state: &AppState,
    modpack_id: &str,
    version_id: &str,
    instance_name: Option<String>,
    app_handle: Option<&AppHandle>,
) -> Result<Instance, AppError> {
    println!(
        "[modpack_install] install_ftb_modpack: modpack_id={}, version_id={}",
        modpack_id, version_id
    );

    emit_progress(app_handle, "Fetching modpack info", 0, None, 0, 0);

    // Get modpack info
    let modpack = ftb_service::get_modpack(&state.http_client, modpack_id).await?;
    println!("[modpack_install] Got FTB modpack: {}", modpack.name);

    // Get version details
    let pack_id: u64 = modpack_id
        .parse()
        .map_err(|_| AppError::ModpackNotFound(modpack_id.to_string()))?;
    let ver_id: u64 = version_id
        .parse()
        .map_err(|_| AppError::ContentNotFound(version_id.to_string()))?;

    emit_progress(app_handle, "Fetching version info", 5, None, 0, 0);
    let version = ftb_service::get_version_details(&state.http_client, pack_id, ver_id).await?;
    println!(
        "[modpack_install] Got FTB version: {} with {} files",
        version.name,
        version.files.len()
    );

    // Create instance
    let instance_name = instance_name.unwrap_or_else(|| modpack.name.clone());
    let instance_id = Uuid::new_v4().to_string();

    emit_progress(app_handle, "Creating instance", 10, None, 0, 0);

    // Get instance directories
    let instances_base = state.settings.read().instances_path.clone();
    let instance_dir =
        crate::utils::paths::get_instance_dir_with_base(&instances_base, &instance_id);
    let game_dir =
        crate::utils::paths::get_instance_game_dir_with_base(&instances_base, &instance_id);

    // Create directories
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

    let instance = Instance {
        id: instance_id.clone(),
        name: instance_name,
        minecraft_version: version.mc_version.clone(),
        loader_type: version.loader_type.clone(),
        loader_version: version.loader_version.clone(),
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
        modpack_platform: Some(ModpackPlatform::FTB),
        modpack_id: Some(modpack_id.to_string()),
        modpack_version_id: Some(version_id.to_string()),
    };

    instance_service::save_instance(state, &instance)?;

    // Download mod files
    let total_files = version.files.len() as u32;
    emit_progress(app_handle, "Downloading files", 15, None, total_files, 0);

    for (i, file) in version.files.iter().enumerate() {
        let filename = file.path.split('/').last().unwrap_or(&file.path);

        emit_progress(
            app_handle,
            "Downloading files",
            15 + ((i as u32 * 75) / total_files.max(1)),
            Some(filename.to_string()),
            total_files,
            i as u32,
        );

        let dest_path = game_dir.join(&file.path);
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let hash = file.hash.as_deref();
        if let Err(e) =
            download_file_with_hash(&state.http_client, &file.url, &dest_path, hash).await
        {
            eprintln!("[modpack_install] Failed to download {}: {}", file.path, e);
        }
    }

    // Install mod loader if not Vanilla
    if instance.loader_type != LoaderType::Vanilla {
        if let Some(ref lv) = instance.loader_version {
            emit_progress(
                app_handle,
                "Installing mod loader",
                90,
                Some(format!("{:?} {}", instance.loader_type, lv)),
                0,
                0,
            );

            loader_service::install_loader(
                &game_dir,
                instance.loader_type.clone(),
                &version.mc_version,
                lv,
                |msg, pct| {
                    emit_progress(
                        app_handle,
                        &format!("Loader: {}", msg),
                        90 + (pct / 20),
                        None,
                        0,
                        0,
                    );
                },
            )
            .await?;
        }
    }

    emit_progress(
        app_handle,
        "Installation complete",
        100,
        None,
        total_files,
        total_files,
    );

    // Create manifest
    if let Err(e) = create_modpack_manifest(state, &instance.id, &game_dir) {
        eprintln!(
            "[modpack_install] Warning: Failed to create manifest: {}",
            e
        );
    }

    println!(
        "[modpack_install] FTB modpack installed successfully: instance_id={}",
        instance.id
    );
    Ok(instance)
}

/// Install a Technic modpack and create a new instance
pub async fn install_technic_modpack(
    state: &AppState,
    modpack_id: &str,
    version_id: &str,
    instance_name: Option<String>,
    app_handle: Option<&AppHandle>,
) -> Result<Instance, AppError> {
    println!(
        "[modpack_install] install_technic_modpack: modpack_id={}, version_id={}",
        modpack_id, version_id
    );

    emit_progress(app_handle, "Fetching modpack info", 0, None, 0, 0);

    // Get full modpack info (includes solder URL if available)
    let pack = technic_service::get_modpack_full(&state.http_client, modpack_id).await?;
    println!("[modpack_install] Got Technic modpack: {}", pack.name);

    // For Solder packs, we need to get loader info from the build, not the pack
    // The pack-level forge field is often null even for Forge packs
    let (mc_version, loader_type, loader_version) = if let Some(ref solder_url) = pack.solder {
        // Try to get build info to determine loader
        match technic_service::get_solder_build(&state.http_client, solder_url, modpack_id, version_id).await {
            Ok(build) => {
                let mc = build.minecraft.clone();
                let (lt, lv) = if let Some(ref forge_ver) = build.forge {
                    (LoaderType::Forge, Some(forge_ver.clone()))
                } else {
                    (LoaderType::Vanilla, None)
                };
                (mc, lt, lv)
            }
            Err(_) => {
                // Fallback to pack-level info
                let mc = pack.minecraft.clone().unwrap_or_else(|| "1.12.2".to_string());
                let lt = if pack.forge.is_some() { LoaderType::Forge } else { LoaderType::Vanilla };
                (mc, lt, pack.forge.clone())
            }
        }
    } else {
        // Non-Solder pack - use pack-level info
        let mc = pack.minecraft.clone().unwrap_or_else(|| "1.12.2".to_string());
        let lt = if pack.forge.is_some() { LoaderType::Forge } else { LoaderType::Vanilla };
        (mc, lt, pack.forge.clone())
    };

    // Create instance
    let instance_name =
        instance_name.unwrap_or_else(|| pack.display_name.clone().unwrap_or(pack.name.clone()));
    let instance_id = Uuid::new_v4().to_string();

    emit_progress(app_handle, "Creating instance", 5, None, 0, 0);

    // Get instance directories
    let instances_base = state.settings.read().instances_path.clone();
    let instance_dir =
        crate::utils::paths::get_instance_dir_with_base(&instances_base, &instance_id);
    let game_dir =
        crate::utils::paths::get_instance_game_dir_with_base(&instances_base, &instance_id);

    // Create directories
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

    let instance = Instance {
        id: instance_id.clone(),
        name: instance_name,
        minecraft_version: mc_version.clone(),
        loader_type: loader_type.clone(),
        loader_version: loader_version.clone(),
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
        modpack_platform: Some(ModpackPlatform::Technic),
        modpack_id: Some(modpack_id.to_string()),
        modpack_version_id: Some(version_id.to_string()),
    };

    instance_service::save_instance(state, &instance)?;

    // Check if this is a Solder pack
    if let Some(ref solder_url) = pack.solder {
        // Solder pack - download individual mods
        emit_progress(app_handle, "Fetching version info", 10, None, 0, 0);

        let build =
            technic_service::get_solder_build(&state.http_client, solder_url, modpack_id, version_id)
                .await?;

        let total_files = build.mods.len() as u32;
        emit_progress(app_handle, "Downloading mods", 15, None, total_files, 0);

        let mods_dir = game_dir.join("mods");
        fs::create_dir_all(&mods_dir)?;

        for (i, mod_file) in build.mods.iter().enumerate() {
            emit_progress(
                app_handle,
                "Downloading mods",
                15 + ((i as u32 * 75) / total_files.max(1)),
                Some(mod_file.name.clone()),
                total_files,
                i as u32,
            );

            // Technic Solder mods are zip files that need to be extracted
            let zip_bytes = download_bytes(&state.http_client, &mod_file.url).await?;

            // Extract the zip to the game directory
            let cursor = std::io::Cursor::new(&zip_bytes);
            if let Ok(mut archive) = ZipArchive::new(cursor) {
                for j in 0..archive.len() {
                    if let Ok(mut file) = archive.by_index(j) {
                        let name = file.name().to_string();
                        if name.ends_with('/') {
                            // Directory
                            fs::create_dir_all(game_dir.join(&name))?;
                        } else {
                            let dest_path = game_dir.join(&name);
                            if let Some(parent) = dest_path.parent() {
                                fs::create_dir_all(parent)?;
                            }
                            let mut outfile = File::create(&dest_path)?;
                            std::io::copy(&mut file, &mut outfile)?;
                        }
                    }
                }
            }
        }
    } else {
        // Non-Solder pack - download the modpack zip
        let download_url = pack.url.ok_or_else(|| {
            AppError::ContentNotFound("No download URL found for modpack".to_string())
        })?;

        emit_progress(app_handle, "Downloading modpack", 10, None, 0, 0);

        let zip_bytes = download_bytes(&state.http_client, &download_url).await?;

        emit_progress(app_handle, "Extracting modpack", 50, None, 0, 0);

        // Extract the zip to the game directory
        let cursor = std::io::Cursor::new(&zip_bytes);
        let mut archive = ZipArchive::new(cursor)?;

        let total_files = archive.len() as u32;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            emit_progress(
                app_handle,
                "Extracting files",
                50 + ((i as u32 * 40) / total_files.max(1)),
                Some(name.clone()),
                total_files,
                i as u32,
            );

            if name.ends_with('/') {
                fs::create_dir_all(game_dir.join(&name))?;
            } else {
                let dest_path = game_dir.join(&name);
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = File::create(&dest_path)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }
    }

    // Install mod loader if not Vanilla
    if loader_type != LoaderType::Vanilla {
        if let Some(ref lv) = loader_version {
            emit_progress(
                app_handle,
                "Installing mod loader",
                90,
                Some(format!("{:?} {}", loader_type, lv)),
                0,
                0,
            );

            loader_service::install_loader(&game_dir, loader_type.clone(), &mc_version, lv, |msg, pct| {
                emit_progress(
                    app_handle,
                    &format!("Loader: {}", msg),
                    90 + (pct / 20),
                    None,
                    0,
                    0,
                );
            })
            .await?;
        }
    }

    emit_progress(app_handle, "Installation complete", 100, None, 0, 0);

    // Create manifest
    if let Err(e) = create_modpack_manifest(state, &instance.id, &game_dir) {
        eprintln!(
            "[modpack_install] Warning: Failed to create manifest: {}",
            e
        );
    }

    println!(
        "[modpack_install] Technic modpack installed successfully: instance_id={}",
        instance.id
    );
    Ok(instance)
}

/// Install an ATLauncher modpack and create a new instance
/// Uses CDN (Configs.json) instead of blocked API
pub async fn install_atlauncher_modpack(
    state: &AppState,
    modpack_id: &str,
    version_id: &str,
    instance_name: Option<String>,
    app_handle: Option<&AppHandle>,
) -> Result<Instance, AppError> {
    println!(
        "[modpack_install] install_atlauncher_modpack: modpack_id={}, version_id={}",
        modpack_id, version_id
    );

    emit_progress(app_handle, "Fetching modpack info", 0, None, 0, 0);

    // Get pack safe name from CDN (needed for Configs.json URL)
    let pack_safe_name =
        atlauncher_service::get_pack_safe_name(&state.http_client, modpack_id).await?;

    println!(
        "[modpack_install] ATLauncher pack safe name: {} -> {}",
        modpack_id, pack_safe_name
    );

    // Fetch version manifest from CDN (Configs.json)
    emit_progress(app_handle, "Fetching version manifest", 5, None, 0, 0);
    let manifest = atlauncher_service::get_version_manifest(
        &state.http_client,
        &pack_safe_name,
        version_id,
    )
    .await?;

    // Parse loader info from manifest
    let loader_type = match manifest.loader_type.as_deref() {
        Some("forge") => LoaderType::Forge,
        Some("neoforge") => LoaderType::NeoForge,
        Some("fabric") => LoaderType::Fabric,
        Some("quilt") => LoaderType::Quilt,
        Some("liteloader") => LoaderType::LiteLoader,
        _ => LoaderType::Vanilla,
    };
    let loader_version = manifest.loader_version.clone();

    // Create instance
    let instance_name = instance_name.unwrap_or_else(|| modpack_id.to_string());
    let instance_id = Uuid::new_v4().to_string();

    emit_progress(app_handle, "Creating instance", 10, None, 0, 0);

    // Get instance directories
    let instances_base = state.settings.read().instances_path.clone();
    let instance_dir =
        crate::utils::paths::get_instance_dir_with_base(&instances_base, &instance_id);
    let game_dir =
        crate::utils::paths::get_instance_game_dir_with_base(&instances_base, &instance_id);

    // Create directories
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

    let instance = Instance {
        id: instance_id.clone(),
        name: instance_name,
        minecraft_version: manifest.minecraft.clone(),
        loader_type: loader_type.clone(),
        loader_version: loader_version.clone(),
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
        modpack_platform: Some(ModpackPlatform::ATLauncher),
        modpack_id: Some(modpack_id.to_string()),
        modpack_version_id: Some(version_id.to_string()),
    };

    instance_service::save_instance(state, &instance)?;

    let cdn_base = atlauncher_service::get_cdn_base();

    // Download Configs.zip if available (configs, scripts, etc.)
    if !manifest.no_configs {
        emit_progress(app_handle, "Downloading configs", 15, None, 0, 0);

        let configs_url = format!(
            "{}/packs/{}/versions/{}/Configs.zip",
            cdn_base, pack_safe_name, version_id
        );

        if let Ok(config_bytes) = download_bytes(&state.http_client, &configs_url).await {
            emit_progress(app_handle, "Extracting configs", 20, None, 0, 0);
            let cursor = std::io::Cursor::new(&config_bytes);
            if let Ok(mut archive) = ZipArchive::new(cursor) {
                for i in 0..archive.len() {
                    if let Ok(mut file) = archive.by_index(i) {
                        let name = file.name().to_string();
                        if name.ends_with('/') {
                            fs::create_dir_all(game_dir.join(&name))?;
                        } else {
                            let dest_path = game_dir.join(&name);
                            if let Some(parent) = dest_path.parent() {
                                fs::create_dir_all(parent)?;
                            }
                            let mut outfile = File::create(&dest_path)?;
                            std::io::copy(&mut file, &mut outfile)?;
                        }
                    }
                }
            }
        }
    }

    // Download mods from manifest
    // Filter to non-optional mods, or optional mods that are selected by default
    let mods_to_download: Vec<_> = manifest
        .mods
        .iter()
        .filter(|m| !m.optional || m.selected)
        .collect();

    let total_mods = mods_to_download.len() as u32;
    let mut browser_downloads = Vec::new();

    emit_progress(app_handle, "Downloading mods", 25, None, total_mods, 0);

    for (i, mod_entry) in mods_to_download.iter().enumerate() {
        emit_progress(
            app_handle,
            "Downloading mods",
            25 + ((i as u32 * 60) / total_mods.max(1)),
            Some(mod_entry.name.clone()),
            total_mods,
            i as u32,
        );

        // Determine file path
        let mod_type = mod_entry.mod_type.as_deref().unwrap_or("mods");
        let filename = mod_entry
            .file
            .as_ref()
            .map(|f| f.clone())
            .unwrap_or_else(|| format!("{}-{}.jar", mod_entry.name, mod_entry.version));
        let dest_path = game_dir.join(mod_type).join(&filename);

        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Determine download URL based on download type
        let download_type = mod_entry.download.as_deref().unwrap_or("server");
        let download_url = match download_type {
            "server" => {
                // Server-hosted: prepend CDN base to URL
                if let Some(url) = &mod_entry.url {
                    Some(format!("{}/{}", cdn_base, url))
                } else {
                    eprintln!(
                        "[atlauncher] No URL for server-hosted mod: {}",
                        mod_entry.name
                    );
                    None
                }
            }
            "direct" => {
                // Direct download: use URL as-is
                mod_entry.url.clone()
            }
            "browser" => {
                // Browser download: user must manually download
                browser_downloads.push(mod_entry.name.clone());
                None
            }
            _ => {
                eprintln!(
                    "[atlauncher] Unknown download type '{}' for mod: {}",
                    download_type, mod_entry.name
                );
                None
            }
        };

        if let Some(url) = download_url {
            let hash = mod_entry.md5.as_deref().or(mod_entry.sha1.as_deref());
            if let Err(e) =
                download_file_with_hash(&state.http_client, &url, &dest_path, hash).await
            {
                eprintln!(
                    "[modpack_install] Failed to download mod {}: {}",
                    mod_entry.name, e
                );
            }
        }
    }

    // Warn about browser downloads
    if !browser_downloads.is_empty() {
        eprintln!(
            "[atlauncher] {} mods require manual browser download: {:?}",
            browser_downloads.len(),
            browser_downloads
        );
    }

    // Install mod loader if not Vanilla
    if loader_type != LoaderType::Vanilla {
        if let Some(ref lv) = loader_version {
            emit_progress(
                app_handle,
                "Installing mod loader",
                90,
                Some(format!("{:?} {}", loader_type, lv)),
                0,
                0,
            );

            loader_service::install_loader(
                &game_dir,
                loader_type.clone(),
                &manifest.minecraft,
                lv,
                |msg, pct| {
                    emit_progress(
                        app_handle,
                        &format!("Loader: {}", msg),
                        90 + (pct / 20),
                        None,
                        0,
                        0,
                    );
                },
            )
            .await?;
        }
    }

    emit_progress(app_handle, "Installation complete", 100, None, 0, 0);

    // Create manifest
    if let Err(e) = create_modpack_manifest(state, &instance.id, &game_dir) {
        eprintln!(
            "[modpack_install] Warning: Failed to create manifest: {}",
            e
        );
    }

    println!(
        "[modpack_install] ATLauncher modpack installed successfully: instance_id={}",
        instance.id
    );
    Ok(instance)
}
