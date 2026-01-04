use crate::error::AppError;
use crate::models::{
    CachedFileHash, ContentType, DetectedCurseForgeProject, DetectedMod, DetectedModrinthProject,
    ScanCache, ScanResult,
};
use crate::services::{curseforge_service, modrinth_service};
use crate::state::AppState;
use crate::utils::hash::hash_files_parallel;
use crate::utils::paths::get_instance_game_dir_with_base;
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Get the content directory for a content type
fn get_content_dir(game_dir: &PathBuf, content_type: &ContentType) -> PathBuf {
    match content_type {
        ContentType::Mod => game_dir.join("mods"),
        ContentType::Shader => game_dir.join("shaderpacks"),
        ContentType::ResourcePack => game_dir.join("resourcepacks"),
    }
}

/// Get the disabled subfolder for a content directory
fn get_disabled_dir(content_dir: &PathBuf) -> PathBuf {
    content_dir.join("disabled")
}

/// Get the cache file path for a content directory
fn get_cache_path(content_dir: &PathBuf, content_type: &ContentType) -> PathBuf {
    let cache_name = match content_type {
        ContentType::Mod => ".etlauncher_mods_cache.json",
        ContentType::Shader => ".etlauncher_shaders_cache.json",
        ContentType::ResourcePack => ".etlauncher_resourcepacks_cache.json",
    };
    content_dir.join(cache_name)
}

/// Get the file extension for a content type
fn get_content_extension(content_type: &ContentType) -> &'static str {
    match content_type {
        ContentType::Mod => "jar",
        ContentType::Shader => "zip",
        ContentType::ResourcePack => "zip",
    }
}

/// Load scan cache from disk
fn load_cache(content_dir: &PathBuf, content_type: &ContentType) -> Option<ScanCache> {
    let cache_path = get_cache_path(content_dir, content_type);
    if cache_path.exists() {
        fs::read_to_string(&cache_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    } else {
        None
    }
}

/// Save scan cache to disk
fn save_cache(content_dir: &PathBuf, content_type: &ContentType, cache: &ScanCache) {
    let cache_path = get_cache_path(content_dir, content_type);
    if let Ok(json) = serde_json::to_string_pretty(cache) {
        let _ = fs::write(&cache_path, json);
    }
}

/// Collect files from a directory, using cache where possible
fn collect_files_from_dir(
    dir: &PathBuf,
    expected_ext: &str,
    cache: &ScanCache,
    is_disabled: bool,
) -> (Vec<PathBuf>, HashMap<String, (CachedFileHash, bool)>, HashMap<String, bool>) {
    let mut file_paths: Vec<PathBuf> = vec![];
    let mut cached_results: HashMap<String, (CachedFileHash, bool)> = HashMap::new();
    let mut current_files: HashMap<String, bool> = HashMap::new();

    if !dir.exists() {
        return (file_paths, cached_results, current_files);
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.eq_ignore_ascii_case(expected_ext) {
                        let filename = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();

                        // Use a unique key that includes disabled status
                        let cache_key = if is_disabled {
                            format!("disabled/{}", filename)
                        } else {
                            filename.clone()
                        };

                        current_files.insert(cache_key.clone(), true);

                        // Check if we have a valid cached hash
                        if let Ok(metadata) = fs::metadata(&path) {
                            let modified_time = metadata
                                .modified()
                                .ok()
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_secs() as i64)
                                .unwrap_or(0);

                            if let Some(cached) = cache.files.get(&cache_key) {
                                // Check if file hasn't changed (same size and modification time)
                                if cached.size == metadata.len()
                                    && cached.modified_time == modified_time
                                {
                                    // Use cached hash
                                    cached_results.insert(cache_key, (cached.clone(), is_disabled));
                                    continue;
                                }
                            }
                        }

                        // File needs to be hashed
                        file_paths.push(path);
                    }
                }
            }
        }
    }

    (file_paths, cached_results, current_files)
}

