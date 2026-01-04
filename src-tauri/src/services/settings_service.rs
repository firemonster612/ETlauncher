use crate::error::AppError;
use crate::models::{AppSettings, UpdateSettingsRequest};
use crate::utils::paths;
use std::path::Path;

/// Load settings from disk, or create defaults if not found
pub fn load_settings() -> Result<AppSettings, AppError> {
    let config_path = paths::get_config_path();

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;

        // Try to parse as new camelCase format first
        if let Ok(mut settings) = serde_json::from_str::<AppSettings>(&content) {
            // Fix empty instances path
            if settings.instances_path.is_empty() {
                settings.instances_path = paths::get_instances_dir()
                    .to_string_lossy()
                    .to_string();
                let _ = save_settings(&settings);
            }
            return Ok(settings);
        }

        // Try to parse as old snake_case format and migrate
        if let Ok(old_settings) = serde_json::from_str::<serde_json::Value>(&content) {
            eprintln!("Migrating old settings format to camelCase...");
            let settings = migrate_old_settings(&old_settings);

            // Save in new format
            if let Err(e) = save_settings(&settings) {
                eprintln!("Warning: Could not save migrated settings: {}", e);
            }

            return Ok(settings);
        }

        // If all parsing fails, create defaults
        eprintln!("Warning: Could not parse settings, creating defaults");
    }

    // Create default settings with proper paths
    let mut settings = AppSettings::default();
    settings.instances_path = paths::get_instances_dir()
        .to_string_lossy()
        .to_string();

    // Try to save the defaults
    if let Err(e) = save_settings(&settings) {
        eprintln!("Warning: Could not save default settings: {}", e);
    }

    Ok(settings)
}

