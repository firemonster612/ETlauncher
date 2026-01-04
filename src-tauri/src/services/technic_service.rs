use crate::error::AppError;
use crate::models::{
    LoaderType, Modpack, ModpackFile, ModpackSearchParams, ModpackSearchResult,
    ModpackVersion,
};
use crate::models::instance::ModpackPlatform;
use reqwest::Client;
use serde::Deserialize;

const TECHNIC_API_BASE: &str = "https://api.technicpack.net";

// ============================================================================
// Technic API Response Types
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct TechnicSearchResponse {
    pub modpacks: Vec<TechnicSearchResult>,
    #[serde(default)]
    pub total: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TechnicSearchResult {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub url: Option<String>,
    #[serde(rename = "iconUrl")]
    pub icon_url: Option<String>,
    #[serde(rename = "logoUrl")]
    pub logo_url: Option<String>,
    #[serde(rename = "platformUrl")]
    pub platform_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TechnicModpack {
    pub id: Option<u64>,
    pub name: String,
    pub display_name: Option<String>,
    pub slug: Option<String>,
    pub url: Option<String>,
    pub user: Option<String>,
    pub description: Option<String>,
    pub minecraft: Option<String>,
    pub forge: Option<String>,
    pub icon: Option<TechnicIcon>,
    pub logo: Option<TechnicLogo>,
    pub background: Option<TechnicBackground>,
    pub runs: Option<u64>,
    pub downloads: Option<u64>,
    pub likes: Option<u64>,
    #[serde(default)]
    pub solder: Option<String>,
    // For non-solder packs, this is the direct download URL
    #[serde(rename = "platformUrl")]
    pub platform_url: Option<String>,
}

/// Get the full modpack info including download URL
pub async fn get_modpack_full(client: &Client, slug: &str) -> Result<TechnicModpack, AppError> {
    let url = format!("{}/modpack/{}?build=multimc", TECHNIC_API_BASE, slug);

    let pack: TechnicModpack = client
        .get(&url)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| {
            if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                AppError::ModpackNotFound(slug.to_string())
            } else {
                AppError::ApiError(e.to_string())
            }
        })?
        .json()
        .await?;

    Ok(pack)
}

/// Get Solder build info for a specific version
pub async fn get_solder_build(
    client: &Client,
    solder_url: &str,
    slug: &str,
    build: &str,
) -> Result<TechnicSolderBuild, AppError> {
    let url = format!(
        "{}/modpack/{}/{}",
        solder_url.trim_end_matches('/'),
        slug,
        build
    );

    let build_info: TechnicSolderBuild = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(build_info)
}