/// Scan an instance's content folder and identify items via Modrinth and CurseForge hash lookup
pub async fn scan_content(
    state: &AppState,
    instance_id: &str,
    content_type: &ContentType,
) -> Result<ScanResult, AppError> {
    let instances_base = state.settings.read().instances_path.clone();
    let game_dir = get_instance_game_dir_with_base(&instances_base, instance_id);
    let content_dir = get_content_dir(&game_dir, content_type);
    let disabled_dir = get_disabled_dir(&content_dir);

    // Check if content folder exists
    if !content_dir.exists() {
        return Ok(ScanResult {
            folder_exists: false,
            items: vec![],
            identified_count: 0,
            unidentified_count: 0,
            scanned_at: Utc::now().timestamp(),
        });
    }

    // Load existing cache
    let mut cache = load_cache(&content_dir, content_type).unwrap_or_default();

    // Get the expected file extension for this content type
    let expected_ext = get_content_extension(content_type);

    // Collect files from both enabled and disabled directories
    let (enabled_paths, enabled_cached, enabled_current) =
        collect_files_from_dir(&content_dir, expected_ext, &cache, false);
    let (disabled_paths, disabled_cached, disabled_current) =
        collect_files_from_dir(&disabled_dir, expected_ext, &cache, true);

    // Merge caches and current files
    let mut cached_results: HashMap<String, (CachedFileHash, bool)> = enabled_cached;
    cached_results.extend(disabled_cached);

    let mut current_files: HashMap<String, bool> = enabled_current;
    current_files.extend(disabled_current);

    // Track which paths are disabled
    let mut path_is_disabled: HashMap<PathBuf, bool> = HashMap::new();
    for path in &enabled_paths {
        path_is_disabled.insert(path.clone(), false);
    }
    for path in &disabled_paths {
        path_is_disabled.insert(path.clone(), true);
    }

    // Merge paths for hashing
    let mut all_paths: Vec<PathBuf> = enabled_paths;
    all_paths.extend(disabled_paths);

    // Step 2: Hash files that need hashing in parallel
    let hash_results = hash_files_parallel(&all_paths);

    // Step 3: Collect all hashes (cached + newly computed)
    // (filename, size, sha512, murmur2, is_disabled)
    let mut content_files: Vec<(String, u64, String, u32, bool)> = vec![];

    // Add cached results
    for (cache_key, (cached, is_disabled)) in &cached_results {
        // Extract just the filename from cache_key (remove "disabled/" prefix if present)
        let filename = if cache_key.starts_with("disabled/") {
            cache_key.strip_prefix("disabled/").unwrap_or(cache_key).to_string()
        } else {
            cache_key.clone()
        };
        content_files.push((
            filename,
            cached.size,
            cached.sha512.clone(),
            cached.murmur2_fingerprint,
            *is_disabled,
        ));
    }

    // Add newly hashed results and update cache
    for result in hash_results {
        match result {
            Ok(hash_result) => {
                let filename = hash_result
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                let is_disabled = path_is_disabled.get(&hash_result.path).copied().unwrap_or(false);

                // Use cache key that includes disabled status
                let cache_key = if is_disabled {
                    format!("disabled/{}", filename)
                } else {
                    filename.clone()
                };

                let modified_time = fs::metadata(&hash_result.path)
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);

                // Update cache
                cache.files.insert(
                    cache_key,
                    CachedFileHash {
                        filename: filename.clone(),
                        size: hash_result.size,
                        modified_time,
                        sha512: hash_result.sha512.clone(),
                        murmur2_fingerprint: hash_result.murmur2_fingerprint,
                    },
                );

                content_files.push((
                    filename,
                    hash_result.size,
                    hash_result.sha512,
                    hash_result.murmur2_fingerprint,
                    is_disabled,
                ));
            }
            Err(e) => {
                eprintln!("Failed to hash file: {}", e);
            }
        }
    }

    // Remove stale entries from cache (files that no longer exist)
    cache.files.retain(|k, _| current_files.contains_key(k));
    cache.last_scan = Utc::now().timestamp();

    // Save updated cache
    save_cache(&content_dir, content_type, &cache);

    // Step 4: Batch lookup hashes via Modrinth API
    let hashes: Vec<String> = content_files.iter().map(|(_, _, h, _, _)| h.clone()).collect();
    let modrinth_results = modrinth_service::get_versions_from_hashes(&state.http_client, &hashes)
        .await
        .unwrap_or_default();

    // Step 5: Batch lookup fingerprints via CurseForge API (if API key configured)
    // Note: CurseForge fingerprinting may only work well for mods
    let fingerprints: Vec<u32> = content_files.iter().map(|(_, _, _, fp, _)| *fp).collect();
    let curseforge_results = if let Some(api_key) = &state.get_settings().curseforge_api_key {
        curseforge_service::get_files_from_fingerprints(&state.http_client, api_key, &fingerprints)
            .await
            .unwrap_or_default()
    } else {
        HashMap::new()
    };

    // Step 6: Build detected items list
    let mut items: Vec<DetectedMod> = vec![];
    let mut identified_count = 0u32;
    let mut unidentified_count = 0u32;

    for (filename, size, hash, murmur2, is_disabled) in content_files {
        let modrinth_project = modrinth_results.get(&hash).map(|version| {
            DetectedModrinthProject {
                project_id: version.project_id.clone(),
                slug: String::new(),
                name: version.name.clone(),
                version_id: version.id.clone(),
                version_number: version.version_number.clone(),
            }
        });

        let curseforge_project = curseforge_results.get(&murmur2).map(|match_info| {
            DetectedCurseForgeProject {
                project_id: match_info.mod_id,
                file_id: match_info.file_id,
                name: match_info.file_name.clone(),
                filename: match_info.file_name.clone(),
            }
        });

        let is_identified = modrinth_project.is_some() || curseforge_project.is_some();

        if is_identified {
            identified_count += 1;
        } else {
            unidentified_count += 1;
        }

        items.push(DetectedMod {
            filename,
            size,
            sha512: hash,
            murmur2_fingerprint: murmur2,
            modrinth_project,
            curseforge_project,
            is_identified,
            is_disabled,
        });
    }

    Ok(ScanResult {
        folder_exists: true,
        items,
        identified_count,
        unidentified_count,
        scanned_at: Utc::now().timestamp(),
    })
}

