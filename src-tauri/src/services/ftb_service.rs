use crate::error::AppError;
use crate::models::{
    LoaderType, Modpack, ModpackFile, ModpackSearchParams, ModpackSearchResult,
    ModpackVersion,
};
use crate::models::instance::ModpackPlatform;
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FtbVersionManifest {
    pub id: u64,
    pub name: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub updated: u64,
    pub targets: Vec<FtbTarget>,
    pub files: Vec<FtbFile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FtbTarget {
    pub id: u64,
    pub name: String,
    #[serde(rename = "type")]
    pub target_type: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FtbFile {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub url: String,
    pub sha1: Option<String>,
    pub size: u64,
    #[serde(rename = "type")]
    pub file_type: String,
    pub updated: u64,
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
    (LoaderType::Vanilla, None)
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
            format!("{}/modpack/search/100?term={}", FTB_API_BASE, urlencoding::encode(query))
        } else {
            format!("{}/modpack/popular/installs/100", FTB_API_BASE)
        }
    } else {
        format!("{}/modpack/popular/installs/100", FTB_API_BASE)
    };

    eprintln!("[ftb] Search URL: {}", url);
    let search_response: FtbPackSearchResponse = match tokio::time::timeout(
        std::time::Duration::from_secs(15),
        client.get(&url).send()
    ).await {
        Ok(Ok(response)) => {
            let resp: FtbPackSearchResponse = response.error_for_status()?.json().await?;
            eprintln!("[ftb] Search response: {} packs, {} curseforge, total={}", resp.packs.len(), resp.curseforge.len(), resp.total);
            resp
        },
        Ok(Err(e)) => return Err(AppError::ApiError(format!("FTB search request failed: {}", e))),
        Err(_) => return Err(AppError::ApiError("FTB search request timed out".to_string())),
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
    eprintln!("[ftb] Fetching {} pack details in parallel", pack_ids.len());
    let futures: Vec<_> = pack_ids
        .into_iter()
        .map(|pack_id| {
            let client = client.clone();
            async move {
                match tokio::time::timeout(
                    std::time::Duration::from_secs(10),
                    get_pack_details(&client, pack_id)
                ).await {
                    Ok(Ok(pack)) => Some(pack),
                    Ok(Err(e)) => {
                        eprintln!("[ftb] Failed to fetch pack {}: {}", pack_id, e);
                        None
                    }
                    Err(_) => {
                        eprintln!("[ftb] Timeout fetching pack {}", pack_id);
                        None
                    }
                }
            }
        })
        .collect();

    let results = futures::future::join_all(futures).await;
    let modpacks: Vec<Modpack> = results.into_iter().flatten().collect();
    eprintln!("[ftb] Got {} modpacks", modpacks.len());

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
        banner_url: pack.art.iter().find(|a| a.art_type == "splash").map(|a| a.url.clone()),
        downloads: pack.installs,
        platform: ModpackPlatform::FTB,
        categories: pack.tags.into_iter().map(|t| t.name).collect(),
        mc_versions: vec![], // Would need to fetch version details
        loaders: vec![], // Would need to fetch version details
        latest_version: None, // Would need to fetch version details
        url: Some(format!("https://www.feed-the-beast.com/modpacks/{}", pack.id)),
        updated_at: if pack.updated > 0 { Some(pack.updated as i64) } else { None },
        created_at: if pack.released > 0 { Some(pack.released as i64) } else { None },
    })
}

/// Get a modpack by ID
pub async fn get_modpack(client: &Client, pack_id: &str) -> Result<Modpack, AppError> {
    let id: u64 = pack_id.parse().map_err(|_| AppError::ModpackNotFound(pack_id.to_string()))?;
    get_pack_details(client, id).await
}

/// Get versions for a modpack
pub async fn get_modpack_versions(
    client: &Client,
    pack_id: &str,
) -> Result<Vec<ModpackVersion>, AppError> {
    let id: u64 = pack_id.parse().map_err(|_| AppError::ModpackNotFound(pack_id.to_string()))?;

    // First get pack info to get version IDs with timeout
    let url = format!("{}/modpack/{}", FTB_API_BASE, id);
    let pack: FtbPack = match tokio::time::timeout(
        std::time::Duration::from_secs(15),
        client.get(&url).send()
    ).await {
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
                    get_version_details(&client, pack_id, version_id)
                ).await {
                    Ok(Ok(version)) => Some(version),
                    _ => {
                        // Return a minimal version entry on failure
                        Some(ModpackVersion {
                            id: version_id.to_string(),
                            name: version_name,
                            mc_version: String::new(),
                            loader_type: LoaderType::Vanilla,
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
    versions.sort_by(|a, b| {
        b.released_at.unwrap_or(0).cmp(&a.released_at.unwrap_or(0))
    });

    Ok(versions)
}

/// Get version details
pub async fn get_version_details(
    client: &Client,
    pack_id: u64,
    version_id: u64,
) -> Result<ModpackVersion, AppError> {
    let url = format!("{}/modpack/{}/{}", FTB_API_BASE, pack_id, version_id);

    let manifest: FtbVersionManifest = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let mc_version = get_mc_version(&manifest.targets).unwrap_or_default();
    let (loader_type, loader_version) = get_loader_info(&manifest.targets);

    // Get the main modpack file
    let files: Vec<ModpackFile> = manifest
        .files
        .into_iter()
        .filter(|f| f.file_type == "mod" || f.file_type == "config" || f.file_type == "modpack")
        .map(|f| ModpackFile {
            url: f.url,
            hash: f.sha1,
            hash_algorithm: Some("sha1".to_string()),
            size: f.size,
            path: format!("{}/{}", f.path, f.name),
            required: true,
        })
        .collect();

    Ok(ModpackVersion {
        id: manifest.id.to_string(),
        name: manifest.name,
        mc_version,
        loader_type,
        loader_version,
        changelog: None,
        released_at: Some(manifest.updated as i64),
        downloads: None,
        files,
    })
}