#[derive(Debug, Clone, Deserialize)]
pub struct TechnicIcon {
    pub url: Option<String>,
    pub md5: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TechnicLogo {
    pub url: Option<String>,
    pub md5: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TechnicBackground {
    pub url: Option<String>,
    pub md5: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TechnicSolderModpack {
    pub name: String,
    pub display_name: Option<String>,
    pub url: Option<String>,
    pub icon: Option<String>,
    pub icon_md5: Option<String>,
    pub logo: Option<String>,
    pub logo_md5: Option<String>,
    pub background: Option<String>,
    pub background_md5: Option<String>,
    pub recommended: Option<String>,
    pub latest: Option<String>,
    pub builds: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TechnicSolderBuild {
    pub minecraft: String,
    pub minecraft_md5: Option<String>,
    pub forge: Option<String>,
    pub mods: Vec<TechnicSolderMod>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TechnicSolderMod {
    pub name: String,
    pub version: String,
    pub md5: String,
    pub url: String,
    pub filesize: Option<u64>,
}

// ============================================================================
// Public API Functions
// ============================================================================

/// Search for modpacks on Technic
pub async fn search_modpacks(
    client: &Client,
    params: &ModpackSearchParams,
) -> Result<ModpackSearchResult, AppError> {
    let page = params.page.unwrap_or(0);
    let page_size = params.page_size.unwrap_or(20).min(25);

    // Use search endpoint if query provided, otherwise use trending endpoint
    let url = if let Some(ref query) = params.query {
        if !query.is_empty() {
            format!(
                "{}/search?build=multimc&q={}",
                TECHNIC_API_BASE,
                urlencoding::encode(query)
            )
        } else {
            format!("{}/trending?build=multimc", TECHNIC_API_BASE)
        }
    } else {
        format!("{}/trending?build=multimc", TECHNIC_API_BASE)
    };

    let response: TechnicSearchResponse = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    // Apply pagination manually since Technic returns all results
    // Note: Technic API returns null for total, so use array length
    let total_count = response.modpacks.len() as u64;
    let start = (page * page_size) as usize;
    let modpacks: Vec<Modpack> = response
        .modpacks
        .into_iter()
        .skip(start)
        .take(page_size as usize)
        .map(|m| Modpack {
            id: m.slug.clone(),
            slug: m.slug.clone(),
            name: m.name,
            author: "Unknown".to_string(), // Technic search doesn't include author
            description: String::new(), // Would need to fetch individual pack
            body: None,
            icon_url: m.icon_url.or(m.logo_url),
            banner_url: None,
            downloads: 0, // Would need to fetch individual pack
            platform: ModpackPlatform::Technic,
            categories: vec![],
            mc_versions: vec![],
            loaders: vec![],
            latest_version: None,
            url: m.platform_url.or(m.url),
            updated_at: None,
            created_at: None,
        })
        .collect();

    Ok(ModpackSearchResult {
        modpacks,
        total_count,
        page,
        page_size,
    })
}

/// Get a modpack by slug
pub async fn get_modpack(client: &Client, slug: &str) -> Result<Modpack, AppError> {
    let url = format!("{}/modpack/{}?build=multimc", TECHNIC_API_BASE, slug);

    let pack: TechnicModpack = client
        .get(&url)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| {
            if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                AppError::ModpackNotFound(slug.to_string())
            } else {
                AppError::ApiError(e.to_string())
            }
        })?
        .json()
        .await?;

    let loaders = if pack.forge.is_some() {
        vec![LoaderType::Forge]
    } else {
        vec![]
    };

    let mc_versions = pack.minecraft.clone().map(|v| vec![v]).unwrap_or_default();

    Ok(Modpack {
        id: pack.slug.clone().unwrap_or_else(|| slug.to_string()),
        slug: pack.slug.unwrap_or_else(|| slug.to_string()),
        name: pack.display_name.unwrap_or(pack.name),
        author: pack.user.unwrap_or_else(|| "Unknown".to_string()),
        description: pack.description.unwrap_or_default(),
        body: None,
        icon_url: pack.icon.and_then(|i| i.url),
        banner_url: pack.background.and_then(|b| b.url),
        downloads: pack.downloads.unwrap_or(0),
        platform: ModpackPlatform::Technic,
        categories: vec![],
        mc_versions,
        loaders,
        latest_version: None,
        url: pack.url,
        updated_at: None,
        created_at: None,
    })
}

/// Get versions for a modpack
/// Note: Technic packs without Solder only have one version (the current one)
/// If Solder API fails, falls back to a single "Latest" version
pub async fn get_modpack_versions(
    client: &Client,
    slug: &str,
) -> Result<Vec<ModpackVersion>, AppError> {
    // First get pack info
    let pack_url = format!("{}/modpack/{}?build=multimc", TECHNIC_API_BASE, slug);
    let pack: TechnicModpack = client
        .get(&pack_url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    // Try Solder if available, but fall back to "Latest" on failure
    if let Some(ref solder_url) = pack.solder {
        match get_solder_versions(client, solder_url, slug).await {
            Ok(versions) if !versions.is_empty() => return Ok(versions),
            Ok(_) => {
                eprintln!("[technic] Solder returned empty for {}, using Latest fallback", slug);
            }
            Err(e) => {
                eprintln!("[technic] Solder failed for {}: {}, using Latest fallback", slug, e);
            }
        }
    }

    // Non-Solder fallback: single "Latest" version
    let loader_type = if pack.forge.is_some() {
        LoaderType::Forge
    } else {
        LoaderType::Vanilla
    };

    Ok(vec![ModpackVersion {
        id: "latest".to_string(),
        name: "Latest".to_string(),
        mc_version: pack.minecraft.unwrap_or_default(),
        loader_type,
        loader_version: pack.forge.clone(),
        changelog: None,
        released_at: None,
        downloads: pack.runs,
        files: vec![], // Files come from the pack itself when installing
    }])
}

/// Get versions from Solder API
async fn get_solder_versions(
    client: &Client,
    solder_url: &str,
    slug: &str,
) -> Result<Vec<ModpackVersion>, AppError> {
    let url = format!("{}/modpack/{}", solder_url.trim_end_matches('/'), slug);

    // Use a short timeout for the Solder API
    let solder_pack: TechnicSolderModpack = match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        client.get(&url).send()
    ).await {
        Ok(Ok(response)) => match response.error_for_status() {
            Ok(r) => match r.json().await {
                Ok(pack) => pack,
                Err(_) => return Ok(vec![]),
            },
            Err(_) => return Ok(vec![]),
        },
        Ok(Err(_)) | Err(_) => return Ok(vec![]), // Timeout or error - return empty
    };

    // Fetch build details in parallel with timeout
    // Reverse to get newest versions first
    let builds_to_fetch: Vec<_> = solder_pack.builds.iter().rev().take(10).collect();

    let futures: Vec<_> = builds_to_fetch
        .iter()
        .map(|build| {
            let build_url = format!(
                "{}/modpack/{}/{}",
                solder_url.trim_end_matches('/'),
                slug,
                build
            );
            let client = client.clone();
            let build_name = (*build).clone();

            async move {
                let result = tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    client.get(&build_url).send()
                ).await;

                match result {
                    Ok(Ok(response)) => {
                        if let Ok(build_info) = response.json::<TechnicSolderBuild>().await {
                            let loader_type = if build_info.forge.is_some() {
                                LoaderType::Forge
                            } else {
                                LoaderType::Vanilla
                            };

                            let files: Vec<ModpackFile> = build_info
                                .mods
                                .into_iter()
                                .map(|m| ModpackFile {
                                    url: m.url,
                                    hash: Some(m.md5),
                                    hash_algorithm: Some("md5".to_string()),
                                    size: m.filesize.unwrap_or(0),
                                    path: format!("mods/{}-{}.zip", m.name, m.version),
                                    required: true,
                                })
                                .collect();

                            return Some(ModpackVersion {
                                id: build_name.clone(),
                                name: build_name,
                                mc_version: build_info.minecraft,
                                loader_type,
                                loader_version: build_info.forge,
                                changelog: None,
                                released_at: None,
                                downloads: None,
                                files,
                            });
                        }
                    }
                    _ => {}
                }

                // Return a minimal version entry on failure
                Some(ModpackVersion {
                    id: build_name.clone(),
                    name: build_name,
                    mc_version: String::new(),
                    loader_type: LoaderType::Vanilla,
                    loader_version: None,
                    changelog: None,
                    released_at: None,
                    downloads: None,
                    files: vec![],
                })
            }
        })
        .collect();

    let results = futures::future::join_all(futures).await;
    let versions: Vec<ModpackVersion> = results.into_iter().flatten().collect();

    Ok(versions)
}
