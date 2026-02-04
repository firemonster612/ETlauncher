use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use crate::error::AppError;
use crate::utils::paths::get_app_data_dir;

/// Metadata for a saved skin in the library
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedSkin {
    pub id: String,
    pub name: String,
    pub variant: String, // "classic" or "slim"
    pub filename: String,
    pub created_at: i64,
}

/// Skin library storage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkinLibrary {
    pub skins: Vec<SavedSkin>,
}

/// Get the skins directory
pub fn get_skins_dir() -> PathBuf {
    get_app_data_dir().join("skins")
}

/// Get the skins files directory
pub fn get_skins_files_dir() -> PathBuf {
    get_skins_dir().join("files")
}

/// Get the skins metadata file path
pub fn get_skins_metadata_path() -> PathBuf {
    get_skins_dir().join("skins.json")
}

/// Ensure skin directories exist
pub fn ensure_skin_directories() -> Result<(), AppError> {
    fs::create_dir_all(get_skins_dir())?;
    fs::create_dir_all(get_skins_files_dir())?;
    Ok(())
}

/// Load the skin library from disk
pub fn load_skin_library() -> Result<SkinLibrary, AppError> {
    let path = get_skins_metadata_path();
    if !path.exists() {
        return Ok(SkinLibrary::default());
    }

    let data = fs::read_to_string(&path)?;
    let library: SkinLibrary = serde_json::from_str(&data)?;
    Ok(library)
}

/// Save the skin library to disk
pub fn save_skin_library(library: &SkinLibrary) -> Result<(), AppError> {
    ensure_skin_directories()?;
    let path = get_skins_metadata_path();
    let data = serde_json::to_string_pretty(library)?;
    fs::write(&path, data)?;
    Ok(())
}

/// Save a skin to the library
pub fn save_skin_to_library(
    name: String,
    variant: String,
    skin_data: &[u8],
) -> Result<SavedSkin, AppError> {
    ensure_skin_directories()?;

    // Generate unique ID and filename
    let id = Uuid::new_v4().to_string();
    let filename = format!("skin_{}.png", &id[..8]);

    // Save the skin file
    let file_path = get_skins_files_dir().join(&filename);
    fs::write(&file_path, skin_data)?;

    // Create skin metadata
    let saved_skin = SavedSkin {
        id,
        name,
        variant,
        filename,
        created_at: chrono::Utc::now().timestamp(),
    };

    // Add to library
    let mut library = load_skin_library()?;
    library.skins.push(saved_skin.clone());
    save_skin_library(&library)?;

    Ok(saved_skin)
}

/// Get all saved skins
pub fn get_skin_library() -> Result<Vec<SavedSkin>, AppError> {
    let library = load_skin_library()?;
    Ok(library.skins)
}

/// Get skin file data by ID
pub fn get_skin_data(skin_id: &str) -> Result<Vec<u8>, AppError> {
    let library = load_skin_library()?;
    let skin = library
        .skins
        .iter()
        .find(|s| s.id == skin_id)
        .ok_or_else(|| AppError::NotFound(format!("Skin not found: {}", skin_id)))?;

    let file_path = get_skins_files_dir().join(&skin.filename);
    let data = fs::read(&file_path)?;
    Ok(data)
}

/// Delete a skin from the library
pub fn delete_skin_from_library(skin_id: &str) -> Result<(), AppError> {
    let mut library = load_skin_library()?;

    // Find and remove the skin
    let index = library
        .skins
        .iter()
        .position(|s| s.id == skin_id)
        .ok_or_else(|| AppError::NotFound(format!("Skin not found: {}", skin_id)))?;

    let skin = library.skins.remove(index);

    // Delete the file
    let file_path = get_skins_files_dir().join(&skin.filename);
    if file_path.exists() {
        fs::remove_file(&file_path)?;
    }

    // Save updated library
    save_skin_library(&library)?;

    Ok(())
}
