use crate::app_error;
use crate::app_info;
use crate::error::AppError;
use crate::models::content::{
    ContentPlatform, ContentSource, InstalledContentManifest, MANIFEST_VERSION,
};
use crate::models::{
    CachedFileHash, ContentType, DetectedCurseForgeProject, DetectedMod, DetectedModrinthProject,
    InstalledContent, ScanCache, ScanResult,
};
use crate::services::{
    curseforge_service, manifest_service, modrinth_service, resource_pool_service,
};
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
        ContentType::Datapack => game_dir.join("datapacks"),
        ContentType::World => game_dir.join("saves"),
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
        ContentType::Datapack => ".etlauncher_datapacks_cache.json",
        ContentType::World => ".etlauncher_worlds_cache.json",
    };
    content_dir.join(cache_name)
}

/// Get the file extension for a content type
fn get_content_extension(content_type: &ContentType) -> &'static str {
    match content_type {
        ContentType::Mod => "jar",
        ContentType::Shader => "zip",
        ContentType::ResourcePack => "zip",
        ContentType::Datapack => "zip",
        ContentType::World => "zip",
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
) -> (
    Vec<PathBuf>,
    HashMap<String, (CachedFileHash, bool)>,
    HashMap<String, bool>,
) {
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

/// Scan world folders in the saves directory
/// Worlds are directories containing level.dat, not ZIP files
fn scan_world_folders(saves_dir: &PathBuf) -> Vec<(String, bool)> {
    let mut worlds = Vec::new();

    if !saves_dir.exists() {
        return worlds;
    }

    if let Ok(entries) = fs::read_dir(saves_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Check if this directory contains level.dat (is a valid world)
                let level_dat = path.join("level.dat");
                if level_dat.exists() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        // Worlds can't be "disabled" in the traditional sense
                        worlds.push((name.to_string(), false));
                    }
                }
            }
        }
    }

    worlds
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

    // Load manifest to get dependency info
    let manifest = manifest_service::load_manifest(state, instance_id).unwrap_or_default();
    let manifest_items: Vec<&InstalledContent> = match content_type {
        ContentType::Mod => manifest.mods.iter().collect(),
        ContentType::Shader => manifest.shaders.iter().collect(),
        ContentType::ResourcePack => manifest.resource_packs.iter().collect(),
        ContentType::Datapack => manifest.datapacks.iter().collect(),
        ContentType::World => manifest.worlds.iter().collect(),
    };
    let manifest_by_filename: std::collections::HashMap<&str, &InstalledContent> = manifest_items
        .iter()
        .map(|item| (item.filename.as_str(), *item))
        .collect();

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

    // Special handling for worlds - they are folders, not files
    // Only show worlds that were installed via the launcher (tracked in manifest)
    if *content_type == ContentType::World {
        let world_folders = scan_world_folders(&content_dir);
        let mut items: Vec<DetectedMod> = Vec::new();
        let mut identified_count = 0u32;

        for (folder_name, is_disabled) in world_folders {
            // Only include worlds that are tracked in the manifest
            // This excludes manually created worlds from the "Installed Content" list
            let Some(entry) = manifest_by_filename.get(folder_name.as_str()) else {
                continue;
            };

            let modrinth_project = entry
                .modrinth_id
                .as_ref()
                .map(|id| DetectedModrinthProject {
                    project_id: id.clone(),
                    slug: entry.slug.clone(),
                    name: entry.name.clone(),
                    version_id: entry.version_id.clone(),
                    version_number: entry.version.clone(),
                });
            let curseforge_project = entry.curseforge_id.map(|id| DetectedCurseForgeProject {
                project_id: id as u64,
                file_id: 0,
                name: entry.name.clone(),
                filename: entry.filename.clone(),
                slug: entry.slug.clone(),
            });

            // All manifest-tracked worlds are considered identified
            identified_count += 1;

            items.push(DetectedMod {
                filename: folder_name,
                size: 0, // Worlds are folders, size is not meaningful
                sha512: String::new(),
                murmur2_fingerprint: 0,
                modrinth_project,
                curseforge_project,
                is_identified: true,
                is_disabled,
                is_dependency: entry.is_dependency,
                dependency_of: entry.dependency_of.clone(),
            });
        }

        return Ok(ScanResult {
            folder_exists: true,
            items,
            identified_count,
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
            cache_key
                .strip_prefix("disabled/")
                .unwrap_or(cache_key)
                .to_string()
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

                let is_disabled = path_is_disabled
                    .get(&hash_result.path)
                    .copied()
                    .unwrap_or(false);

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
                app_error!("Failed to hash file: {}", e);
            }
        }
    }

    // Remove stale entries from cache (files that no longer exist)
    cache.files.retain(|k, _| current_files.contains_key(k));
    cache.last_scan = Utc::now().timestamp();

    // Save updated cache
    save_cache(&content_dir, content_type, &cache);

    // Step 4: Batch lookup hashes via Modrinth API
    let hashes: Vec<String> = content_files
        .iter()
        .map(|(_, _, h, _, _)| h.clone())
        .collect();
    let modrinth_results = modrinth_service::get_versions_from_hashes(&state.http_client, &hashes)
        .await
        .unwrap_or_default();

    // Best-effort: fetch project titles/slugs so we can display project name (not version name)
    let mut modrinth_project_ids: Vec<String> = modrinth_results
        .values()
        .map(|v| v.project_id.clone())
        .collect();
    modrinth_project_ids.sort();
    modrinth_project_ids.dedup();

    let modrinth_projects =
        modrinth_service::get_projects_by_ids(&state.http_client, &modrinth_project_ids)
            .await
            .unwrap_or_default();
    let modrinth_project_map: HashMap<String, modrinth_service::ModrinthProjectLite> =
        modrinth_projects
            .into_iter()
            .map(|p| (p.id.clone(), p))
            .collect();

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

    // Best-effort: fetch CurseForge mod names so we can display project name (not file/version name)
    let curseforge_mod_map: HashMap<u64, curseforge_service::CurseForgeMod> = if let Some(api_key) =
        &state.get_settings().curseforge_api_key
    {
        let mut mod_ids: Vec<u32> = curseforge_results
            .values()
            .filter_map(|m| u32::try_from(m.mod_id).ok())
            .collect();
        mod_ids.sort();
        mod_ids.dedup();

        let mut all_mods: Vec<curseforge_service::CurseForgeMod> = Vec::new();
        for chunk in mod_ids.chunks(50) {
            let mut mods = curseforge_service::get_mods_by_ids(&state.http_client, api_key, chunk)
                .await
                .unwrap_or_default();
            all_mods.append(&mut mods);
        }

        all_mods.into_iter().map(|m| (m.id, m)).collect()
    } else {
        HashMap::new()
    };

    // Step 5b: Fetch CurseForge file details to get dependency info
    // This is needed for backfilling dependency_ids for existing instances
    let curseforge_file_map: HashMap<u64, curseforge_service::CurseForgeFile> =
        if let Some(api_key) = &state.get_settings().curseforge_api_key {
            let file_ids: Vec<u64> = curseforge_results.values().map(|m| m.file_id).collect();

            if !file_ids.is_empty() {
                curseforge_service::get_files_by_ids(&state.http_client, api_key, &file_ids)
                    .await
                    .unwrap_or_default()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };

    // Step 6: Load manifest for fallback identification
    // This allows us to identify content that was installed via the launcher
    // but couldn't be matched via API lookups (e.g., CurseForge-exclusive content)
    let manifest = manifest_service::load_manifest(state, instance_id).ok();
    let manifest_content: HashMap<String, &InstalledContent> = manifest
        .as_ref()
        .map(|m| {
            let list: &[InstalledContent] = match content_type {
                ContentType::Mod => &m.mods,
                ContentType::Shader => &m.shaders,
                ContentType::ResourcePack => &m.resource_packs,
                ContentType::Datapack => &m.datapacks,
                ContentType::World => &m.worlds,
            };
            list.iter().map(|c| (c.filename.clone(), c)).collect()
        })
        .unwrap_or_default();

    // Step 7: Build detected items list
    let mut items: Vec<DetectedMod> = vec![];
    let mut identified_count = 0u32;
    let mut unidentified_count = 0u32;

    for (filename, size, hash, murmur2, is_disabled) in &content_files {
        // Try API-based identification first
        let mut modrinth_project = modrinth_results.get(hash).map(|version| {
            let (slug, name) = modrinth_project_map
                .get(&version.project_id)
                .map(|p| (p.slug.clone(), p.title.clone()))
                .unwrap_or_else(|| (String::new(), version.name.clone()));
            DetectedModrinthProject {
                project_id: version.project_id.clone(),
                slug,
                name,
                version_id: version.id.clone(),
                version_number: version.version_number.clone(),
            }
        });

        let mut curseforge_project = curseforge_results.get(murmur2).map(|match_info| {
            let mod_info = curseforge_mod_map.get(&match_info.mod_id);
            let name = mod_info
                .map(|m| m.name.clone())
                .unwrap_or_else(|| match_info.file_name.clone());
            let slug = mod_info.map(|m| m.slug.clone()).unwrap_or_default();
            DetectedCurseForgeProject {
                project_id: match_info.mod_id,
                file_id: match_info.file_id,
                name,
                filename: match_info.file_name.clone(),
                slug,
            }
        });

        // Fallback to manifest-based identification when API lookups fail
        // This handles cases where hash/fingerprint APIs don't return matches
        // (common for shaders, resource packs, or when APIs are down)
        if curseforge_project.is_none() && modrinth_project.is_none() {
            if let Some(manifest_entry) = manifest_content.get(filename) {
                // Use manifest data to create project info
                if let Some(ref mr_id) = manifest_entry.modrinth_id {
                    modrinth_project = Some(DetectedModrinthProject {
                        project_id: mr_id.clone(),
                        slug: manifest_entry.slug.clone(),
                        name: manifest_entry.name.clone(),
                        version_id: manifest_entry.version_id.clone(),
                        version_number: manifest_entry.version.clone(),
                    });
                } else if let Some(cf_id) = manifest_entry.curseforge_id {
                    curseforge_project = Some(DetectedCurseForgeProject {
                        project_id: cf_id as u64,
                        file_id: 0, // We don't store file_id in manifest
                        name: manifest_entry.name.clone(),
                        filename: manifest_entry.filename.clone(),
                        slug: manifest_entry.slug.clone(),
                    });
                }
            }
        }

        let is_identified = modrinth_project.is_some() || curseforge_project.is_some();

        if is_identified {
            identified_count += 1;
        } else {
            unidentified_count += 1;
        }

        // Get dependency info from manifest
        let (is_dependency, dependency_of) = manifest_by_filename
            .get(filename.as_str())
            .map(|item| (item.is_dependency, item.dependency_of.clone()))
            .unwrap_or((false, Vec::new()));

        items.push(DetectedMod {
            filename: filename.clone(),
            size: *size,
            sha512: hash.clone(),
            murmur2_fingerprint: *murmur2,
            modrinth_project,
            curseforge_project,
            is_identified,
            is_disabled: *is_disabled,
            is_dependency,
            dependency_of,
        });
    }

    // Backfill dependency_ids for existing instances from Modrinth/CurseForge data
    // This ensures the reverse dependency lookup works for content installed before
    // the dependency tracking feature was added
    for (filename, _size, hash, murmur2, _is_disabled) in &content_files {
        // Check if this content exists in manifest but is missing dependency_ids
        let Some(manifest_entry) = manifest_by_filename.get(filename.as_str()) else {
            continue;
        };

        if !manifest_entry.dependency_ids.is_empty() {
            continue;
        }

        // Try to get dependency_ids from Modrinth first
        let mut dep_ids: Vec<String> = Vec::new();

        if let Some(version) = modrinth_results.get(hash) {
            // Extract dependency_ids from Modrinth version's dependencies
            dep_ids = version
                .dependencies
                .iter()
                .filter(|d| d.dependency_type == "required")
                .filter_map(|d| d.project_id.as_ref().map(|pid| format!("modrinth:{}", pid)))
                .collect();
        } else if let Some(fp_match) = curseforge_results.get(murmur2) {
            // Try CurseForge - get file details which include dependencies
            if let Some(file_info) = curseforge_file_map.get(&fp_match.file_id) {
                // relation_type 3 = RequiredDependency
                dep_ids = file_info
                    .dependencies
                    .iter()
                    .filter(|d| d.relation_type == 3)
                    .map(|d| format!("curseforge:{}", d.mod_id))
                    .collect();
            }
        }

        if dep_ids.is_empty() {
            continue;
        }

        // Update the manifest with dependency_ids
        if let Ok(true) = manifest_service::update_dependency_ids(
            state,
            instance_id,
            filename,
            content_type,
            dep_ids,
        ) {
            // Reload the manifest entry to get the updated version
            if let Ok(Some(updated)) =
                manifest_service::get_content(state, instance_id, filename, content_type)
            {
                // Now update reverse dependencies for this content
                let _ = manifest_service::update_reverse_dependencies(state, instance_id, &updated);
            }
        }
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

/// Re-scan all content folders for an instance, identify via APIs, and rebuild the manifest.
///
/// This is used to populate manifests for instances that were imported without one
/// (e.g., Prism/MultiMC imports, vanilla .minecraft imports), or to repair a
/// manifest that has gone out of sync.
///
/// Preserves existing manifest entries when the file is already tracked (by filename),
/// so user-added metadata (dependency_of, source, etc.) is not lost for known content.
pub async fn rescan_and_rebuild_manifest(
    state: &AppState,
    instance_id: &str,
) -> Result<RescanResult, AppError> {
    // Register task in the task registry
    let task_id = uuid::Uuid::new_v4().to_string();
    state.task_registry.register(
        task_id.clone(),
        crate::task_registry::TaskType::ContentScan,
        "Scanning instance content".to_string(),
        Some(instance_id.to_string()),
        None,
    );
    state.task_registry.start(&task_id);

    let result = rescan_and_rebuild_manifest_inner(state, instance_id, &task_id).await;

    match result {
        Ok(res) => {
            state.task_registry.complete(&task_id);
            Ok(res)
        }
        Err(e) => {
            state.task_registry.fail(&task_id, e.to_string());
            Err(e)
        }
    }
}

/// Inner implementation for `rescan_and_rebuild_manifest` that can use `?` freely.
async fn rescan_and_rebuild_manifest_inner(
    state: &AppState,
    instance_id: &str,
    task_id: &str,
) -> Result<RescanResult, AppError> {
    let content_types = [
        (ContentType::Mod, "mods", "Scanning mods"),
        (ContentType::Shader, "shaders", "Scanning shaders"),
        (
            ContentType::ResourcePack,
            "resource packs",
            "Scanning resource packs",
        ),
    ];

    // Load existing manifest so we can preserve entries that are already tracked
    state
        .task_registry
        .update_stage(task_id, "Loading existing manifest".to_string());
    let existing_manifest = manifest_service::load_manifest(state, instance_id)?;
    let existing_by_filename: HashMap<String, &InstalledContent> = existing_manifest
        .mods
        .iter()
        .chain(existing_manifest.shaders.iter())
        .chain(existing_manifest.resource_packs.iter())
        .map(|c| (c.filename.clone(), c))
        .collect();

    let mut manifest = InstalledContentManifest {
        manifest_version: MANIFEST_VERSION,
        mods: Vec::new(),
        shaders: Vec::new(),
        resource_packs: Vec::new(),
        datapacks: existing_manifest.datapacks.clone(),
        worlds: existing_manifest.worlds.clone(),
        last_synced_at: Some(Utc::now().timestamp()),
    };

    let mut total = 0u32;
    let mut identified = 0u32;

    for (content_type, label, stage) in &content_types {
        state.task_registry.update_stage(task_id, stage.to_string());

        let scan_result = match scan_content(state, instance_id, content_type).await {
            Ok(result) => result,
            Err(e) => {
                app_error!(
                    "[rescan] Failed to scan {} for instance {}: {}",
                    label,
                    instance_id,
                    e
                );
                continue;
            }
        };

        for item in &scan_result.items {
            // If already in manifest, preserve existing entry but update hashes
            if let Some(existing) = existing_by_filename.get(&item.filename) {
                if existing.modrinth_id.is_some() || existing.curseforge_id.is_some() {
                    // Already tracked and identified — keep it
                    match content_type {
                        ContentType::Mod => manifest.mods.push((*existing).clone()),
                        ContentType::Shader => manifest.shaders.push((*existing).clone()),
                        ContentType::ResourcePack => {
                            manifest.resource_packs.push((*existing).clone())
                        }
                        _ => {}
                    }
                    total += 1;
                    identified += 1;
                    continue;
                }
            }

            // Build new entry from scan results
            let (modrinth_id, curseforge_id, installed_from, name, slug, version, version_id) =
                if let Some(ref mr) = item.modrinth_project {
                    (
                        Some(mr.project_id.clone()),
                        item.curseforge_project
                            .as_ref()
                            .map(|cf| cf.project_id as u32),
                        ContentPlatform::Modrinth,
                        mr.name.clone(),
                        mr.slug.clone(),
                        mr.version_number.clone(),
                        mr.version_id.clone(),
                    )
                } else if let Some(ref cf) = item.curseforge_project {
                    (
                        None,
                        Some(cf.project_id as u32),
                        ContentPlatform::CurseForge,
                        cf.name.clone(),
                        cf.slug.clone(),
                        cf.filename.clone(),
                        cf.file_id.to_string(),
                    )
                } else {
                    (
                        None,
                        None,
                        ContentPlatform::Modrinth,
                        item.filename
                            .trim_end_matches(".jar")
                            .trim_end_matches(".zip")
                            .to_string(),
                        item.filename
                            .trim_end_matches(".jar")
                            .trim_end_matches(".zip")
                            .to_lowercase()
                            .replace(' ', "-"),
                        "unknown".to_string(),
                        "unknown".to_string(),
                    )
                };

            let is_id = modrinth_id.is_some() || curseforge_id.is_some();

            let installed = InstalledContent {
                name,
                slug,
                modrinth_id,
                curseforge_id,
                installed_from,
                version,
                version_id,
                filename: item.filename.clone(),
                content_type: *content_type,
                installed_at: Utc::now().timestamp(),
                is_dependency: item.is_dependency,
                dependency_of: item.dependency_of.clone(),
                dependency_ids: Vec::new(),
                source: ContentSource::UserAdded,
                sha512_hash: Some(item.sha512.clone()),
                murmur2_fingerprint: Some(item.murmur2_fingerprint),
                is_pooled: false,
            };

            match content_type {
                ContentType::Mod => manifest.mods.push(installed),
                ContentType::Shader => manifest.shaders.push(installed),
                ContentType::ResourcePack => manifest.resource_packs.push(installed),
                _ => {}
            }

            total += 1;
            if is_id {
                identified += 1;
            }
        }
    }

    state
        .task_registry
        .update_stage(task_id, "Building manifest".to_string());

    app_info!(
        "[rescan] Rebuilt manifest for instance {}: {} total ({} identified, {} unidentified)",
        instance_id,
        total,
        identified,
        total - identified,
    );

    manifest_service::save_manifest(state, instance_id, &manifest)?;

    Ok(RescanResult {
        total_items: total,
        identified_items: identified,
        unidentified_items: total - identified,
    })
}

/// Result of a rescan operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RescanResult {
    pub total_items: u32,
    pub identified_items: u32,
    pub unidentified_items: u32,
}

/// Check if an instance has content files on disk but an empty or missing manifest.
/// Returns true if the instance likely needs a rescan.
pub fn needs_manifest_rebuild(state: &AppState, instance_id: &str) -> bool {
    let instances_base = state.settings.read().instances_path.clone();
    let game_dir = get_instance_game_dir_with_base(&instances_base, instance_id);

    let manifest = match manifest_service::load_manifest(state, instance_id) {
        Ok(m) => m,
        Err(_) => return false,
    };

    // If manifest already has entries, it doesn't need a rebuild
    let manifest_count =
        manifest.mods.len() + manifest.shaders.len() + manifest.resource_packs.len();
    if manifest_count > 0 {
        return false;
    }

    // Check if there are actual content files on disk
    let mods_dir = game_dir.join("mods");
    if mods_dir.exists() {
        if let Ok(entries) = fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".jar") && !name.starts_with('.') {
                    return true;
                }
            }
        }
    }

    let shaders_dir = game_dir.join("shaderpacks");
    if shaders_dir.exists() {
        if let Ok(entries) = fs::read_dir(&shaders_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if (name.ends_with(".zip") || name.ends_with(".jar")) && !name.starts_with('.') {
                    return true;
                }
            }
        }
    }

    false
}

/// Uninstall content by filename (delete the file/folder and remove from manifest)
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

    // Worlds are folders, other content types are files
    if *content_type == ContentType::World {
        if file_path.exists() && file_path.is_dir() {
            fs::remove_dir_all(&file_path)?;
        }
    } else if file_path.exists() {
        fs::remove_file(&file_path)?;
    } else if disabled_path.exists() {
        fs::remove_file(&disabled_path)?;
    }

    // Also remove from manifest so reinstall works correctly
    manifest_service::remove_content(state, instance_id, filename, content_type)?;

    // Trigger garbage collection if conditions are met (24+ hours since last GC)
    // This runs in the background and cleans up unused pool resources
    resource_pool_service::maybe_run_gc_background(state);

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
