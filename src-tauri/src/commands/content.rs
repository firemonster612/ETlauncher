use crate::error::CommandError;
use crate::models::{
    Content, ContentPlatform, ContentSearchParams, ContentSearchResult, ContentType,
    ContentVersion, InstalledContent, LoaderType, ResolvedDependency, ScanResult,
};
use crate::models::content::QueueInstallRequest;
use crate::services::{content_install_service, content_queue_service, content_scan_service, curseforge_service, instance_service, modrinth_service};
use crate::state::AppState;
use tauri::{AppHandle, State};

/// Validate that content installation is allowed for the given instance and content type.
/// Mods and shaders require a mod loader; vanilla instances cannot install them.
fn validate_vanilla_restrictions(
    state: &AppState,
    instance_id: &str,
    content_type: &ContentType,
) -> Result<(), CommandError> {
    // Only mods and shaders require a mod loader
    if !matches!(content_type, ContentType::Mod | ContentType::Shader) {
        return Ok(());
    }

    // Get the instance to check its loader type
    let instance = instance_service::get_instance(state, instance_id)
        .map_err(CommandError::from)?;

    if instance.loader_type == LoaderType::Vanilla {
        let content_name = match content_type {
            ContentType::Mod => "Mods",
            ContentType::Shader => "Shaders",
            _ => "This content",
        };
        return Err(CommandError {
            code: "VANILLA_RESTRICTION".to_string(),
            message: format!(
                "{} require a mod loader (Fabric, Forge, etc.). This instance is vanilla.",
                content_name
            ),
        });
    }

    Ok(())
}

/// Search for content (mods, shaders, resource packs) across platforms
#[tauri::command]
pub async fn search_content(
    state: State<'_, AppState>,
    params: ContentSearchParams,
) -> Result<ContentSearchResult, CommandError> {
    // For now, only search the specified platform or default to Modrinth
    let platform = params.platform.clone().unwrap_or(ContentPlatform::Modrinth);

    match platform {
        ContentPlatform::Modrinth => {
            modrinth_service::search_content(&state.http_client, &params)
                .await
                .map_err(CommandError::from)
        }
        ContentPlatform::CurseForge => {
            let api_key = state.get_settings().curseforge_api_key.ok_or_else(|| CommandError {
                code: "API_KEY_REQUIRED".to_string(),
                message: "CurseForge API key not configured. Add it to your settings.".to_string(),
            })?;
            curseforge_service::search_content(&state.http_client, &api_key, &params)
                .await
                .map_err(CommandError::from)
        }
    }
}

/// Get content by ID
#[tauri::command]
pub async fn get_content(
    state: State<'_, AppState>,
    platform: ContentPlatform,
    id: String,
) -> Result<Content, CommandError> {
    match platform {
        ContentPlatform::Modrinth => modrinth_service::get_content(&state.http_client, &id)
            .await
            .map_err(CommandError::from),
        ContentPlatform::CurseForge => {
            let api_key = state.get_settings().curseforge_api_key.ok_or_else(|| CommandError {
                code: "API_KEY_REQUIRED".to_string(),
                message: "CurseForge API key not configured".to_string(),
            })?;
            curseforge_service::get_content(&state.http_client, &api_key, &id)
                .await
                .map_err(CommandError::from)
        }
    }
}

/// Get versions for content
#[tauri::command]
pub async fn get_content_versions(
    state: State<'_, AppState>,
    platform: ContentPlatform,
    id: String,
    mc_version: Option<String>,
    loader: Option<LoaderType>,
) -> Result<Vec<ContentVersion>, CommandError> {
    match platform {
        ContentPlatform::Modrinth => {
            modrinth_service::get_content_versions(
                &state.http_client,
                &id,
                mc_version.as_deref(),
                loader.as_ref(),
            )
            .await
            .map_err(CommandError::from)
        }
        ContentPlatform::CurseForge => {
            let api_key = state.get_settings().curseforge_api_key.ok_or_else(|| CommandError {
                code: "API_KEY_REQUIRED".to_string(),
                message: "CurseForge API key not configured".to_string(),
            })?;
            curseforge_service::get_content_versions(
                &state.http_client,
                &api_key,
                &id,
                mc_version.as_deref(),
                loader.as_ref(),
            )
            .await
            .map_err(CommandError::from)
        }
    }
}

/// Get a specific version by ID
/// Note: For CurseForge, version_id should be in format "modId:fileId"
#[tauri::command]
pub async fn get_content_version(
    state: State<'_, AppState>,
    platform: ContentPlatform,
    version_id: String,
    mod_id: Option<String>,
) -> Result<ContentVersion, CommandError> {
    match platform {
        ContentPlatform::Modrinth => modrinth_service::get_version(&state.http_client, &version_id)
            .await
            .map_err(CommandError::from),
        ContentPlatform::CurseForge => {
            let api_key = state.get_settings().curseforge_api_key.ok_or_else(|| CommandError {
                code: "API_KEY_REQUIRED".to_string(),
                message: "CurseForge API key not configured".to_string(),
            })?;
            let mod_id = mod_id.ok_or_else(|| CommandError {
                code: "MISSING_MOD_ID".to_string(),
                message: "mod_id is required for CurseForge versions".to_string(),
            })?;
            curseforge_service::get_version(&state.http_client, &api_key, &mod_id, &version_id)
                .await
                .map_err(CommandError::from)
        }
    }
}

