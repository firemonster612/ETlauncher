use serde::Serialize;
use std::path::Path;
use tauri::{State, WebviewWindow};
use uuid::Uuid;

use crate::cache::clear_all_disk_caches;
use crate::error::CommandError;
use crate::models::{AppSettings, UpdateSettingsRequest};
use crate::services::{java_service, settings_service};
use crate::state::{ApiCache, AppState};
use crate::utils::paths;

/// Get current application settings
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, CommandError> {
    Ok(state.get_settings())
}

/// Update application settings
#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    updates: UpdateSettingsRequest,
) -> Result<AppSettings, CommandError> {
    // Get current settings
    let current = state.get_settings();

    // Merge updates
    let new_settings = settings_service::update_settings(&current, updates);

    // Validate the new settings
    settings_service::validate_settings(&new_settings).map_err(CommandError::from)?;

    // Update state
    {
        let mut settings = state.settings.write();
        *settings = new_settings.clone();
    }

    // Save to disk
    state.save_settings().map_err(CommandError::from)?;

    Ok(new_settings)
}

/// Reset settings to defaults
#[tauri::command]
pub fn reset_settings(state: State<'_, AppState>) -> Result<AppSettings, CommandError> {
    let settings = AppSettings {
        instances_path: settings_service::get_default_instances_path(),
        ..Default::default()
    };

    // Update state
    {
        let mut current = state.settings.write();
        *current = settings.clone();
    }

    // Save to disk
    state.save_settings().map_err(CommandError::from)?;

    Ok(settings)
}

/// Get the default instances path
#[tauri::command]
pub fn get_default_instances_path() -> String {
    settings_service::get_default_instances_path()
}

/// Response from cache clearing operation
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheClearResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// Number of in-memory caches cleared
    pub memory_caches_cleared: u32,
    /// Whether disk cache was cleared
    pub disk_cache_cleared: bool,
}

/// Clear all API caches (for troubleshooting)
#[tauri::command]
pub fn clear_api_caches(state: State<'_, AppState>) -> Result<CacheClearResult, CommandError> {
    // Clear in-memory caches
    state.api_cache.clear_all();

    // Clear disk caches
    let disk_cleared = clear_all_disk_caches().is_ok();

    Ok(CacheClearResult {
        success: true,
        memory_caches_cleared: ApiCache::CACHE_COUNT,
        disk_cache_cleared: disk_cleared,
    })
}

// Note: Folder picker will be handled via tauri-plugin-dialog in frontend
// The frontend will use the dialog plugin directly

/// Get the system theme (light or dark)
/// This uses multiple detection methods for cross-platform support
#[tauri::command]
pub fn get_system_theme(window: WebviewWindow) -> String {
    // On Linux, try to detect via gsettings (GNOME/GTK color-scheme)
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "color-scheme"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let scheme = stdout.trim().trim_matches('\'');
            if scheme == "prefer-light" || scheme == "default" {
                return "light".to_string();
            } else if scheme == "prefer-dark" {
                return "dark".to_string();
            }
        }
    }

    // Fallback to Tauri's window theme detection
    match window.theme() {
        Ok(tauri::Theme::Light) => "light".to_string(),
        Ok(tauri::Theme::Dark) | Ok(_) => "dark".to_string(),
        Err(_) => "dark".to_string(),
    }
}

/// Copy a background file to the backgrounds directory
/// Returns the new filename (uuid-based)
#[tauri::command]
pub fn copy_background_file(source_path: String) -> Result<String, CommandError> {
    use crate::error::AppError;

    let source = Path::new(&source_path);

    // Validate source exists
    if !source.exists() {
        return Err(
            AppError::InvalidInput(format!("Source file does not exist: {}", source_path)).into(),
        );
    }

    // Get extension from source file
    let extension = source.extension().and_then(|e| e.to_str()).unwrap_or("bin");

    // Generate UUID-based filename
    let new_filename = format!("{}.{}", Uuid::new_v4(), extension);

    // Ensure backgrounds directory exists
    let backgrounds_dir = paths::get_backgrounds_dir();
    std::fs::create_dir_all(&backgrounds_dir).map_err(AppError::from)?;

    // Copy file to backgrounds directory
    let dest_path = backgrounds_dir.join(&new_filename);
    std::fs::copy(source, &dest_path).map_err(AppError::from)?;

    Ok(new_filename)
}

