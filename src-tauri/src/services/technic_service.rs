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
                "{}/search?build=modpacks&q={}",
                TECHNIC_API_BASE,
                urlencoding::encode(query)
            )
        } else {
            format!("{}/trending?build=modpacks", TECHNIC_API_BASE)
        }
    } else {
        format!("{}/trending?build=modpacks", TECHNIC_API_BASE)
    };

    let response: TechnicSearchResponse = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    // Apply pagination manually since Technic returns all results
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
        total_count: response.total,
        page,
        page_size,
    })
}

/// Get a modpack by slug
pub async fn get_modpack(client: &Client, slug: &str) -> Result<Modpack, AppError> {
    let url = format!("{}/modpack/{}", TECHNIC_API_BASE, slug);

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
pub async fn get_modpack_versions(
    client: &Client,
    slug: &str,
) -> Result<Vec<ModpackVersion>, AppError> {
    // First check if pack uses Solder API
    let pack_url = format!("{}/modpack/{}", TECHNIC_API_BASE, slug);
    let pack: TechnicModpack = client
        .get(&pack_url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    if let Some(solder_url) = pack.solder {
        // Solder API provides multiple versions
        get_solder_versions(client, &solder_url, slug).await
    } else {
        // Non-Solder packs only have the current version
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
            loader_version: pack.forge,
            changelog: None,
            released_at: None,
            downloads: pack.runs,
            files: vec![], // Files come from the pack itself when installing
        }])
    }
}

/// Get versions from Solder API
async fn get_solder_versions(
    client: &Client,
    solder_url: &str,
    slug: &str,
) -> Result<Vec<ModpackVersion>, AppError> {
    let url = format!("{}/modpack/{}", solder_url.trim_end_matches('/'), slug);

    let solder_pack: TechnicSolderModpack = match client
        .get(&url)
        .send()
        .await?
        .error_for_status()
    {
        Ok(response) => response.json().await?,
        Err(_) => return Ok(vec![]), // Solder API might not be accessible
    };

    // Fetch build details for recent versions
    let mut versions = Vec::new();
    for build in solder_pack.builds.iter().take(20) {
        let build_url = format!(
            "{}/modpack/{}/{}",
            solder_url.trim_end_matches('/'),
            slug,
            build
        );

        if let Ok(response) = client.get(&build_url).send().await {
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

                versions.push(ModpackVersion {
                    id: build.clone(),
                    name: build.clone(),
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
    }

    // Mark recommended/latest versions
    if versions.is_empty() && !solder_pack.builds.is_empty() {
        // Fallback: create version entries without details
        for build in solder_pack.builds.iter().take(10) {
            versions.push(ModpackVersion {
                id: build.clone(),
                name: build.clone(),
                mc_version: String::new(),
                loader_type: LoaderType::Vanilla,
                loader_version: None,
                changelog: None,
                released_at: None,
                downloads: None,
                files: vec![],
            });
        }
    }

    Ok(versions)
}
