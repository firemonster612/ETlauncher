use crate::error::AppError;
use crate::models::{ContentType, LoaderType};
use crate::services::{content_scan_service, instance_service, modrinth_service};
use crate::state::AppState;
use crate::utils::hash::{sha1_file, sha512_file};
use crate::utils::paths::get_instance_game_dir_with_base;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::FileOptions;
use zip::ZipWriter;

/// Modrinth pack index file format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MrpackIndex {
    pub format_version: u32,
    pub game: String,
    pub version_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub dependencies: HashMap<String, String>,
    pub files: Vec<MrpackFile>,
}

/// A file entry in the mrpack index
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MrpackFile {
    pub path: String,
    pub hashes: MrpackHashes,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<MrpackEnv>,
    pub downloads: Vec<String>,
    pub file_size: u64,
}

/// File hashes for mrpack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MrpackHashes {
    pub sha1: String,
    pub sha512: String,
}

/// Environment settings for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MrpackEnv {
    pub client: String,
    pub server: String,
}

/// Directories to include in overrides
const OVERRIDE_DIRS: &[&str] = &["config", "shaderpacks", "resourcepacks"];

/// Export an instance to Modrinth .mrpack format
pub async fn export_to_mrpack(
    state: &AppState,
    instance_id: &str,
    output_path: &Path,
) -> Result<PathBuf, AppError> {
    // Get instance data
    let instance = instance_service::get_instance(state, instance_id)?;
    let instances_base = state.settings.read().instances_path.clone();
    let game_dir = get_instance_game_dir_with_base(&instances_base, instance_id);

    // Scan installed mods to identify them via Modrinth hash lookup
    let scan_result =
        content_scan_service::scan_content(state, instance_id, &ContentType::Mod).await?;

    // Build the files list and collect unidentified files for overrides
    let mut mrpack_files: Vec<MrpackFile> = Vec::new();
    let mut unidentified_mod_files: Vec<String> = Vec::new();

    let mods_dir = game_dir.join("mods");

    for item in &scan_result.items {
        // Skip disabled items - they shouldn't be exported
        if item.is_disabled {
            continue;
        }

        let file_path = mods_dir.join(&item.filename);

        if let Some(modrinth) = &item.modrinth_project {
            // Identified via Modrinth - get the download URL from version info
            if let Ok(version) =
                modrinth_service::get_version(&state.http_client, &modrinth.version_id).await
            {
                // Find the primary file or the first file
                if let Some(file) = version
                    .files
                    .iter()
                    .find(|f| f.primary)
                    .or_else(|| version.files.first())
                {
                    // Calculate hashes for the local file
                    let sha1 = sha1_file(&file_path).unwrap_or_default();
                    let sha512 = sha512_file(&file_path).unwrap_or_default();

                    mrpack_files.push(MrpackFile {
                        path: format!("mods/{}", item.filename),
                        hashes: MrpackHashes { sha1, sha512 },
                        env: Some(MrpackEnv {
                            client: "required".to_string(),
                            server: "required".to_string(),
                        }),
                        downloads: vec![file.url.clone()],
                        file_size: item.size,
                    });
                    continue;
                }
            }
            // If we couldn't get version info, treat as unidentified
            unidentified_mod_files.push(item.filename.clone());
        } else {
            // Not identified via Modrinth - will be included in overrides
            unidentified_mod_files.push(item.filename.clone());
        }
    }

    // Build dependencies map
    let mut dependencies: HashMap<String, String> = HashMap::new();
    dependencies.insert("minecraft".to_string(), instance.minecraft_version.clone());

    // Add loader dependency if not vanilla
    match &instance.loader_type {
        LoaderType::Fabric => {
            if let Some(version) = &instance.loader_version {
                dependencies.insert("fabric-loader".to_string(), version.clone());
            }
        }
        LoaderType::Forge => {
            if let Some(version) = &instance.loader_version {
                dependencies.insert("forge".to_string(), version.clone());
            }
        }
        LoaderType::NeoForge => {
            if let Some(version) = &instance.loader_version {
                dependencies.insert("neoforge".to_string(), version.clone());
            }
        }
        LoaderType::Quilt => {
            if let Some(version) = &instance.loader_version {
                dependencies.insert("quilt-loader".to_string(), version.clone());
            }
        }
        _ => {}
    }

    // Create the mrpack index
    let index = MrpackIndex {
        format_version: 1,
        game: "minecraft".to_string(),
        version_id: "1.0.0".to_string(),
        name: instance.name.clone(),
        summary: Some(format!(
            "Exported from ETLauncher - MC {} with {}",
            instance.minecraft_version, instance.loader_type
        )),
        dependencies,
        files: mrpack_files,
    };

    // Determine output file path
    let output_file = if output_path.is_dir() {
        output_path.join(format!("{}.mrpack", sanitize_filename(&instance.name)))
    } else {
        output_path.to_path_buf()
    };

    // Create the ZIP file
    let file = File::create(&output_file)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // Write modrinth.index.json
    let index_json = serde_json::to_string_pretty(&index)?;
    zip.start_file("modrinth.index.json", options)?;
    zip.write_all(index_json.as_bytes())?;

    // Add unidentified mods to overrides
    for filename in &unidentified_mod_files {
        let file_path = mods_dir.join(filename);
        if file_path.exists() {
            add_file_to_zip(
                &mut zip,
                &file_path,
                &format!("overrides/mods/{}", filename),
                options,
            )?;
        }
    }

    // Add other override directories (config, shaderpacks, resourcepacks)
    for dir_name in OVERRIDE_DIRS {
        let dir_path = game_dir.join(dir_name);
        if dir_path.exists() && dir_path.is_dir() {
            add_directory_to_zip(
                &mut zip,
                &dir_path,
                &format!("overrides/{}", dir_name),
                options,
            )?;
        }
    }

    // Finalize the ZIP
    zip.finish()?;

    Ok(output_file)
}

/// Add a single file to the ZIP archive
fn add_file_to_zip<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    file_path: &Path,
    archive_path: &str,
    options: FileOptions<()>,
) -> Result<(), AppError> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    zip.start_file(archive_path, options)?;
    zip.write_all(&buffer)?;

    Ok(())
}

/// Recursively add a directory to the ZIP archive
fn add_directory_to_zip<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    dir_path: &Path,
    archive_prefix: &str,
    options: FileOptions<()>,
) -> Result<(), AppError> {
    if !dir_path.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files and cache files
        if name.starts_with('.') {
            continue;
        }

        let archive_path = format!("{}/{}", archive_prefix, name);

        if path.is_dir() {
            add_directory_to_zip(zip, &path, &archive_path, options)?;
        } else if path.is_file() {
            add_file_to_zip(zip, &path, &archive_path, options)?;
        }
    }

    Ok(())
}

/// Sanitize a filename by removing/replacing invalid characters
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}
