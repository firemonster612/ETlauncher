use crate::error::AppError;
use crate::models::{
    LoaderType, Modpack, ModpackSearchParams, ModpackSearchResult,
    ModpackSortBy, ModpackVersion,
};
use crate::models::instance::ModpackPlatform;
use reqwest::Client;
use serde::Deserialize;

// CDN endpoint (like PrismLauncher uses) - the API is blocked by Cloudflare
const ATLAUNCHER_CDN_BASE: &str = "https://download.nodecdn.net/containers/atl";
// API still needed for full pack details
const ATLAUNCHER_API_BASE: &str = "https://api.atlauncher.com/v1";

// ============================================================================
// ATLauncher CDN Response Types (for pack list)
// ============================================================================

/// Pack info from the CDN packsnew.json endpoint
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlauncherCdnPack {
    pub id: u64,
    #[serde(default)]
    pub position: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub pack_type: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub versions: Vec<AtlauncherCdnVersion>,
    #[serde(default)]
    pub dev_versions: Vec<AtlauncherCdnVersion>,
}

/// Version info from CDN
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlauncherCdnVersion {
    pub version: String,
    pub minecraft: String,
    #[serde(default)]
    pub can_update: bool,
    #[serde(default)]
    pub is_recommended: bool,
    #[serde(default)]
    pub hash: Option<String>,
}

