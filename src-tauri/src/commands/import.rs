use crate::error::CommandError;
use crate::models::Instance;
use crate::services::{instance_export_service, instance_import_service};
use crate::state::AppState;
use std::path::PathBuf;
use tauri::{AppHandle, State};

/// Analyze an import source to determine its type and extract metadata
#[tauri::command]
pub fn analyze_import_source(
    path: String,
) -> Result<instance_import_service::ImportAnalysis, CommandError> {
    let source_path = PathBuf::from(&path);
    instance_import_service::analyze_import_source(&source_path).map_err(CommandError::from)
}

/// Import from a folder (vanilla .minecraft or MultiMC/Prism)
#[tauri::command]
pub async fn import_from_folder(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    source_path: String,
    instance_name: String,
    source_type: String,
) -> Result<Instance, CommandError> {
    let path = PathBuf::from(&source_path);

    match source_type.as_str() {
        "vanillaMinecraft" => instance_import_service::import_vanilla_minecraft(
            &state,
            &path,
            instance_name,
            Some(&app_handle),
        )
        .await
        .map_err(CommandError::from),
        "multiMC" | "prismLauncher" => instance_import_service::import_multimc_prism(
            &state,
            &path,
            instance_name,
            Some(&app_handle),
        )
        .await
        .map_err(CommandError::from),
        _ => Err(CommandError {
            code: "INVALID_SOURCE_TYPE".to_string(),
            message: format!("Unknown source type: {}", source_type),
        }),
    }
}

/// Import a CurseForge modpack from a .zip file
#[tauri::command]
pub async fn import_curseforge_zip(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    file_path: String,
    instance_name: Option<String>,
) -> Result<Instance, CommandError> {
    let path = PathBuf::from(&file_path);
    instance_import_service::import_curseforge_zip(&state, &path, instance_name, Some(&app_handle))
        .await
        .map_err(CommandError::from)
}

/// Export an instance to CurseForge .zip format
#[tauri::command]
pub async fn export_curseforge_modpack(
    state: State<'_, AppState>,
    instance_id: String,
    output_path: String,
) -> Result<String, CommandError> {
    let path = PathBuf::from(&output_path);
    let result_path =
        instance_export_service::export_to_curseforge_zip(&state, &instance_id, &path)
            .await
            .map_err(CommandError::from)?;

    Ok(result_path.to_string_lossy().to_string())
}