/// Scan an instance's mods folder (convenience wrapper)
pub async fn scan_mods(state: &AppState, instance_id: &str) -> Result<ScanResult, AppError> {
    scan_content(state, instance_id, &ContentType::Mod).await
}

/// Uninstall content by filename (delete the file)
pub fn uninstall_by_filename(
    state: &AppState,
    instance_id: &str,
    filename: &str,
    content_type: &ContentType,
) -> Result<(), AppError> {
    let instances_base = state.settings.read().instances_path.clone();
    let game_dir = get_instance_game_dir_with_base(&instances_base, instance_id);
    let content_dir = get_content_dir(&game_dir, content_type);

    // Check both enabled and disabled folders
    let file_path = content_dir.join(filename);
    let disabled_path = get_disabled_dir(&content_dir).join(filename);

    if file_path.exists() {
        fs::remove_file(&file_path)?;
    } else if disabled_path.exists() {
        fs::remove_file(&disabled_path)?;
    }

    Ok(())
}

/// Disable content by moving files to the disabled subfolder
pub fn disable_content(
    state: &AppState,
    instance_id: &str,
    filenames: &[String],
    content_type: &ContentType,
) -> Result<(), AppError> {
    let instances_base = state.settings.read().instances_path.clone();
    let game_dir = get_instance_game_dir_with_base(&instances_base, instance_id);
    let content_dir = get_content_dir(&game_dir, content_type);
    let disabled_dir = get_disabled_dir(&content_dir);

    // Create disabled folder if it doesn't exist
    if !disabled_dir.exists() {
        fs::create_dir_all(&disabled_dir)?;
    }

    for filename in filenames {
        let source = content_dir.join(filename);
        let dest = disabled_dir.join(filename);

        if source.exists() {
            fs::rename(&source, &dest)?;
        }
    }

    Ok(())
}

/// Enable content by moving files from the disabled subfolder back to the main folder
pub fn enable_content(
    state: &AppState,
    instance_id: &str,
    filenames: &[String],
    content_type: &ContentType,
) -> Result<(), AppError> {
    let instances_base = state.settings.read().instances_path.clone();
    let game_dir = get_instance_game_dir_with_base(&instances_base, instance_id);
    let content_dir = get_content_dir(&game_dir, content_type);
    let disabled_dir = get_disabled_dir(&content_dir);

    for filename in filenames {
        let source = disabled_dir.join(filename);
        let dest = content_dir.join(filename);

        if source.exists() {
            fs::rename(&source, &dest)?;
        }
    }

    Ok(())
}