// ============================================================================
// ATLauncher API Response Types (for full pack details)
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct AtlauncherPacksResponse {
    pub error: bool,
    pub code: u32,
    pub message: Option<String>,
    pub data: Vec<AtlauncherPack>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlauncherPack {
    pub id: u64,
    pub name: String,
    pub safe_name: String,
    pub description: Option<String>,
    pub versions: Vec<AtlauncherVersionInfo>,
    #[serde(default)]
    pub create_server: bool,
    #[serde(default)]
    pub leaderboards: bool,
    #[serde(default)]
    pub logging: bool,
    #[serde(default)]
    pub crash_reports: bool,
    #[serde(rename = "type")]
    pub pack_type: Option<String>,
    #[serde(default)]
    pub discord_invite_url: Option<String>,
    #[serde(default)]
    pub support_url: Option<String>,
    #[serde(default)]
    pub website_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlauncherVersionInfo {
    pub version: String,
    pub minecraft: String,
    #[serde(default)]
    pub recommended: bool,
    #[serde(default)]
    pub published: Option<u64>,
    #[serde(default)]
    pub changelog: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AtlauncherPackResponse {
    pub error: bool,
    pub code: u32,
    pub message: Option<String>,
    pub data: AtlauncherPackFull,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlauncherPackFull {
    pub id: u64,
    pub name: String,
    pub safe_name: String,
    pub description: Option<String>,
    pub versions: Vec<AtlauncherVersionFull>,
    #[serde(default)]
    pub discord_invite_url: Option<String>,
    #[serde(default)]
    pub support_url: Option<String>,
    #[serde(default)]
    pub website_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlauncherVersionFull {
    pub version: String,
    pub minecraft: String,
    #[serde(default)]
    pub recommended: bool,
    #[serde(default)]
    pub published: Option<u64>,
    #[serde(default)]
    pub changelog: Option<String>,
    #[serde(default)]
    pub has_loader: bool,
    #[serde(default)]
    pub loader_version: Option<String>,
    #[serde(rename = "loaderType")]
    pub loader_type: Option<String>,
    #[serde(default)]
    pub mods: Vec<AtlauncherMod>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlauncherMod {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub url: Option<String>,
    pub file: Option<String>,
    #[serde(default)]
    pub md5: Option<String>,
    #[serde(default)]
    pub file_size: Option<u64>,
    #[serde(rename = "type")]
    pub mod_type: Option<String>,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub selected: bool,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert ATLauncher loader type to our LoaderType
fn parse_loader_type(loader: Option<&str>) -> LoaderType {
    match loader.map(|s| s.to_lowercase()).as_deref() {
        Some("forge") => LoaderType::Forge,
        Some("neoforge") => LoaderType::NeoForge,
        Some("fabric") => LoaderType::Fabric,
        Some("quilt") => LoaderType::Quilt,
        Some("liteloader") => LoaderType::LiteLoader,
        _ => LoaderType::Vanilla,
    }
}

/// Get icon URL for ATLauncher pack
/// Uses the CDN endpoint with safeName derived from pack name
fn get_icon_url(safe_name: &str) -> String {
    format!("{}/launcher/images/{}.png", ATLAUNCHER_CDN_BASE, safe_name)
}

// ============================================================================
// Public API Functions
// ============================================================================

/// Convert pack name to safe_name (lowercase, alphanumeric only)
fn name_to_safe_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

/// Search for modpacks on ATLauncher
/// Uses the CDN endpoint (like PrismLauncher) since the API is blocked by Cloudflare
pub async fn search_modpacks(
    client: &Client,
    params: &ModpackSearchParams,
) -> Result<ModpackSearchResult, AppError> {
    let page = params.page.unwrap_or(0);
    let page_size = params.page_size.unwrap_or(20);

    // Fetch all packs from CDN (API is blocked by Cloudflare)
    let url = format!("{}/launcher/json/packsnew.json", ATLAUNCHER_CDN_BASE);
    let packs: Vec<AtlauncherCdnPack> = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    // Filter by query and type (only show public packs)
    let mut filtered: Vec<_> = packs.into_iter().filter(|pack| {
        // Only show public packs
        if pack.pack_type.as_deref() != Some("public") {
            return false;
        }

        // Filter by query
        if let Some(ref query) = params.query {
            if !query.is_empty() {
                let query_lower = query.to_lowercase();
                if !pack.name.to_lowercase().contains(&query_lower) {
                    return false;
                }
            }
        }

        // Filter by MC version
        if let Some(ref mc_version) = params.mc_version {
            if !pack.versions.iter().any(|v| &v.minecraft == mc_version) {
                return false;
            }
        }

        true
    }).collect();

    // Sort
    match params.sort_by.as_ref().unwrap_or(&ModpackSortBy::Name) {
        ModpackSortBy::Name => filtered.sort_by(|a, b| a.name.cmp(&b.name)),
        ModpackSortBy::RecentlyUpdated => {
            // Sort by position (lower = more popular/recent)
            filtered.sort_by(|a, b| a.position.cmp(&b.position));
        }
        _ => {
            // Default to position for downloads/relevance
            filtered.sort_by(|a, b| a.position.cmp(&b.position));
        }
    }

    let total_count = filtered.len() as u64;

    // Apply pagination
    let start = (page * page_size) as usize;
    let modpacks: Vec<Modpack> = filtered
        .into_iter()
        .skip(start)
        .take(page_size as usize)
        .map(|pack| {
            let mc_versions: Vec<String> = pack.versions.iter().map(|v| v.minecraft.clone()).collect();
            let safe_name = name_to_safe_name(&pack.name);

            Modpack {
                id: pack.id.to_string(),
                slug: safe_name.clone(),
                name: pack.name,
                author: "ATLauncher".to_string(),
                description: pack.description.unwrap_or_default(),
                body: None,
                icon_url: Some(get_icon_url(&safe_name)),
                banner_url: None,
                downloads: 0, // Not available from CDN
                platform: ModpackPlatform::ATLauncher,
                categories: vec![],
                mc_versions,
                loaders: vec![], // Would need full pack details
                latest_version: None,
                url: None,
                updated_at: None,
                created_at: None,
            }
        })
        .collect();

    Ok(ModpackSearchResult {
        modpacks,
        total_count,
        page,
        page_size,
    })
}

/// Get a modpack by name
pub async fn get_modpack(client: &Client, pack_name: &str) -> Result<Modpack, AppError> {
    let url = format!("{}/packs/full/{}", ATLAUNCHER_API_BASE, pack_name);

    let response: AtlauncherPackResponse = client
        .get(&url)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| {
            if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                AppError::ModpackNotFound(pack_name.to_string())
            } else {
                AppError::ApiError(e.to_string())
            }
        })?
        .json()
        .await?;

    if response.error {
        return Err(AppError::ModpackNotFound(pack_name.to_string()));
    }

    let pack = response.data;
    let mc_versions: Vec<String> = pack.versions.iter().map(|v| v.minecraft.clone()).collect();

    // Get loaders from versions
    let mut loaders: Vec<LoaderType> = pack
        .versions
        .iter()
        .filter_map(|v| {
            if v.has_loader {
                Some(parse_loader_type(v.loader_type.as_deref()))
            } else {
                None
            }
        })
        .collect();
    loaders.dedup();

    Ok(Modpack {
        id: pack.id.to_string(),
        slug: pack.safe_name.clone(),
        name: pack.name,
        author: "ATLauncher".to_string(),
        description: pack.description.unwrap_or_default(),
        body: None,
        icon_url: Some(get_icon_url(&pack.safe_name)),
        banner_url: None,
        downloads: 0,
        platform: ModpackPlatform::ATLauncher,
        categories: vec![],
        mc_versions,
        loaders,
        latest_version: None, // Would need full version details
        url: pack.website_url,
        updated_at: pack.versions.first().and_then(|v| v.published.map(|p| p as i64)),
        created_at: None,
    })
}

/// Get versions for a modpack
/// Uses the CDN endpoint since the API is blocked by Cloudflare
pub async fn get_modpack_versions(
    client: &Client,
    pack_name: &str,
) -> Result<Vec<ModpackVersion>, AppError> {
    // Fetch all packs from CDN and find the matching one
    let url = format!("{}/launcher/json/packsnew.json", ATLAUNCHER_CDN_BASE);
    let packs: Vec<AtlauncherCdnPack> = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    // Find the pack by name or safeName
    let pack_name_lower = pack_name.to_lowercase();
    let pack = packs
        .into_iter()
        .find(|p| {
            p.name.to_lowercase() == pack_name_lower
                || name_to_safe_name(&p.name) == pack_name_lower
                || p.id.to_string() == pack_name
        })
        .ok_or_else(|| AppError::ModpackNotFound(pack_name.to_string()))?;

    // Convert CDN versions to ModpackVersion
    // Note: CDN doesn't include loader info or file details, so we provide what we can
    let versions: Vec<ModpackVersion> = pack
        .versions
        .into_iter()
        .map(|v| {
            ModpackVersion {
                id: v.version.clone(),
                name: if v.is_recommended {
                    format!("{} (Recommended)", v.version)
                } else {
                    v.version
                },
                mc_version: v.minecraft,
                loader_type: LoaderType::Vanilla, // CDN doesn't include loader info
                loader_version: None,
                changelog: None,
                released_at: None,
                downloads: None,
                files: vec![], // Files require full pack manifest (API is blocked)
            }
        })
        .collect();

    Ok(versions)
}
