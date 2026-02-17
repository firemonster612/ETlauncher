use crate::app_error;
use crate::error::AppError;
use crate::models::instance::ModpackPlatform;
use crate::models::{
    LoaderType, Modpack, ModpackContentType, ModpackFile, ModpackMod, ModpackSearchParams,
    ModpackSearchResult, ModpackVersion,
};
use reqwest::Client;
use serde::Deserialize;

const FTB_API_BASE: &str = "https://api.modpacks.ch/public";

// ============================================================================
// FTB API Response Types
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct FtbPackSearchResponse {
    #[serde(default)]
    pub packs: Vec<u64>,
    #[serde(default)]
    pub curseforge: Vec<u64>,
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub limit: u32,
    #[serde(default)]
    pub refreshed: u64,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(rename = "type")]
    #[serde(default)]
    pub response_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FtbPack {
    pub id: u64,
    pub name: String,
    pub synopsis: String,
    pub description: String,
    pub art: Vec<FtbArt>,
    pub authors: Vec<FtbAuthor>,
    pub versions: Vec<FtbVersionInfo>,
    pub installs: u64,
    pub plays: u64,
    pub tags: Vec<FtbTag>,
    #[serde(default)]
    pub updated: u64,
    #[serde(default)]
    pub released: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FtbArt {
    pub width: u32,
    pub height: u32,
    pub url: String,
    #[serde(rename = "type")]
    pub art_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FtbAuthor {
    pub id: u64,
    pub name: String,
    #[serde(rename = "type")]
    pub author_type: String,
    pub website: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FtbVersionInfo {
    pub id: u64,
    pub name: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub updated: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FtbTag {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FtbVersionManifest {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "type")]
    #[serde(default)]
    pub version_type: String,
    #[serde(default)]
    pub updated: u64,
    #[serde(default)]
    pub targets: Vec<FtbTarget>,
    #[serde(default)]
    pub files: Vec<FtbFile>,
    // Error response fields - FTB API returns these instead of manifest for invalid packs
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FtbTarget {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "type")]
    #[serde(default)]
    pub target_type: String,
    #[serde(default)]
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FtbFile {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub size: u64,
    #[serde(rename = "type")]
    #[serde(default)]
    pub file_type: String,
    #[serde(default)]
    pub updated: u64,
    #[serde(default)]
    pub curseforge: Option<FtbCurseForgeRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FtbCurseForgeRef {
    pub project: u64,
    #[allow(dead_code)]
    pub file: u64,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract Minecraft version from targets
fn get_mc_version(targets: &[FtbTarget]) -> Option<String> {
    targets
        .iter()
        .find(|t| t.target_type.to_lowercase() == "game" && t.name.to_lowercase() == "minecraft")
        .map(|t| t.version.clone())
}

/// Extract loader info from targets
fn get_loader_info(targets: &[FtbTarget]) -> (LoaderType, Option<String>) {
    for target in targets {
        if target.target_type.to_lowercase() == "modloader" {
            let loader_type = match target.name.to_lowercase().as_str() {
                "forge" => LoaderType::Forge,
                "neoforge" => LoaderType::NeoForge,
                "fabric" => LoaderType::Fabric,
                "quilt" => LoaderType::Quilt,
                _ => continue,
            };
            return (loader_type, Some(target.version.clone()));
        }
    }
    // Return Unknown instead of Vanilla when loader info is not available
    (LoaderType::Unknown, None)
}

/// Get icon URL from art
fn get_icon_url(art: &[FtbArt]) -> Option<String> {
    art.iter()
        .find(|a| a.art_type == "square" || a.art_type == "logo")
        .or_else(|| art.first())
        .map(|a| a.url.clone())
}

/// Get author name
fn get_author(authors: &[FtbAuthor]) -> String {
    authors
        .iter()
        .find(|a| a.author_type == "team" || a.author_type == "owner")
        .or_else(|| authors.first())
        .map(|a| a.name.clone())
        .unwrap_or_else(|| "Unknown".to_string())
}

// ============================================================================
// Public API Functions
// ============================================================================

/// Search for modpacks on FTB
pub async fn search_modpacks(
    client: &Client,
    params: &ModpackSearchParams,
) -> Result<ModpackSearchResult, AppError> {
    let page = params.page.unwrap_or(0);
    let page_size = params.page_size.unwrap_or(20).min(50);

    // FTB search endpoint - use limit of 100 to get more packs
    // Note: FTB API requires at least 3 characters for search, otherwise returns empty
    let url = if let Some(ref query) = params.query {
        if query.len() >= 3 {
            format!(
                "{}/modpack/search/100?term={}",
                FTB_API_BASE,
                urlencoding::encode(query)
            )
        } else {
            format!("{}/modpack/popular/installs/100", FTB_API_BASE)
        }
    } else {
        format!("{}/modpack/popular/installs/100", FTB_API_BASE)
    };

    app_error!("[ftb] Search URL: {}", url);
    let search_response: FtbPackSearchResponse =
        match tokio::time::timeout(std::time::Duration::from_secs(15), client.get(&url).send())
            .await
        {
            Ok(Ok(response)) => {
                let resp: FtbPackSearchResponse = response.error_for_status()?.json().await?;
                app_error!(
                    "[ftb] Search response: {} packs, {} curseforge, total={}",
                    resp.packs.len(),
                    resp.curseforge.len(),
                    resp.total
                );
                resp
            }
            Ok(Err(e)) => {
                return Err(AppError::ApiError(format!(
                    "FTB search request failed: {}",
                    e
                )))
            }
            Err(_) => {
                return Err(AppError::ApiError(
                    "FTB search request timed out".to_string(),
                ))
            }
        };

    // Fetch individual pack details for the visible page in parallel
    let start = (page * page_size) as usize;
    let pack_ids: Vec<_> = search_response
        .packs
        .into_iter()
        .skip(start)
        .take(page_size as usize)
        .collect();

    // Fetch all pack details in parallel with timeout
    app_error!("[ftb] Fetching {} pack details in parallel", pack_ids.len());
    let futures: Vec<_> = pack_ids
        .into_iter()
        .map(|pack_id| {
            let client = client.clone();
            async move {
                match tokio::time::timeout(
                    std::time::Duration::from_secs(10),
                    get_pack_details(&client, pack_id),
                )
                .await
                {
                    Ok(Ok(pack)) => Some(pack),
                    Ok(Err(e)) => {
                        app_error!("[ftb] Failed to fetch pack {}: {}", pack_id, e);
                        None
                    }
                    Err(_) => {
                        app_error!("[ftb] Timeout fetching pack {}", pack_id);
                        None
                    }
                }
            }
        })
        .collect();

    let results = futures::future::join_all(futures).await;
    let modpacks: Vec<Modpack> = results.into_iter().flatten().collect();
    app_error!("[ftb] Got {} modpacks", modpacks.len());

    Ok(ModpackSearchResult {
        modpacks,
        total_count: search_response.total,
        page,
        page_size,
    })
}

/// Get pack details by ID
async fn get_pack_details(client: &Client, pack_id: u64) -> Result<Modpack, AppError> {
    let url = format!("{}/modpack/{}", FTB_API_BASE, pack_id);

    let pack: FtbPack = client
        .get(&url)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| {
            if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                AppError::ModpackNotFound(pack_id.to_string())
            } else {
                AppError::ApiError(e.to_string())
            }
        })?
        .json()
        .await?;

    Ok(Modpack {
        id: pack.id.to_string(),
        slug: pack.name.to_lowercase().replace(' ', "-"),
        name: pack.name,
        author: get_author(&pack.authors),
        description: pack.synopsis,
        body: Some(pack.description),
        icon_url: get_icon_url(&pack.art),
        banner_url: pack
            .art
            .iter()
            .find(|a| a.art_type == "splash")
            .map(|a| a.url.clone()),
        downloads: pack.installs,
        platform: ModpackPlatform::FTB,
        categories: pack.tags.into_iter().map(|t| t.name).collect(),
        gallery: Vec::new(),
        mc_versions: vec![],
        loaders: vec![LoaderType::Unknown], // FTB API doesn't include loader in pack list
        latest_version: None,
        url: Some(format!(
            "https://www.feed-the-beast.com/modpacks/{}",
            pack.id
        )),
        updated_at: if pack.updated > 0 {
            Some(pack.updated as i64)
        } else {
            None
        },
        created_at: if pack.released > 0 {
            Some(pack.released as i64)
        } else {
            None
        },
        external_links: None,
        team_members: Vec::new(),
        followers: None,
        client_side: None,
        server_side: None,
    })
}

/// Get a modpack by ID
pub async fn get_modpack(client: &Client, pack_id: &str) -> Result<Modpack, AppError> {
    let id: u64 = pack_id
        .parse()
        .map_err(|_| AppError::ModpackNotFound(pack_id.to_string()))?;
    get_pack_details(client, id).await
}

/// Get versions for a modpack
pub async fn get_modpack_versions(
    client: &Client,
    pack_id: &str,
) -> Result<Vec<ModpackVersion>, AppError> {
    let id: u64 = pack_id
        .parse()
        .map_err(|_| AppError::ModpackNotFound(pack_id.to_string()))?;

    // First get pack info to get version IDs with timeout
    let url = format!("{}/modpack/{}", FTB_API_BASE, id);
    let pack: FtbPack =
        match tokio::time::timeout(std::time::Duration::from_secs(15), client.get(&url).send())
            .await
        {
            Ok(Ok(response)) => response.error_for_status()?.json().await?,
            Ok(Err(e)) => return Err(AppError::ApiError(format!("FTB API request failed: {}", e))),
            Err(_) => return Err(AppError::ApiError("FTB API request timed out".to_string())),
        };

    // Fetch version details in parallel with timeout
    // Reverse to get newest versions first (API returns oldest first)
    let versions_to_fetch: Vec<_> = pack.versions.iter().rev().take(10).collect();

    let futures: Vec<_> = versions_to_fetch
        .iter()
        .map(|version_info| {
            let client = client.clone();
            let pack_id = id;
            let version_id = version_info.id;
            let version_name = version_info.name.clone();

            async move {
                match tokio::time::timeout(
                    std::time::Duration::from_secs(10),
                    get_version_details(&client, pack_id, version_id),
                )
                .await
                {
                    Ok(Ok(version)) => Some(version),
                    _ => {
                        // Return a minimal version entry on failure
                        Some(ModpackVersion {
                            id: version_id.to_string(),
                            name: version_name,
                            mc_version: String::new(),
                            loader_type: LoaderType::Unknown,
                            loader_version: None,
                            changelog: None,
                            released_at: None,
                            downloads: None,
                            files: vec![],
                        })
                    }
                }
            }
        })
        .collect();

    let results = futures::future::join_all(futures).await;
    let mut versions: Vec<ModpackVersion> = results.into_iter().flatten().collect();

    // Sort newest first by released_at timestamp
    versions.sort_by(|a, b| b.released_at.unwrap_or(0).cmp(&a.released_at.unwrap_or(0)));

    Ok(versions)
}

/// Get version details
/// Returns a ModpackVersion with whatever data is available.
/// If JSON parsing fails completely, returns a version with empty fields.
/// The caller should check if files are empty and handle accordingly.
pub async fn get_version_details(
    client: &Client,
    pack_id: u64,
    version_id: u64,
) -> Result<ModpackVersion, AppError> {
    let url = format!("{}/modpack/{}/{}", FTB_API_BASE, pack_id, version_id);

    // Try to fetch and parse the manifest, fall back to default on any error
    let manifest: FtbVersionManifest = match client.get(&url).send().await {
        Ok(response) => match response.error_for_status() {
            Ok(resp) => match resp.json().await {
                Ok(m) => m,
                Err(e) => {
                    app_error!("[ftb] Failed to parse version manifest JSON: {}", e);
                    FtbVersionManifest::default()
                }
            },
            Err(e) => {
                app_error!("[ftb] API returned error status: {}", e);
                FtbVersionManifest::default()
            }
        },
        Err(e) => {
            app_error!("[ftb] Failed to fetch version manifest: {}", e);
            FtbVersionManifest::default()
        }
    };

    // Check if FTB API returned an error response
    if let Some(ref status) = manifest.status {
        if status == "error" {
            let msg = manifest
                .message
                .clone()
                .unwrap_or_else(|| "Unknown FTB API error".to_string());
            app_error!("[ftb] API error response: {}", msg);
            // Don't fail - return empty manifest and let caller handle
        }
    }

    let mc_version = get_mc_version(&manifest.targets).unwrap_or_default();
    let (loader_type, loader_version) = get_loader_info(&manifest.targets);

    // Get the modpack files, filtering out entries with empty URLs
    let files: Vec<ModpackFile> = manifest
        .files
        .into_iter()
        .filter(|f| {
            !f.url.is_empty()
                && (f.file_type == "mod"
                    || f.file_type == "config"
                    || f.file_type == "modpack"
                    || f.file_type.is_empty())
        })
        .map(|f| ModpackFile {
            url: f.url,
            hash: f.sha1,
            hash_algorithm: Some("sha1".to_string()),
            size: f.size,
            path: format!("{}/{}", f.path, f.name),
            required: true,
        })
        .collect();

    // Note: We don't fail on empty files here - let the caller decide what to do
    // The caller can check if files.is_empty() and mc_version.is_empty() to detect legacy packs

    Ok(ModpackVersion {
        id: if manifest.id > 0 {
            manifest.id.to_string()
        } else {
            version_id.to_string()
        },
        name: if manifest.name.is_empty() {
            format!("Version {}", version_id)
        } else {
            manifest.name
        },
        mc_version,
        loader_type,
        loader_version,
        changelog: None,
        released_at: if manifest.updated > 0 {
            Some(manifest.updated as i64)
        } else {
            None
        },
        downloads: None,
        files,
    })
}

/// Get a mod list for an FTB modpack version (best-effort)
pub async fn get_modpack_mods(
    client: &Client,
    pack_id: &str,
    version_id: &str,
) -> Result<Vec<ModpackMod>, AppError> {
    let pack_id_num: u64 = pack_id
        .parse()
        .map_err(|_| AppError::ModpackNotFound(pack_id.to_string()))?;
    let version_id_num: u64 = version_id
        .parse()
        .map_err(|_| AppError::ContentNotFound("Invalid version id".to_string()))?;

    let url = format!(
        "{}/modpack/{}/{}",
        FTB_API_BASE, pack_id_num, version_id_num
    );
    let manifest: FtbVersionManifest = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let mut mods: Vec<ModpackMod> = manifest
        .files
        .into_iter()
        .filter(|f| f.file_type == "mod")
        .map(|f| {
            let url = f
                .curseforge
                .as_ref()
                .map(|cf| {
                    format!(
                        "https://www.curseforge.com/minecraft/mc-mods/{}",
                        cf.project
                    )
                })
                .or_else(|| {
                    if f.url.is_empty() {
                        None
                    } else {
                        Some(f.url.clone())
                    }
                });

            ModpackMod {
                id: f
                    .curseforge
                    .as_ref()
                    .map(|cf| cf.project.to_string())
                    .unwrap_or_else(|| f.id.to_string()),
                name: f.name,
                icon_url: None,
                author: None,
                url,
                content_type: ModpackContentType::default(),
            }
        })
        .collect();

    mods.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(mods)
}
