use crate::error::CommandError;
use crate::models::{Instance, Modpack, ModpackSearchParams, ModpackSearchResult, ModpackVersion};
use crate::models::instance::ModpackPlatform;
use crate::services::{atlauncher_service, curseforge_service, ftb_service, modpack_install_service, modrinth_service, technic_service};
use crate::state::AppState;
use tauri::{AppHandle, State};

/// Search for modpacks across platforms
#[tauri::command]
pub async fn search_modpacks(
    state: State<'_, AppState>,
    params: ModpackSearchParams,
) -> Result<ModpackSearchResult, CommandError> {
    println!("[modpack_cmd] search_modpacks: platform={:?}, query={:?}", params.platform, params.query);

    // If no platform specified, search all platforms and aggregate results
    if params.platform.is_none() {
        return search_all_platforms(&state, &params).await;
    }

    let platform = params.platform.clone().unwrap();

    let result = match platform {
        ModpackPlatform::Modrinth => {
            modrinth_service::search_modpacks(&state.http_client, &params)
                .await
                .map_err(CommandError::from)
        }
        ModpackPlatform::CurseForge => {
            let api_key = state.get_settings().curseforge_api_key.ok_or_else(|| CommandError {
                code: "API_KEY_REQUIRED".to_string(),
                message: "CurseForge API key not configured. Add it to your settings.".to_string(),
            })?;
            curseforge_service::search_modpacks(&state.http_client, &api_key, &params)
                .await
                .map_err(CommandError::from)
        }
        ModpackPlatform::FTB => {
            ftb_service::search_modpacks(&state.http_client, &params)
                .await
                .map_err(CommandError::from)
        }
        ModpackPlatform::Technic => {
            technic_service::search_modpacks(&state.http_client, &params)
                .await
                .map_err(CommandError::from)
        }
        ModpackPlatform::ATLauncher => {
            atlauncher_service::search_modpacks(&state.http_client, &params)
                .await
                .map_err(CommandError::from)
        }
    };

    match &result {
        Ok(r) => println!("[modpack_cmd] search_modpacks: success, count={}", r.modpacks.len()),
        Err(e) => println!("[modpack_cmd] search_modpacks: error={:?}", e),
    }

    result
}

/// Search all platforms and aggregate results
async fn search_all_platforms(
    state: &State<'_, AppState>,
    params: &ModpackSearchParams,
) -> Result<ModpackSearchResult, CommandError> {
    let page = params.page.unwrap_or(0);
    let page_size = params.page_size.unwrap_or(20);

    // Create futures for each platform search
    let modrinth_future = modrinth_service::search_modpacks(&state.http_client, params);
    let ftb_future = ftb_service::search_modpacks(&state.http_client, params);
    let atlauncher_future = atlauncher_service::search_modpacks(&state.http_client, params);

    // Technic uses trending endpoint when no query provided
    let technic_future = technic_service::search_modpacks(&state.http_client, params);

    // CurseForge requires API key
    let curseforge_future = async {
        if let Some(api_key) = state.get_settings().curseforge_api_key {
            curseforge_service::search_modpacks(&state.http_client, &api_key, params).await
        } else {
            Ok(ModpackSearchResult {
                modpacks: vec![],
                total_count: 0,
                page,
                page_size,
            })
        }
    };

    // Run all searches in parallel
    let results = tokio::join!(
        modrinth_future,
        ftb_future,
        atlauncher_future,
        technic_future,
        curseforge_future
    );

    // Collect successful results, log errors but don't fail
    let mut all_modpacks = Vec::new();
    let mut total_count: u64 = 0;

    if let Ok(result) = results.0 {
        println!("[modpack_cmd] Modrinth: {} packs", result.modpacks.len());
        total_count += result.total_count;
        all_modpacks.extend(result.modpacks);
    } else if let Err(e) = &results.0 {
        println!("[modpack_cmd] Modrinth error: {:?}", e);
    }

    if let Ok(result) = results.1 {
        println!("[modpack_cmd] FTB: {} packs", result.modpacks.len());
        total_count += result.total_count;
        all_modpacks.extend(result.modpacks);
    } else if let Err(e) = &results.1 {
        println!("[modpack_cmd] FTB error: {:?}", e);
    }

    if let Ok(result) = results.2 {
        println!("[modpack_cmd] ATLauncher: {} packs", result.modpacks.len());
        total_count += result.total_count;
        all_modpacks.extend(result.modpacks);
    } else if let Err(e) = &results.2 {
        println!("[modpack_cmd] ATLauncher error: {:?}", e);
    }

    if let Ok(result) = results.3 {
        println!("[modpack_cmd] Technic: {} packs", result.modpacks.len());
        total_count += result.total_count;
        all_modpacks.extend(result.modpacks);
    } else if let Err(e) = &results.3 {
        println!("[modpack_cmd] Technic error: {:?}", e);
    }

    if let Ok(result) = results.4 {
        println!("[modpack_cmd] CurseForge: {} packs", result.modpacks.len());
        total_count += result.total_count;
        all_modpacks.extend(result.modpacks);
    } else if let Err(e) = &results.4 {
        println!("[modpack_cmd] CurseForge error: {:?}", e);
    }

    // Sort by downloads (descending) to show most popular first
    all_modpacks.sort_by(|a, b| b.downloads.cmp(&a.downloads));

    println!("[modpack_cmd] All platforms total: {} packs", all_modpacks.len());

    Ok(ModpackSearchResult {
        modpacks: all_modpacks,
        total_count,
        page,
        page_size,
    })
}