/// Delete a background file from the backgrounds directory
#[tauri::command]
pub fn delete_background_file(filename: String) -> Result<(), CommandError> {
    use crate::error::AppError;

    // Validate filename doesn't contain path traversal
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(AppError::InvalidInput("Invalid filename".to_string()).into());
    }

    let file_path = paths::get_backgrounds_dir().join(&filename);

    // Only delete if it exists
    if file_path.exists() {
        std::fs::remove_file(&file_path).map_err(AppError::from)?;
    }

    Ok(())
}

/// Get the full path to a background file
#[tauri::command]
pub fn get_background_path(filename: String) -> Result<String, CommandError> {
    use crate::error::AppError;

    // Validate filename doesn't contain path traversal
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(AppError::InvalidInput("Invalid filename".to_string()).into());
    }

    let file_path = paths::get_backgrounds_dir().join(&filename);

    if !file_path.exists() {
        return Err(AppError::InvalidInput(format!(
            "Background file does not exist: {}",
            filename
        ))
        .into());
    }

    Ok(file_path.to_string_lossy().to_string())
}

/// Get background file data as base64
#[tauri::command]
pub fn get_background_data(filename: String) -> Result<String, CommandError> {
    use crate::error::AppError;

    // Validate filename doesn't contain path traversal
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(AppError::InvalidInput("Invalid filename".to_string()).into());
    }

    let file_path = paths::get_backgrounds_dir().join(&filename);

    if !file_path.exists() {
        return Err(AppError::InvalidInput(format!(
            "Background file does not exist: {}",
            filename
        ))
        .into());
    }

    let data = std::fs::read(&file_path).map_err(AppError::from)?;
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &data,
    ))
}