/// Install content (mod, shader, resource pack) to an instance
#[tauri::command]
pub async fn install_content(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    instance_id: String,
    platform: ContentPlatform,
    content_id: String,
    content_name: String,
    content_slug: String,
    content_type: ContentType,
    version: ContentVersion,
    is_dependency: Option<bool>,
) -> Result<InstalledContent, CommandError> {
    // Validate vanilla restrictions (mods/shaders require a mod loader)
    validate_vanilla_restrictions(&state, &instance_id, &content_type)?;

    content_install_service::install_content(
        &state,
        &instance_id,
        platform,
        &content_id,
        &content_name,
        &content_slug,
        content_type,
        &version,
        is_dependency.unwrap_or(false),
        None, // source: defaults to UserAdded
        Some(&app_handle),
    )
    .await
    .map_err(CommandError::from)
}

/// Resolve dependencies for a content version
#[tauri::command]
pub async fn resolve_content_dependencies(
    state: State<'_, AppState>,
    instance_id: String,
    platform: ContentPlatform,
    version: ContentVersion,
    mc_version: String,
    loader: Option<LoaderType>,
) -> Result<Vec<ResolvedDependency>, CommandError> {
    content_install_service::resolve_dependencies(
        &state,
        &instance_id,
        &platform,
        &version,
        &mc_version,
        loader.as_ref(),
    )
    .await
    .map_err(CommandError::from)
}

/// Install content with its dependencies
#[tauri::command]
pub async fn install_content_with_dependencies(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    instance_id: String,
    platform: ContentPlatform,
    content: Content,
    version: ContentVersion,
    mc_version: String,
    loader: Option<LoaderType>,
) -> Result<Vec<InstalledContent>, CommandError> {
    // Validate vanilla restrictions (mods/shaders require a mod loader)
    validate_vanilla_restrictions(&state, &instance_id, &content.content_type)?;

    content_install_service::install_content_with_dependencies(
        &state,
        &instance_id,
        platform,
        &content,
        &version,
        &mc_version,
        loader.as_ref(),
        None, // source: defaults to UserAdded
        Some(&app_handle),
    )
    .await
    .map_err(CommandError::from)
}

/// Scan an instance's content folder and identify installed items via Modrinth hash lookup
#[tauri::command]
pub async fn scan_installed_content(
    state: State<'_, AppState>,
    instance_id: String,
    content_type: ContentType,
) -> Result<ScanResult, CommandError> {
    content_scan_service::scan_content(&state, &instance_id, &content_type)
        .await
        .map_err(CommandError::from)
}

/// Uninstall content by filename (delete the file directly)
#[tauri::command]
pub fn uninstall_content_by_filename(
    state: State<'_, AppState>,
    instance_id: String,
    filename: String,
    content_type: ContentType,
) -> Result<(), CommandError> {
    content_scan_service::uninstall_by_filename(&state, &instance_id, &filename, &content_type)
        .map_err(CommandError::from)
}

/// Disable content by moving files to the disabled subfolder
#[tauri::command]
pub fn disable_content(
    state: State<'_, AppState>,
    instance_id: String,
    filenames: Vec<String>,
    content_type: ContentType,
) -> Result<(), CommandError> {
    content_scan_service::disable_content(&state, &instance_id, &filenames, &content_type)
        .map_err(CommandError::from)
}

/// Enable content by moving files from the disabled subfolder back
#[tauri::command]
pub fn enable_content(
    state: State<'_, AppState>,
    instance_id: String,
    filenames: Vec<String>,
    content_type: ContentType,
) -> Result<(), CommandError> {
    content_scan_service::enable_content(&state, &instance_id, &filenames, &content_type)
        .map_err(CommandError::from)
}

/// Queue content for parallel installation
#[tauri::command]
pub async fn queue_content_install(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    request: QueueInstallRequest,
) -> Result<(), CommandError> {
    // Validate vanilla restrictions
    validate_vanilla_restrictions(&state, &request.instance_id, &request.content_type)?;

    content_queue_service::queue_content_install(&state, app_handle, request)
        .await
        .map_err(CommandError::from)
}

/// Cancel a queued content download
#[tauri::command]
pub async fn cancel_content_queue_item(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    queue_id: String,
) -> Result<(), CommandError> {
    content_queue_service::cancel_queue_item(&state, app_handle, queue_id)
        .await
        .map_err(CommandError::from)
}

/// Try to process pending queue items (called when a slot becomes available)
#[tauri::command]
pub async fn try_process_content_queue(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), CommandError> {
    content_queue_service::try_process_queue(&state, app_handle).await;
    Ok(())
}
