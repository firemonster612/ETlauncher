use std::path::PathBuf;

/// Get the application data directory
pub fn get_app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("etlauncher")
}

/// Get the config file path
pub fn get_config_path() -> PathBuf {
    get_app_data_dir().join("config.json")
}

/// Get the accounts file path
pub fn get_accounts_path() -> PathBuf {
    get_app_data_dir().join("accounts.json")
}

/// Get the instances directory
pub fn get_instances_dir() -> PathBuf {
    get_app_data_dir().join("instances")
}

/// Get the cache directory
pub fn get_cache_dir() -> PathBuf {
    get_app_data_dir().join("cache")
}

/// Get the versions cache directory
pub fn get_versions_cache_dir() -> PathBuf {
    get_cache_dir().join("versions")
}

/// Get the assets directory
pub fn get_assets_dir() -> PathBuf {
    get_cache_dir().join("assets")
}

/// Get the libraries directory
pub fn get_libraries_dir() -> PathBuf {
    get_cache_dir().join("libraries")
}

/// Get the Java installations directory
pub fn get_java_dir() -> PathBuf {
    get_app_data_dir().join("java")
}

/// Get the Java manifest file path
pub fn get_java_manifest_path() -> PathBuf {
    get_java_dir().join("manifest.json")
}

/// Get a specific instance directory by ID
pub fn get_instance_dir(instance_id: &str) -> PathBuf {
    get_instances_dir().join(instance_id)
}

/// Get the game directory for an instance
pub fn get_instance_game_dir(instance_id: &str) -> PathBuf {
    get_instance_dir(instance_id).join(".minecraft")
}

/// Get the natives directory for an instance
pub fn get_instance_natives_dir(instance_id: &str) -> PathBuf {
    get_instance_dir(instance_id).join("natives")
}

/// Get instances directory with custom base path
pub fn get_instances_dir_with_base(base: &str) -> PathBuf {
    PathBuf::from(base)
}

/// Get instance directory with custom base path
pub fn get_instance_dir_with_base(base: &str, instance_id: &str) -> PathBuf {
    get_instances_dir_with_base(base).join(instance_id)
}

/// Get game directory for instance with custom base path
pub fn get_instance_game_dir_with_base(base: &str, instance_id: &str) -> PathBuf {
    get_instance_dir_with_base(base, instance_id).join(".minecraft")
}

/// Get natives directory for instance with custom base path
pub fn get_instance_natives_dir_with_base(base: &str, instance_id: &str) -> PathBuf {
    get_instance_dir_with_base(base, instance_id).join("natives")
}

/// Get the resource pool directory
pub fn get_resource_pool_dir() -> PathBuf {
    get_cache_dir().join("resources")
}

/// Get the resource pool directory for a specific content type
pub fn get_resource_pool_dir_for_type(content_type: &crate::models::ContentType) -> PathBuf {
    let type_dir = match content_type {
        crate::models::ContentType::Mod => "mods",
        crate::models::ContentType::Shader => "shaderpacks",
        crate::models::ContentType::ResourcePack => "resourcepacks",
        crate::models::ContentType::Datapack => "datapacks",
        crate::models::ContentType::World => "worlds",
    };
    get_resource_pool_dir().join(type_dir)
}

/// Get the pool index file path
pub fn get_pool_index_path() -> PathBuf {
    get_resource_pool_dir().join("pool_index.json")
}

/// Get the path to a pooled resource by its hash
pub fn get_pooled_resource_path(
    content_type: &crate::models::ContentType,
    sha512: &str,
    original_filename: &str,
) -> PathBuf {
    // Use hash as filename but preserve extension for compatibility
    let extension = std::path::Path::new(original_filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jar");
    get_resource_pool_dir_for_type(content_type).join(format!("{}.{}", sha512, extension))
}

/// Ensure all required directories exist
pub fn ensure_directories() -> std::io::Result<()> {
    std::fs::create_dir_all(get_app_data_dir())?;
    std::fs::create_dir_all(get_instances_dir())?;
    std::fs::create_dir_all(get_versions_cache_dir())?;
    std::fs::create_dir_all(get_assets_dir().join("indexes"))?;
    std::fs::create_dir_all(get_assets_dir().join("objects"))?;
    std::fs::create_dir_all(get_libraries_dir())?;
    std::fs::create_dir_all(get_java_dir())?;
    // Create resource pool directories
    std::fs::create_dir_all(get_resource_pool_dir_for_type(
        &crate::models::ContentType::Mod,
    ))?;
    std::fs::create_dir_all(get_resource_pool_dir_for_type(
        &crate::models::ContentType::Shader,
    ))?;
    std::fs::create_dir_all(get_resource_pool_dir_for_type(
        &crate::models::ContentType::ResourcePack,
    ))?;
    Ok(())
}