/// Get a modpack by ID
#[tauri::command]
pub async fn get_modpack(
    state: State<'_, AppState>,
    platform: ModpackPlatform,
    id: String,
) -> Result<Modpack, CommandError> {
    println!("[modpack_cmd] get_modpack: platform={:?}, id={}", platform, id);
    let result = match platform {
        ModpackPlatform::Modrinth => {
            modrinth_service::get_modpack(&state.http_client, &id)
                .await
                .map_err(CommandError::from)
        }
        ModpackPlatform::CurseForge => {
            let api_key = state.get_settings().curseforge_api_key.ok_or_else(|| CommandError {
                code: "API_KEY_REQUIRED".to_string(),
                message: "CurseForge API key not configured".to_string(),
            })?;
            curseforge_service::get_modpack(&state.http_client, &api_key, &id)
                .await
                .map_err(CommandError::from)
        }
        ModpackPlatform::FTB => {
            ftb_service::get_modpack(&state.http_client, &id)
                .await
                .map_err(CommandError::from)
        }
        ModpackPlatform::Technic => {
            technic_service::get_modpack(&state.http_client, &id)
                .await
                .map_err(CommandError::from)
        }
        ModpackPlatform::ATLauncher => {
            atlauncher_service::get_modpack(&state.http_client, &id)
                .await
                .map_err(CommandError::from)
        }
    };
    match &result {
        Ok(m) => println!("[modpack_cmd] get_modpack: success, name={}", m.name),
        Err(e) => println!("[modpack_cmd] get_modpack: error={:?}", e),
    }
    result
}

/// Get versions for a modpack
#[tauri::command]
pub async fn get_modpack_versions(
    state: State<'_, AppState>,
    platform: ModpackPlatform,
    id: String,
) -> Result<Vec<ModpackVersion>, CommandError> {
    println!("[modpack_cmd] get_modpack_versions: platform={:?}, id={}", platform, id);
    let result = match platform {
        ModpackPlatform::Modrinth => {
            modrinth_service::get_modpack_versions(&state.http_client, &id)
                .await
                .map_err(CommandError::from)
        }
        ModpackPlatform::CurseForge => {
            let api_key = state.get_settings().curseforge_api_key.ok_or_else(|| CommandError {
                code: "API_KEY_REQUIRED".to_string(),
                message: "CurseForge API key not configured".to_string(),
            })?;
            curseforge_service::get_modpack_versions(&state.http_client, &api_key, &id)
                .await
                .map_err(CommandError::from)
        }
        ModpackPlatform::FTB => {
            ftb_service::get_modpack_versions(&state.http_client, &id)
                .await
                .map_err(CommandError::from)
        }
        ModpackPlatform::Technic => {
            technic_service::get_modpack_versions(&state.http_client, &id)
                .await
                .map_err(CommandError::from)
        }
        ModpackPlatform::ATLauncher => {
            atlauncher_service::get_modpack_versions(&state.http_client, &id)
                .await
                .map_err(CommandError::from)
        }
    };
    match &result {
        Ok(versions) => println!("[modpack_cmd] get_modpack_versions: success, count={}", versions.len()),
        Err(e) => println!("[modpack_cmd] get_modpack_versions: error={:?}", e),
    }
    result
}

/// Install a modpack and create a new instance
#[tauri::command]
pub async fn install_modpack(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    platform: ModpackPlatform,
    modpack_id: String,
    version_id: String,
    instance_name: Option<String>,
) -> Result<Instance, CommandError> {
    println!("[modpack_cmd] install_modpack: platform={:?}, modpack_id={}, version_id={}, instance_name={:?}",
        platform, modpack_id, version_id, instance_name);
    let result = match platform {
        ModpackPlatform::Modrinth => {
            modpack_install_service::install_modrinth_modpack(
                &state,
                &modpack_id,
                &version_id,
                instance_name,
                Some(&app_handle),
            )
            .await
            .map_err(CommandError::from)
        }
        ModpackPlatform::CurseForge => {
            modpack_install_service::install_curseforge_modpack(
                &state,
                &modpack_id,
                &version_id,
                instance_name,
                Some(&app_handle),
            )
            .await
            .map_err(CommandError::from)
        }
        ModpackPlatform::FTB | ModpackPlatform::Technic | ModpackPlatform::ATLauncher => {
            // TODO: Implement installation for these platforms
            println!("[modpack_cmd] install_modpack: platform {:?} not implemented", platform);
            Err(CommandError {
                code: "NOT_IMPLEMENTED".to_string(),
                message: format!("Modpack installation for {} is not yet implemented", platform),
            })
        }
    };
    match &result {
        Ok(instance) => println!("[modpack_cmd] install_modpack: success, instance_id={}", instance.id),
        Err(e) => println!("[modpack_cmd] install_modpack: error={:?}", e),
    }
    result
}

/// Import an instance from a local .mrpack file
#[tauri::command]
pub async fn import_modpack_file(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    file_path: String,
    instance_name: Option<String>,
) -> Result<Instance, CommandError> {
    println!(
        "[modpack_cmd] import_modpack_file: file_path={}, instance_name={:?}",
        file_path, instance_name
    );

    let result = modpack_install_service::import_from_mrpack_file(
        &state,
        &file_path,
        instance_name,
        Some(&app_handle),
    )
    .await
    .map_err(CommandError::from);

    match &result {
        Ok(instance) => println!(
            "[modpack_cmd] import_modpack_file: success, instance_id={}",
            instance.id
        ),
        Err(e) => println!("[modpack_cmd] import_modpack_file: error={:?}", e),
    }
    result
}