/// Migrate old snake_case settings to new format
fn migrate_old_settings(old: &serde_json::Value) -> AppSettings {
    let default_instances = paths::get_instances_dir().to_string_lossy().to_string();

    AppSettings {
        instances_path: old.get("instances_path")
            .or_else(|| old.get("instancesPath"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| default_instances.clone()),
        java_path: old.get("java_path")
            .or_else(|| old.get("javaPath"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        memory_min_mb: old.get("memory_min_mb")
            .or_else(|| old.get("memoryMinMb"))
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(512),
        memory_max_mb: old.get("memory_max_mb")
            .or_else(|| old.get("memoryMaxMb"))
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(4096),
        concurrent_downloads: old.get("concurrent_downloads")
            .or_else(|| old.get("concurrentDownloads"))
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(4),
        close_launcher_on_game_start: old.get("close_launcher_on_game_start")
            .or_else(|| old.get("closeLauncherOnGameStart"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        reopen_launcher_on_game_close: old.get("reopen_launcher_on_game_close")
            .or_else(|| old.get("reopenLauncherOnGameClose"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        show_snapshots: old.get("show_snapshots")
            .or_else(|| old.get("showSnapshots"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        show_old_versions: old.get("show_old_versions")
            .or_else(|| old.get("showOldVersions"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        theme: old.get("theme")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "dark" => Some(crate::models::Theme::Dark),
                "light" => Some(crate::models::Theme::Light),
                "system" => Some(crate::models::Theme::System),
                _ => None,
            })
            .unwrap_or_default(),
        curseforge_api_key: old.get("curseforge_api_key")
            .or_else(|| old.get("curseforgeApiKey"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    }
}

/// Save settings to disk
pub fn save_settings(settings: &AppSettings) -> Result<(), AppError> {
    let config_path = paths::get_config_path();

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(settings)?;
    std::fs::write(&config_path, content)?;

    Ok(())
}

/// Update settings with partial updates, merging with existing values
pub fn update_settings(
    current: &AppSettings,
    updates: UpdateSettingsRequest,
) -> AppSettings {
    AppSettings {
        instances_path: updates.instances_path.unwrap_or_else(|| current.instances_path.clone()),
        java_path: updates.java_path.or_else(|| current.java_path.clone()),
        memory_min_mb: updates.memory_min_mb.unwrap_or(current.memory_min_mb),
        memory_max_mb: updates.memory_max_mb.unwrap_or(current.memory_max_mb),
        concurrent_downloads: updates.concurrent_downloads.unwrap_or(current.concurrent_downloads),
        close_launcher_on_game_start: updates.close_launcher_on_game_start.unwrap_or(current.close_launcher_on_game_start),
        reopen_launcher_on_game_close: updates.reopen_launcher_on_game_close.unwrap_or(current.reopen_launcher_on_game_close),
        show_snapshots: updates.show_snapshots.unwrap_or(current.show_snapshots),
        show_old_versions: updates.show_old_versions.unwrap_or(current.show_old_versions),
        theme: updates.theme.unwrap_or_else(|| current.theme.clone()),
        curseforge_api_key: updates.curseforge_api_key.or_else(|| current.curseforge_api_key.clone()),
    }
}

/// Validate settings values
pub fn validate_settings(settings: &AppSettings) -> Result<(), AppError> {
    // Validate memory settings
    if settings.memory_min_mb > settings.memory_max_mb {
        return Err(AppError::SettingsError(
            "Minimum memory cannot be greater than maximum memory".to_string()
        ));
    }

    if settings.memory_min_mb < 256 {
        return Err(AppError::SettingsError(
            "Minimum memory must be at least 256 MB".to_string()
        ));
    }

    if settings.memory_max_mb > 32768 {
        return Err(AppError::SettingsError(
            "Maximum memory cannot exceed 32 GB".to_string()
        ));
    }

    // Validate concurrent downloads
    if settings.concurrent_downloads < 1 || settings.concurrent_downloads > 16 {
        return Err(AppError::SettingsError(
            "Concurrent downloads must be between 1 and 16".to_string()
        ));
    }

    // Validate instances path exists or can be created
    let instances_path = Path::new(&settings.instances_path);
    if !instances_path.exists() {
        std::fs::create_dir_all(instances_path).map_err(|e| {
            AppError::SettingsError(format!(
                "Cannot create instances directory '{}': {}",
                settings.instances_path, e
            ))
        })?;
    }

    // Validate Java path if specified
    if let Some(ref java_path) = settings.java_path {
        let java_path = Path::new(java_path);
        if !java_path.exists() {
            return Err(AppError::SettingsError(format!(
                "Java path '{}' does not exist",
                java_path.display()
            )));
        }
    }

    Ok(())
}

/// Get the default instances path
pub fn get_default_instances_path() -> String {
    paths::get_instances_dir().to_string_lossy().to_string()
}

/// Detect Java installations on the system
pub fn detect_java_path() -> Option<String> {
    // Try common Java locations
    let candidates = if cfg!(target_os = "windows") {
        vec![
            std::env::var("JAVA_HOME").ok().map(|p| format!("{}\\bin\\java.exe", p)),
            Some("C:\\Program Files\\Java\\jdk-21\\bin\\java.exe".to_string()),
            Some("C:\\Program Files\\Java\\jdk-17\\bin\\java.exe".to_string()),
            Some("C:\\Program Files\\Eclipse Adoptium\\jdk-21\\bin\\java.exe".to_string()),
            Some("C:\\Program Files\\Eclipse Adoptium\\jdk-17\\bin\\java.exe".to_string()),
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            std::env::var("JAVA_HOME").ok().map(|p| format!("{}/bin/java", p)),
            Some("/Library/Java/JavaVirtualMachines/temurin-21.jdk/Contents/Home/bin/java".to_string()),
            Some("/Library/Java/JavaVirtualMachines/temurin-17.jdk/Contents/Home/bin/java".to_string()),
            Some("/usr/bin/java".to_string()),
        ]
    } else {
        // Linux
        vec![
            std::env::var("JAVA_HOME").ok().map(|p| format!("{}/bin/java", p)),
            Some("/usr/lib/jvm/java-21-openjdk/bin/java".to_string()),
            Some("/usr/lib/jvm/java-17-openjdk/bin/java".to_string()),
            Some("/usr/bin/java".to_string()),
        ]
    };

    for candidate in candidates.into_iter().flatten() {
        if Path::new(&candidate).exists() {
            return Some(candidate);
        }
    }

    // Try to find java in PATH
    which_java()
}

/// Find java executable in PATH
fn which_java() -> Option<String> {
    let java_name = if cfg!(target_os = "windows") {
        "java.exe"
    } else {
        "java"
    };

    std::env::var("PATH").ok().and_then(|path| {
        let separator = if cfg!(target_os = "windows") { ';' } else { ':' };
        for dir in path.split(separator) {
            let candidate = Path::new(dir).join(java_name);
            if candidate.exists() {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
        None
    })
}