/// Collect comprehensive debug information for troubleshooting (pretty-printed JSON)
#[tauri::command]
pub fn get_debug_info(state: State<'_, AppState>) -> Result<String, CommandError> {
    use crate::models::account::AccountType;
    use crate::services::{account_service, instance_service, resource_pool_service};
    use crate::utils::platform;
    use serde_json::{json, Map, Value};

    let settings = state.get_settings();
    let mut root = Map::new();

    root.insert(
        "generated".into(),
        json!(chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string()),
    );

    // App
    let mut app = Map::new();
    app.insert("version".into(), json!(env!("CARGO_PKG_VERSION")));
    app.insert(
        "executable".into(),
        json!(std::env::current_exe()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".into())),
    );
    #[cfg(target_os = "linux")]
    {
        let appimage_path = std::env::var("APPIMAGE").ok();
        app.insert("appimage".into(), json!(appimage_path.is_some()));
        if let Some(ref path) = appimage_path {
            app.insert("appimagePath".into(), json!(path));
        }
    }
    root.insert("app".into(), Value::Object(app));

    // System
    let mut system = Map::new();
    system.insert("os".into(), json!(platform::get_os_name()));
    system.insert("arch".into(), json!(platform::get_arch()));
    system.insert(
        "osDetail".into(),
        json!(format!(
            "{} {}",
            std::env::consts::OS,
            std::env::consts::ARCH
        )),
    );
    #[cfg(target_os = "linux")]
    {
        if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
            let mem: Map<String, Value> = meminfo
                .lines()
                .take(3)
                .filter_map(|line| {
                    let mut parts = line.splitn(2, ':');
                    let key = parts.next()?.trim().to_string();
                    let val = parts.next()?.trim().to_string();
                    Some((key, json!(val)))
                })
                .collect();
            system.insert("memory".into(), Value::Object(mem));
        }
        if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            system.insert("desktop".into(), json!(desktop));
        }
        if let Ok(session) = std::env::var("XDG_SESSION_TYPE") {
            system.insert("sessionType".into(), json!(session));
        }
    }
    root.insert("system".into(), Value::Object(system));

    // Paths
    let data_dir = paths::get_app_data_dir();
    let instances_path = std::path::PathBuf::from(&settings.instances_path);
    let mut paths_obj = Map::new();
    paths_obj.insert("dataDir".into(), json!(data_dir.display().to_string()));
    paths_obj.insert("dataDirExists".into(), json!(data_dir.exists()));
    paths_obj.insert("instancesDir".into(), json!(settings.instances_path));
    paths_obj.insert("instancesDirExists".into(), json!(instances_path.exists()));
    paths_obj.insert(
        "cacheDir".into(),
        json!(paths::get_cache_dir().display().to_string()),
    );
    paths_obj.insert(
        "javaDir".into(),
        json!(paths::get_java_dir().display().to_string()),
    );
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("df")
            .args(["-h", &data_dir.to_string_lossy()])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().nth(1) {
                paths_obj.insert("diskSpace".into(), json!(line.trim()));
            }
        }
    }
    root.insert("paths".into(), Value::Object(paths_obj));

    // Java
    let java_manifest = java_service::load_java_manifest();
    let java_list: Vec<Value> = java_manifest
        .installations
        .iter()
        .map(|install| {
            json!({
                "majorVersion": install.major_version,
                "path": install.java_path,
                "exists": std::path::PathBuf::from(&install.java_path).exists(),
            })
        })
        .collect();
    root.insert("java".into(), json!(java_list));

    // Accounts (sanitized)
    let mut accounts_obj = Map::new();
    match account_service::load_accounts() {
        Ok(accounts) => {
            let microsoft_count = accounts
                .iter()
                .filter(|a| a.account_type == AccountType::Microsoft)
                .count();
            let offline_count = accounts
                .iter()
                .filter(|a| a.account_type == AccountType::Offline)
                .count();
            accounts_obj.insert("microsoft".into(), json!(microsoft_count));
            accounts_obj.insert("offline".into(), json!(offline_count));
            if let Some(active) = accounts.iter().find(|a| a.is_active) {
                accounts_obj.insert(
                    "active".into(),
                    json!({
                        "username": active.username,
                        "type": format!("{:?}", active.account_type),
                    }),
                );
            }
        }
        Err(e) => {
            accounts_obj.insert("error".into(), json!(e.to_string()));
        }
    }
    root.insert("accounts".into(), Value::Object(accounts_obj));

    // Instances
    let mut instances_obj = Map::new();
    match instance_service::get_all_instances(&state) {
        Ok(instances) => {
            instances_obj.insert("total".into(), json!(instances.len()));
            let list: Vec<Value> = instances
                .iter()
                .map(|inst| {
                    let loader = if inst.loader_type == crate::models::instance::LoaderType::Vanilla
                    {
                        "vanilla".to_string()
                    } else {
                        format!(
                            "{} {}",
                            inst.loader_type,
                            inst.loader_version.as_deref().unwrap_or("")
                        )
                    };
                    json!({
                        "name": inst.name,
                        "minecraftVersion": inst.minecraft_version,
                        "loader": loader,
                        "modpack": inst.modpack_platform.as_ref().map(|p| p.to_string()),
                    })
                })
                .collect();
            instances_obj.insert("list".into(), json!(list));
        }
        Err(e) => {
            instances_obj.insert("error".into(), json!(e.to_string()));
        }
    }
    root.insert("instances".into(), Value::Object(instances_obj));

    // Running instances
    let running_list: Vec<Value> = {
        let running = state.running_instances.read();
        running
            .iter()
            .map(|(id, info)| json!({"id": id, "pid": info.pid}))
            .collect()
    };
    root.insert("runningInstances".into(), json!(running_list));

    // Resource pool
    let mut pool_obj = Map::new();
    pool_obj.insert("enabled".into(), json!(settings.resource_pool.enabled));
    if settings.resource_pool.enabled {
        let stats = resource_pool_service::get_pool_stats(&state);
        pool_obj.insert("totalResources".into(), json!(stats.total_resources));
        pool_obj.insert("totalSizeBytes".into(), json!(stats.total_size_bytes));
        pool_obj.insert("spaceSavedBytes".into(), json!(stats.space_saved_bytes));
        pool_obj.insert("mods".into(), json!(stats.mod_count));
        pool_obj.insert("shaders".into(), json!(stats.shader_count));
        pool_obj.insert("resourcePacks".into(), json!(stats.resource_pack_count));
        pool_obj.insert(
            "linkStrategy".into(),
            json!(format!("{:?}", settings.resource_pool.link_strategy)),
        );
    }
    root.insert("resourcePool".into(), Value::Object(pool_obj));

    // Settings (sanitized)
    let mut settings_obj = Map::new();
    settings_obj.insert("theme".into(), json!(format!("{:?}", settings.theme)));
    settings_obj.insert(
        "colorPreset".into(),
        json!(format!("{:?}", settings.color_preset)),
    );
    settings_obj.insert("memoryMinMb".into(), json!(settings.memory_min_mb));
    settings_obj.insert("memoryMaxMb".into(), json!(settings.memory_max_mb));
    settings_obj.insert(
        "concurrentDownloads".into(),
        json!(settings.concurrent_downloads),
    );
    settings_obj.insert(
        "closeLauncherOnGameStart".into(),
        json!(settings.close_launcher_on_game_start),
    );
    settings_obj.insert(
        "reopenLauncherOnGameClose".into(),
        json!(settings.reopen_launcher_on_game_close),
    );
    settings_obj.insert("showSnapshots".into(), json!(settings.show_snapshots));
    settings_obj.insert("autoUpdate".into(), json!(settings.auto_update));
    settings_obj.insert(
        "includePreReleases".into(),
        json!(settings.include_pre_releases),
    );
    settings_obj.insert(
        "curseforgeApiKeySet".into(),
        json!(settings.curseforge_api_key.is_some()),
    );
    if let Some(ref bg) = settings.background {
        settings_obj.insert(
            "background".into(),
            json!({"type": format!("{:?}", bg.bg_type), "blur": bg.blur}),
        );
    }
    settings_obj.insert(
        "fontFamily".into(),
        json!(format!("{:?}", settings.font_family)),
    );
    settings_obj.insert(
        "sidebarStyle".into(),
        json!(format!("{:?}", settings.sidebar_style)),
    );
    settings_obj.insert(
        "disableHoverLift".into(),
        json!(settings.disable_hover_lift),
    );
    root.insert("settings".into(), Value::Object(settings_obj));

    // Yggdrasil server
    let ygg_port = state
        .yggdrasil_port
        .load(std::sync::atomic::Ordering::Relaxed);
    root.insert(
        "yggdrasilServer".into(),
        json!({"port": ygg_port, "running": ygg_port > 0}),
    );

    // Task registry
    let tasks = state.task_registry.list();
    let task_list: Vec<Value> = tasks
        .iter()
        .map(|task| {
            json!({
                "id": task.id,
                "label": task.label,
                "status": format!("{:?}", task.status),
                "progress": task.progress.as_ref().map(|p| {
                    if let Some(pct) = p.percent {
                        format!("{:.0}%", pct)
                    } else if p.total > 0 {
                        format!("{}/{}", p.current, p.total)
                    } else {
                        format!("{}", p.current)
                    }
                }),
            })
        })
        .collect();
    root.insert(
        "taskRegistry".into(),
        json!({"activeTasks": tasks.len(), "tasks": task_list}),
    );

    // Recent logs (from in-memory ring buffer)
    let log_entries = crate::log_buffer::snapshot();
    let log_list: Vec<Value> = log_entries
        .iter()
        .map(|entry| {
            json!({
                "timestamp": entry.timestamp,
                "level": entry.level,
                "message": entry.message,
            })
        })
        .collect();
    root.insert(
        "recentLogs".into(),
        json!({"count": log_list.len(), "entries": log_list}),
    );

    serde_json::to_string_pretty(&Value::Object(root))
        .map_err(|e| CommandError::from(crate::error::AppError::from(e)))
}
