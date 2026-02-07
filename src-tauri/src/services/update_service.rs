//! Update checking service
//!
//! Provides functionality to check for updates to modpacks, content, and version migrations.

use crate::error::AppError;
use crate::models::content::{
    ContentPlatform, ContentSource, ContentUpdateInfo, ContentUpdateStatus, InstalledContent,
    InstanceUpdateCheck, ModpackInstanceUpdateCheck, ModpackUpdateInfo, ModpackVersionOption,
    UpdateCheckResult,
};
use crate::models::instance::{LoaderType, ModpackPlatform};
use crate::services::{
    atlauncher_service, curseforge_service, download_service, ftb_service, instance_service,
    loader_service, manifest_service, modrinth_service, technic_service,
};
use crate::state::AppState;
use crate::utils::version::is_version_newer;

/// Check for modpack updates
///
/// Returns information about available modpack updates, or None if already on latest version.
/// Only works for instances created from modpacks (has modpack_platform set).
pub async fn check_modpack_update(
    state: &AppState,
    instance_id: &str,
) -> Result<Option<ModpackUpdateInfo>, AppError> {
    let instance = instance_service::get_instance(state, instance_id)?;

    // Only check modpack instances
    let platform = match &instance.modpack_platform {
        Some(p) => *p,
        None => return Ok(None), // Not a modpack instance
    };

    let modpack_id = instance
        .modpack_id
        .as_ref()
        .ok_or_else(|| AppError::InvalidInput("Modpack ID missing".to_string()))?;

    let current_version_id = instance
        .modpack_version_id
        .as_ref()
        .ok_or_else(|| AppError::InvalidInput("Modpack version ID missing".to_string()))?;

    // Fetch available versions from platform
    let versions = match platform {
        ModpackPlatform::Modrinth => {
            modrinth_service::get_modpack_versions(&state.http_client, modpack_id).await?
        }
        ModpackPlatform::CurseForge => {
            let api_key = state.get_settings().curseforge_api_key.ok_or_else(|| {
                AppError::ApiError("CurseForge API key not configured".to_string())
            })?;
            curseforge_service::get_modpack_versions(&state.http_client, &api_key, modpack_id)
                .await?
        }
        _ => {
            // FTB, Technic, ATLauncher - limited support
            return Ok(None);
        }
    };

    // Find the current version and any newer versions
    let current_idx = versions.iter().position(|v| v.id == *current_version_id);
    let current_version = current_idx.and_then(|i| versions.get(i));

    // Versions are typically sorted newest first
    // Find the latest version that's newer than current
    if let Some(latest) = versions.first() {
        if latest.id != *current_version_id {
            let current_name = current_version
                .map(|v| v.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            return Ok(Some(ModpackUpdateInfo {
                platform,
                modpack_id: modpack_id.clone(),
                current_version_id: current_version_id.clone(),
                current_version_name: current_name,
                available_version_id: latest.id.clone(),
                available_version_name: latest.name.clone(),
                available_mc_version: latest.mc_version.clone(),
                changelog: latest.changelog.clone(),
            }));
        }
    }

    Ok(None)
}

/// Check for content updates (same MC version, newer mod versions)
///
/// Returns an UpdateCheckResult with lists of updatable, up-to-date, incompatible, and unidentified content.
pub async fn check_content_updates(
    state: &AppState,
    instance_id: &str,
) -> Result<UpdateCheckResult, AppError> {
    let instance = instance_service::get_instance(state, instance_id)?;
    let manifest = manifest_service::load_manifest(state, instance_id)?;

    // Get loader versions for the current MC version
    let available_loader_versions = if instance.loader_type != LoaderType::Vanilla {
        loader_service::get_loader_versions(instance.loader_type, &instance.minecraft_version)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut result = UpdateCheckResult {
        instance_id: instance_id.to_string(),
        current_mc_version: instance.minecraft_version.clone(),
        current_loader_version: instance.loader_version.clone(),
        target_mc_version: instance.minecraft_version.clone(),
        target_loader: instance.loader_type,
        target_loader_version: instance.loader_version.clone(),
        available_loader_versions,
        updatable: Vec::new(),
        up_to_date: Vec::new(),
        incompatible: Vec::new(),
        unidentified: Vec::new(),
        modpack_update: None,
    };

    // Check all content concurrently to avoid sequential API bottleneck
    let mc_ver = &instance.minecraft_version;
    let loader = instance.loader_type;

    let mut futures = Vec::new();
    for content in &manifest.mods {
        futures.push(check_single_content_update(
            state,
            content,
            mc_ver,
            Some(&loader),
        ));
    }
    for content in &manifest.shaders {
        futures.push(check_single_content_update(
            state,
            content,
            mc_ver,
            Some(&loader),
        ));
    }
    for content in &manifest.resource_packs {
        futures.push(check_single_content_update(state, content, mc_ver, None));
    }

    let results = futures::future::join_all(futures).await;

    for info in results {
        categorize_update_info(&mut result, info);
    }

    // Check for modpack update if this is a modpack instance
    if instance.modpack_platform.is_some() {
        result.modpack_update = check_modpack_update(state, instance_id)
            .await
            .ok()
            .flatten();
    }

    Ok(result)
}

/// Check content compatibility for a different MC version (version migration preview)
pub async fn check_version_migration(
    state: &AppState,
    instance_id: &str,
    target_mc_version: &str,
    target_loader: &LoaderType,
) -> Result<UpdateCheckResult, AppError> {
    let instance = instance_service::get_instance(state, instance_id)?;
    let manifest = manifest_service::load_manifest(state, instance_id)?;

    // Get loader versions for the target MC version
    let available_loader_versions = if *target_loader != LoaderType::Vanilla {
        loader_service::get_loader_versions(*target_loader, target_mc_version)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    // Default to latest stable loader version
    let target_loader_version = available_loader_versions
        .iter()
        .find(|v| v.stable)
        .or_else(|| available_loader_versions.first())
        .map(|v| v.version.clone());

    let mut result = UpdateCheckResult {
        instance_id: instance_id.to_string(),
        current_mc_version: instance.minecraft_version.clone(),
        current_loader_version: instance.loader_version.clone(),
        target_mc_version: target_mc_version.to_string(),
        target_loader: *target_loader,
        target_loader_version,
        available_loader_versions,
        updatable: Vec::new(),
        up_to_date: Vec::new(),
        incompatible: Vec::new(),
        unidentified: Vec::new(),
        modpack_update: None,
    };

    // Check all content concurrently to avoid sequential API bottleneck
    let mut futures = Vec::new();
    for content in &manifest.mods {
        futures.push(check_single_content_update(
            state,
            content,
            target_mc_version,
            Some(target_loader),
        ));
    }
    for content in &manifest.shaders {
        futures.push(check_single_content_update(
            state,
            content,
            target_mc_version,
            Some(target_loader),
        ));
    }
    for content in &manifest.resource_packs {
        futures.push(check_single_content_update(
            state,
            content,
            target_mc_version,
            None,
        ));
    }

    let results = futures::future::join_all(futures).await;

    for info in results {
        categorize_update_info(&mut result, info);
    }

    Ok(result)
}

/// Check a single content item for updates
async fn check_single_content_update(
    state: &AppState,
    content: &InstalledContent,
    target_mc_version: &str,
    target_loader: Option<&LoaderType>,
) -> ContentUpdateInfo {
    // If content has no platform IDs, it's unidentified
    if content.modrinth_id.is_none() && content.curseforge_id.is_none() {
        return ContentUpdateInfo {
            filename: content.filename.clone(),
            name: content.name.clone(),
            source: content.source.clone(),
            platform: None,
            project_id: None,
            current_version_id: None,
            status: ContentUpdateStatus::Unidentified,
        };
    }

    // Try Modrinth first, then CurseForge
    let update_status = if let Some(ref modrinth_id) = content.modrinth_id {
        find_latest_modrinth_version(
            state,
            modrinth_id,
            &content.version_id,
            &content.version,
            target_mc_version,
            target_loader,
        )
        .await
    } else if let Some(curseforge_id) = content.curseforge_id {
        find_latest_curseforge_version(
            state,
            curseforge_id,
            &content.version_id,
            &content.version,
            target_mc_version,
            target_loader,
        )
        .await
    } else {
        ContentUpdateStatus::Unidentified
    };

    ContentUpdateInfo {
        filename: content.filename.clone(),
        name: content.name.clone(),
        source: content.source.clone(),
        platform: if content.modrinth_id.is_some() {
            Some(ContentPlatform::Modrinth)
        } else {
            Some(ContentPlatform::CurseForge)
        },
        project_id: content
            .modrinth_id
            .clone()
            .or_else(|| content.curseforge_id.map(|id| id.to_string())),
        current_version_id: Some(content.version_id.clone()),
        status: update_status,
    }
}

/// Find the latest Modrinth version for content
async fn find_latest_modrinth_version(
    state: &AppState,
    project_id: &str,
    current_version_id: &str,
    current_version: &str,
    mc_version: &str,
    loader: Option<&LoaderType>,
) -> ContentUpdateStatus {
    let versions = match modrinth_service::get_content_versions(
        &state.http_client,
        project_id,
        Some(mc_version),
        loader,
    )
    .await
    {
        Ok(v) => v,
        Err(_) => return ContentUpdateStatus::Unavailable,
    };

    if versions.is_empty() {
        return ContentUpdateStatus::NoCompatibleVersion;
    }

    // Check if there's a newer version
    let latest = &versions[0]; // Versions are sorted newest first

    if latest.id == current_version_id {
        return ContentUpdateStatus::UpToDate;
    }

    // Compare versions to confirm it's actually newer
    if is_version_newer(current_version, &latest.version_number) {
        ContentUpdateStatus::UpdateAvailable {
            current_version: current_version.to_string(),
            available_version: latest.version_number.clone(),
            available_version_id: latest.id.clone(),
        }
    } else {
        // Latest available is same or older than what we have
        ContentUpdateStatus::UpToDate
    }
}

/// Find the latest CurseForge version for content
async fn find_latest_curseforge_version(
    state: &AppState,
    project_id: u32,
    current_version_id: &str,
    current_version: &str,
    mc_version: &str,
    loader: Option<&LoaderType>,
) -> ContentUpdateStatus {
    let api_key = match state.get_settings().curseforge_api_key {
        Some(key) => key,
        None => return ContentUpdateStatus::Unavailable,
    };

    let versions = match curseforge_service::get_content_versions(
        &state.http_client,
        &api_key,
        &project_id.to_string(),
        Some(mc_version),
        loader,
    )
    .await
    {
        Ok(v) => v,
        Err(_) => return ContentUpdateStatus::Unavailable,
    };

    if versions.is_empty() {
        return ContentUpdateStatus::NoCompatibleVersion;
    }

    // Check if there's a newer version
    let latest = &versions[0]; // Versions are sorted newest first

    if latest.id == current_version_id {
        return ContentUpdateStatus::UpToDate;
    }

    // Compare versions to confirm it's actually newer
    if is_version_newer(current_version, &latest.version_number) {
        ContentUpdateStatus::UpdateAvailable {
            current_version: current_version.to_string(),
            available_version: latest.version_number.clone(),
            available_version_id: latest.id.clone(),
        }
    } else {
        ContentUpdateStatus::UpToDate
    }
}

/// Categorize update info into the appropriate list in the result
fn categorize_update_info(result: &mut UpdateCheckResult, info: ContentUpdateInfo) {
    match &info.status {
        ContentUpdateStatus::UpdateAvailable { .. } => result.updatable.push(info),
        ContentUpdateStatus::UpToDate => result.up_to_date.push(info),
        ContentUpdateStatus::NoCompatibleVersion => result.incompatible.push(info),
        ContentUpdateStatus::Unidentified => result.unidentified.push(info),
        ContentUpdateStatus::Unavailable => result.incompatible.push(info),
    }
}

// =============================================================================
// NEW UPDATE SYSTEM FUNCTIONS
// =============================================================================

/// Check for modpack instance updates
///
/// Returns all available modpack versions (not just latest) for the user to choose from.
/// Supports all 5 platforms: Modrinth, CurseForge, FTB, Technic, ATLauncher.
pub async fn check_modpack_instance_updates(
    state: &AppState,
    instance_id: &str,
) -> Result<ModpackInstanceUpdateCheck, AppError> {
    let instance = instance_service::get_instance(state, instance_id)?;

    // Verify this is a modpack instance
    let platform = *instance
        .modpack_platform
        .as_ref()
        .ok_or_else(|| AppError::InvalidInput("Not a modpack instance".to_string()))?;

    let modpack_id = instance
        .modpack_id
        .as_ref()
        .ok_or_else(|| AppError::InvalidInput("Modpack ID missing".to_string()))?;

    let current_version_id = instance
        .modpack_version_id
        .as_ref()
        .ok_or_else(|| AppError::InvalidInput("Modpack version ID missing".to_string()))?;

    // Fetch modpack info and versions from the appropriate platform
    let (modpack_name, versions) = match &platform {
        ModpackPlatform::Modrinth => {
            let modpack = modrinth_service::get_modpack(&state.http_client, modpack_id).await?;
            let versions =
                modrinth_service::get_modpack_versions(&state.http_client, modpack_id).await?;
            (modpack.name, versions)
        }
        ModpackPlatform::CurseForge => {
            let api_key = state.get_settings().curseforge_api_key.ok_or_else(|| {
                AppError::ApiError("CurseForge API key not configured".to_string())
            })?;
            let modpack =
                curseforge_service::get_modpack(&state.http_client, &api_key, modpack_id).await?;
            let versions =
                curseforge_service::get_modpack_versions(&state.http_client, &api_key, modpack_id)
                    .await?;
            (modpack.name, versions)
        }
        ModpackPlatform::FTB => {
            let modpack = ftb_service::get_modpack(&state.http_client, modpack_id).await?;
            let versions =
                ftb_service::get_modpack_versions(&state.http_client, modpack_id).await?;
            (modpack.name, versions)
        }
        ModpackPlatform::Technic => {
            let modpack = technic_service::get_modpack(&state.http_client, modpack_id).await?;
            let versions =
                technic_service::get_modpack_versions(&state.http_client, modpack_id).await?;
            (modpack.name, versions)
        }
        ModpackPlatform::ATLauncher => {
            let modpack = atlauncher_service::get_modpack(&state.http_client, modpack_id).await?;
            let versions =
                atlauncher_service::get_modpack_versions(&state.http_client, modpack_id).await?;
            (modpack.name, versions)
        }
    };

    // Convert ModpackVersion to ModpackVersionOption
    let available_versions: Vec<ModpackVersionOption> = versions
        .iter()
        .map(|v| ModpackVersionOption {
            version_id: v.id.clone(),
            version_name: v.name.clone(),
            mc_version: v.mc_version.clone(),
            loader_type: v.loader_type,
            loader_version: v.loader_version.clone(),
            released_at: v.released_at,
            changelog: v.changelog.clone(),
            is_current: v.id == *current_version_id,
        })
        .collect();

    // Find current version info (or create a fallback)
    let current_version = available_versions
        .iter()
        .find(|v| v.is_current)
        .cloned()
        .unwrap_or_else(|| ModpackVersionOption {
            version_id: current_version_id.clone(),
            version_name: "Unknown".to_string(),
            mc_version: instance.minecraft_version.clone(),
            loader_type: instance.loader_type,
            loader_version: instance.loader_version.clone(),
            released_at: None,
            changelog: None,
            is_current: true,
        });

    // Load manifest and find user-added content
    let manifest = manifest_service::load_manifest(state, instance_id)?;
    let user_added_content: Vec<InstalledContent> = manifest
        .mods
        .iter()
        .chain(manifest.shaders.iter())
        .chain(manifest.resource_packs.iter())
        .filter(|c| {
            c.source == ContentSource::UserAdded || c.source == ContentSource::UserDependency
        })
        .cloned()
        .collect();

    // Check if update is available (any version newer than current)
    let has_update = available_versions
        .first()
        .map(|v| !v.is_current)
        .unwrap_or(false);

    Ok(ModpackInstanceUpdateCheck {
        instance_id: instance_id.to_string(),
        modpack_name,
        platform,
        modpack_id: modpack_id.clone(),
        current_version,
        available_versions,
        user_added_content,
        has_update,
    })
}

/// Check for non-modpack instance updates
///
/// Targets the LATEST Minecraft version automatically and checks content compatibility.
pub async fn check_instance_updates(
    state: &AppState,
    instance_id: &str,
) -> Result<InstanceUpdateCheck, AppError> {
    let instance = instance_service::get_instance(state, instance_id)?;

    // Verify this is NOT a modpack instance
    if instance.modpack_platform.is_some() {
        return Err(AppError::InvalidInput(
            "Use check_modpack_instance_updates for modpack instances".to_string(),
        ));
    }

    // Fetch the latest Minecraft version
    let version_manifest =
        download_service::fetch_version_manifest(&state.http_client, false).await?;
    let latest_mc_version = version_manifest.latest.release.clone();

    // Check if MC update is available
    let has_mc_update = is_version_newer(&instance.minecraft_version, &latest_mc_version);

    // Get target loader version for the new MC version
    let target_loader_version = if instance.loader_type != LoaderType::Vanilla && has_mc_update {
        let loader_versions =
            loader_service::get_loader_versions(instance.loader_type, &latest_mc_version)
                .await
                .unwrap_or_default();

        // Get latest stable, or latest if no stable available
        loader_versions
            .iter()
            .find(|v| v.stable)
            .or_else(|| loader_versions.first())
            .map(|v| v.version.clone())
    } else {
        instance.loader_version.clone()
    };

    // If no MC update available, return early with minimal result
    if !has_mc_update {
        return Ok(InstanceUpdateCheck {
            instance_id: instance_id.to_string(),
            current_mc_version: instance.minecraft_version.clone(),
            current_loader_type: instance.loader_type,
            current_loader_version: instance.loader_version.clone(),
            latest_mc_version,
            has_mc_update: false,
            target_loader_version: instance.loader_version.clone(),
            compatible_content: vec![],
            incompatible_content: vec![],
            unidentified_content: vec![],
        });
    }

    // Check content compatibility with the new MC version
    let manifest = manifest_service::load_manifest(state, instance_id)?;

    let mut compatible_content = Vec::new();
    let mut incompatible_content = Vec::new();
    let mut unidentified_content = Vec::new();

    // Check all content types — run checks concurrently to avoid sequential API bottleneck
    let loader = instance.loader_type;
    let mut futures = Vec::new();
    for content in manifest.mods.iter().chain(manifest.shaders.iter()) {
        futures.push(check_single_content_update(
            state,
            content,
            &latest_mc_version,
            Some(&loader),
        ));
    }
    for content in &manifest.resource_packs {
        futures.push(check_single_content_update(
            state,
            content,
            &latest_mc_version,
            None,
        ));
    }

    let results = futures::future::join_all(futures).await;

    for update_info in results {
        match &update_info.status {
            ContentUpdateStatus::UpdateAvailable { .. } | ContentUpdateStatus::UpToDate => {
                compatible_content.push(update_info)
            }
            ContentUpdateStatus::NoCompatibleVersion | ContentUpdateStatus::Unavailable => {
                incompatible_content.push(update_info)
            }
            ContentUpdateStatus::Unidentified => unidentified_content.push(update_info),
        }
    }

    Ok(InstanceUpdateCheck {
        instance_id: instance_id.to_string(),
        current_mc_version: instance.minecraft_version.clone(),
        current_loader_type: instance.loader_type,
        current_loader_version: instance.loader_version.clone(),
        latest_mc_version,
        has_mc_update,
        target_loader_version,
        compatible_content,
        incompatible_content,
        unidentified_content,
    })
}
