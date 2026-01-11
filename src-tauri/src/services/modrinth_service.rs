use crate::error::AppError;
use crate::models::instance::ModpackPlatform;
use crate::models::{
    Content, ContentDependency, ContentFile, ContentGalleryImage, ContentPlatform,
    ContentSearchParams, ContentSearchResult, ContentType, ContentVersion, DependencyType,
    LoaderType, Modpack, ModpackFile, ModpackMod, ModpackSearchParams, ModpackSearchResult,
    ModpackSortBy, ModpackVersion,
};
use reqwest::Client;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::io::Read;

const MODRINTH_API_BASE: &str = "https://api.modrinth.com/v2";
const USER_AGENT: &str = "ETLauncher/1.0 (github.com/etlauncher)";

/// Modrinth project type
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModrinthProjectType {
    Mod,
    Modpack,
    Resourcepack,
    Shader,
}

/// Modrinth search hit (project)
#[derive(Debug, Clone, Deserialize)]
pub struct ModrinthSearchHit {
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub icon_url: Option<String>,
    pub downloads: u64,
    pub project_type: ModrinthProjectType,
    pub categories: Vec<String>,
    pub versions: Vec<String>,
    #[serde(default)]
    pub loaders: Vec<String>,
    pub date_modified: String,
    pub date_created: String,
}

/// Modrinth search response
#[derive(Debug, Clone, Deserialize)]
pub struct ModrinthSearchResponse {
    pub hits: Vec<ModrinthSearchHit>,
    pub offset: u32,
    pub limit: u32,
    pub total_hits: u64,
}

/// Modrinth full project details
#[derive(Debug, Clone, Deserialize)]
pub struct ModrinthProject {
    pub id: String,
    pub slug: String,
    pub project_type: ModrinthProjectType,
    pub title: String,
    pub description: String,
    pub body: String,
    pub icon_url: Option<String>,
    pub downloads: u64,
    pub categories: Vec<String>,
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub loaders: Vec<String>,
    pub team: String,
    pub updated: String,
    pub published: String,
    #[serde(default)]
    pub gallery: Option<Vec<ModrinthGalleryImage>>,
}

/// Modrinth gallery image
#[derive(Debug, Clone, Deserialize)]
pub struct ModrinthGalleryImage {
    pub url: String,
    pub raw_url: Option<String>,
    pub featured: Option<bool>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub ordering: Option<i64>,
}

/// Modrinth version file
#[derive(Debug, Clone, Deserialize)]
pub struct ModrinthVersionFile {
    pub url: String,
    pub filename: String,
    pub size: u64,
    pub primary: bool,
    pub hashes: ModrinthFileHashes,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModrinthFileHashes {
    pub sha1: Option<String>,
    pub sha512: Option<String>,
}

/// Modrinth version dependency
#[derive(Debug, Clone, Deserialize)]
pub struct ModrinthDependency {
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: String,
}

/// Modrinth version
#[derive(Debug, Clone, Deserialize)]
pub struct ModrinthVersion {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub version_number: String,
    pub changelog: Option<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub files: Vec<ModrinthVersionFile>,
    pub downloads: u64,
    pub date_published: String,
    #[serde(default)]
    pub dependencies: Vec<ModrinthDependency>,
}

/// Modrinth team member
#[derive(Debug, Clone, Deserialize)]
pub struct ModrinthTeamMember {
    pub user: ModrinthUser,
    pub role: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModrinthUser {
    pub username: String,
    pub name: Option<String>,
}

/// Convert Modrinth loaders to our LoaderType
fn parse_loaders(loaders: &[String]) -> Vec<LoaderType> {
    loaders
        .iter()
        .filter_map(|l| match l.to_lowercase().as_str() {
            "forge" => Some(LoaderType::Forge),
            "neoforge" => Some(LoaderType::NeoForge),
            "fabric" => Some(LoaderType::Fabric),
            "quilt" => Some(LoaderType::Quilt),
            "liteloader" => Some(LoaderType::LiteLoader),
            _ => None,
        })
        .collect()
}

/// Extract loaders from categories (Modrinth includes loaders in categories for search results)
fn extract_loaders_from_categories(categories: &[String]) -> Vec<LoaderType> {
    categories
        .iter()
        .filter_map(|c| match c.to_lowercase().as_str() {
            "forge" => Some(LoaderType::Forge),
            "neoforge" => Some(LoaderType::NeoForge),
            "fabric" => Some(LoaderType::Fabric),
            "quilt" => Some(LoaderType::Quilt),
            "liteloader" => Some(LoaderType::LiteLoader),
            _ => None,
        })
        .collect()
}

/// Filter out loader names from categories to get actual categories
fn filter_categories(categories: Vec<String>) -> Vec<String> {
    let loader_names = ["forge", "neoforge", "fabric", "quilt", "liteloader"];
    categories
        .into_iter()
        .filter(|c| !loader_names.contains(&c.to_lowercase().as_str()))
        .collect()
}

/// Convert LoaderType to Modrinth loader string
fn loader_to_string(loader: &LoaderType) -> Option<&'static str> {
    match loader {
        LoaderType::Vanilla | LoaderType::Unknown => None,
        LoaderType::Forge => Some("forge"),
        LoaderType::NeoForge => Some("neoforge"),
        LoaderType::Fabric => Some("fabric"),
        LoaderType::Quilt => Some("quilt"),
        LoaderType::LiteLoader => Some("liteloader"),
    }
}

/// Convert ModpackSortBy to Modrinth index
fn sort_to_modrinth(sort: &ModpackSortBy) -> &'static str {
    match sort {
        ModpackSortBy::Downloads => "downloads",
        ModpackSortBy::RecentlyUpdated => "updated",
        ModpackSortBy::Name => "newest", // Modrinth doesn't have alphabetical, use newest
        ModpackSortBy::Relevance => "relevance",
    }
}

/// Parse ISO 8601 date to Unix timestamp
fn parse_date(date: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(date)
        .ok()
        .map(|dt| dt.timestamp())
}

/// Search for modpacks on Modrinth
pub async fn search_modpacks(
    client: &Client,
    params: &ModpackSearchParams,
) -> Result<ModpackSearchResult, AppError> {
    let page = params.page.unwrap_or(0);
    let page_size = params.page_size.unwrap_or(20).min(100);
    let offset = page * page_size;

    // Build facets for filtering
    let mut facets: Vec<Vec<String>> = vec![vec!["project_type:modpack".to_string()]];

    if let Some(ref mc_version) = params.mc_version {
        facets.push(vec![format!("versions:{}", mc_version)]);
    }

    if let Some(ref loader) = params.loader {
        if let Some(loader_str) = loader_to_string(loader) {
            facets.push(vec![format!("categories:{}", loader_str)]);
        }
    }

    if let Some(ref category) = params.category {
        facets.push(vec![format!("categories:{}", category)]);
    }

    let facets_json = serde_json::to_string(&facets)?;
    let sort_index = params
        .sort_by
        .as_ref()
        .map(sort_to_modrinth)
        .unwrap_or("downloads");

    let mut url = format!(
        "{}/search?facets={}&index={}&offset={}&limit={}",
        MODRINTH_API_BASE,
        urlencoding::encode(&facets_json),
        sort_index,
        offset,
        page_size
    );

    if let Some(ref query) = params.query {
        if !query.is_empty() {
            url.push_str(&format!("&query={}", urlencoding::encode(query)));
        }
    }

    let response: ModrinthSearchResponse = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let modpacks = response
        .hits
        .into_iter()
        .filter_map(|hit| {
            if !matches!(hit.project_type, ModrinthProjectType::Modpack) {
                return None;
            }
            // Modrinth search doesn't return loaders separately - they're in categories
            let loaders = if hit.loaders.is_empty() {
                extract_loaders_from_categories(&hit.categories)
            } else {
                parse_loaders(&hit.loaders)
            };
            let categories = filter_categories(hit.categories);
            Some(Modpack {
                id: hit.project_id,
                slug: hit.slug.clone(),
                name: hit.title,
                author: hit.author,
                description: hit.description,
                body: None,
                icon_url: hit.icon_url,
                banner_url: None,
                downloads: hit.downloads,
                platform: ModpackPlatform::Modrinth,
                categories,
                gallery: Vec::new(),
                mc_versions: hit.versions,
                loaders,
                latest_version: None,
                url: Some(format!("https://modrinth.com/modpack/{}", hit.slug)),
                updated_at: parse_date(&hit.date_modified),
                created_at: parse_date(&hit.date_created),
            })
        })
        .collect();

    Ok(ModpackSearchResult {
        modpacks,
        total_count: response.total_hits,
        page,
        page_size,
    })
}

/// Get a modpack by ID or slug
pub async fn get_modpack(client: &Client, id_or_slug: &str) -> Result<Modpack, AppError> {
    let url = format!("{}/project/{}", MODRINTH_API_BASE, id_or_slug);

    let project: ModrinthProject = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| {
            if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                AppError::ModpackNotFound(id_or_slug.to_string())
            } else {
                AppError::ApiError(e.to_string())
            }
        })?
        .json()
        .await?;

    if !matches!(project.project_type, ModrinthProjectType::Modpack) {
        return Err(AppError::ModpackNotFound(format!(
            "{} is not a modpack",
            id_or_slug
        )));
    }

    // Get team info for author
    let author = get_project_author(client, &project.team)
        .await
        .unwrap_or_else(|_| "Unknown".to_string());

    // Project endpoint returns loaders, but fallback to categories just in case
    let loaders = if project.loaders.is_empty() {
        extract_loaders_from_categories(&project.categories)
    } else {
        parse_loaders(&project.loaders)
    };
    let categories = filter_categories(project.categories);

    Ok(Modpack {
        id: project.id.clone(),
        slug: project.slug,
        name: project.title,
        author,
        description: project.description,
        body: Some(project.body),
        icon_url: project.icon_url,
        banner_url: None,
        downloads: project.downloads,
        platform: ModpackPlatform::Modrinth,
        categories,
        gallery: project
            .gallery
            .unwrap_or_default()
            .into_iter()
            .map(|image| ContentGalleryImage {
                url: image.url,
                raw_url: image.raw_url,
                title: image.title,
                description: image.description,
                featured: image.featured.unwrap_or(false),
            })
            .collect(),
        mc_versions: project.game_versions,
        loaders,
        latest_version: None,
        url: Some(format!("https://modrinth.com/modpack/{}", project.id)),
        updated_at: parse_date(&project.updated),
        created_at: parse_date(&project.published),
    })
}

/// Get author name from team ID
async fn get_project_author(client: &Client, team_id: &str) -> Result<String, AppError> {
    let url = format!("{}/team/{}/members", MODRINTH_API_BASE, team_id);

    let members: Vec<ModrinthTeamMember> = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    // Find owner or first member
    let author = members
        .iter()
        .find(|m| m.role.to_lowercase() == "owner")
        .or_else(|| members.first())
        .map(|m| {
            m.user
                .name
                .clone()
                .unwrap_or_else(|| m.user.username.clone())
        })
        .unwrap_or_else(|| "Unknown".to_string());

    Ok(author)
}

/// Get versions for a modpack
pub async fn get_modpack_versions(
    client: &Client,
    id_or_slug: &str,
) -> Result<Vec<ModpackVersion>, AppError> {
    let url = format!("{}/project/{}/version", MODRINTH_API_BASE, id_or_slug);

    let versions: Vec<ModrinthVersion> = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(versions
        .into_iter()
        .map(|v| {
            let loaders = parse_loaders(&v.loaders);
            let loader_type = loaders.first().cloned().unwrap_or(LoaderType::Vanilla);

            ModpackVersion {
                id: v.id,
                name: v.name,
                mc_version: v.game_versions.first().cloned().unwrap_or_default(),
                loader_type,
                loader_version: None, // Modrinth doesn't specify loader version directly
                changelog: v.changelog,
                released_at: parse_date(&v.date_published),
                downloads: Some(v.downloads),
                files: v
                    .files
                    .into_iter()
                    .map(|f| ModpackFile {
                        url: f.url,
                        hash: f.hashes.sha512.clone().or(f.hashes.sha1.clone()),
                        hash_algorithm: if f.hashes.sha512.is_some() {
                            Some("sha512".to_string())
                        } else {
                            Some("sha1".to_string())
                        },
                        size: f.size,
                        path: f.filename,
                        required: true,
                    })
                    .collect(),
            }
        })
        .collect())
}

/// Get a specific modpack version by ID
pub async fn get_modpack_version(
    client: &Client,
    version_id: &str,
) -> Result<ModpackVersion, AppError> {
    let url = format!("{}/version/{}", MODRINTH_API_BASE, version_id);

    let v: ModrinthVersion = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let loaders = parse_loaders(&v.loaders);
    let loader_type = loaders.first().cloned().unwrap_or(LoaderType::Vanilla);

    Ok(ModpackVersion {
        id: v.id,
        name: v.name,
        mc_version: v.game_versions.first().cloned().unwrap_or_default(),
        loader_type,
        loader_version: None,
        changelog: v.changelog,
        released_at: parse_date(&v.date_published),
        downloads: Some(v.downloads),
        files: v
            .files
            .into_iter()
            .map(|f| ModpackFile {
                url: f.url,
                hash: f.hashes.sha512.clone().or(f.hashes.sha1.clone()),
                hash_algorithm: if f.hashes.sha512.is_some() {
                    Some("sha512".to_string())
                } else {
                    Some("sha1".to_string())
                },
                size: f.size,
                path: f.filename,
                required: true,
            })
            .collect(),
    })
}

// ============================================================================
// Content (Mods, Shaders, Resource Packs)
// ============================================================================

/// Map Modrinth project type to our ContentType
fn project_type_to_content_type(pt: &ModrinthProjectType) -> Option<ContentType> {
    match pt {
        ModrinthProjectType::Mod => Some(ContentType::Mod),
        ModrinthProjectType::Shader => Some(ContentType::Shader),
        ModrinthProjectType::Resourcepack => Some(ContentType::ResourcePack),
        ModrinthProjectType::Modpack => None,
    }
}

/// Map ContentType to Modrinth project type string
fn content_type_to_modrinth(ct: &ContentType) -> &'static str {
    match ct {
        ContentType::Mod => "mod",
        ContentType::Shader => "shader",
        ContentType::ResourcePack => "resourcepack",
    }
}

/// Search for content on Modrinth
pub async fn search_content(
    client: &Client,
    params: &ContentSearchParams,
) -> Result<ContentSearchResult, AppError> {
    let page = params.page.unwrap_or(0);
    let page_size = params.page_size.unwrap_or(20).min(100);
    let offset = page * page_size;

    // Build facets for filtering
    let mut facets: Vec<Vec<String>> = vec![];

    // Filter by content type
    if let Some(ref content_type) = params.content_type {
        facets.push(vec![format!(
            "project_type:{}",
            content_type_to_modrinth(content_type)
        )]);
    } else {
        // Default to mods, shaders, resourcepacks (exclude modpacks)
        facets.push(vec![
            "project_type:mod".to_string(),
            "project_type:shader".to_string(),
            "project_type:resourcepack".to_string(),
        ]);
    }

    if let Some(ref mc_version) = params.mc_version {
        facets.push(vec![format!("versions:{}", mc_version)]);
    }

    // Only apply loader filter to mods (shaders/resourcepacks don't have loaders)
    if let Some(ref loader) = params.loader {
        let is_mod = params
            .content_type
            .as_ref()
            .is_none_or(|ct| *ct == ContentType::Mod);
        if is_mod {
            if let Some(loader_str) = loader_to_string(loader) {
                facets.push(vec![format!("categories:{}", loader_str)]);
            }
        }
    }

    if let Some(ref category) = params.category {
        facets.push(vec![format!("categories:{}", category)]);
    }

    let facets_json = serde_json::to_string(&facets)?;
    let sort_index = params
        .sort_by
        .as_ref()
        .map(|s| match s {
            crate::models::ContentSortBy::Downloads => "downloads",
            crate::models::ContentSortBy::RecentlyUpdated => "updated",
            crate::models::ContentSortBy::Name => "newest",
            crate::models::ContentSortBy::Relevance => "relevance",
        })
        .unwrap_or("downloads");

    let mut url = format!(
        "{}/search?facets={}&index={}&offset={}&limit={}",
        MODRINTH_API_BASE,
        urlencoding::encode(&facets_json),
        sort_index,
        offset,
        page_size
    );

    if let Some(ref query) = params.query {
        if !query.is_empty() {
            url.push_str(&format!("&query={}", urlencoding::encode(query)));
        }
    }

    let response: ModrinthSearchResponse = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let items = response
        .hits
        .into_iter()
        .filter_map(|hit| {
            let content_type = project_type_to_content_type(&hit.project_type)?;
            let url_path = match content_type {
                ContentType::Mod => "mod",
                ContentType::Shader => "shader",
                ContentType::ResourcePack => "resourcepack",
            };
            // Modrinth search doesn't return loaders separately - they're in categories
            let loaders = if hit.loaders.is_empty() {
                extract_loaders_from_categories(&hit.categories)
            } else {
                parse_loaders(&hit.loaders)
            };
            let categories = filter_categories(hit.categories);
            Some(Content {
                id: hit.project_id,
                slug: hit.slug.clone(),
                name: hit.title,
                author: hit.author,
                description: hit.description,
                body: None,
                icon_url: hit.icon_url,
                downloads: hit.downloads,
                platform: ContentPlatform::Modrinth,
                content_type,
                categories,
                gallery: Vec::new(),
                mc_versions: hit.versions,
                loaders,
                latest_version: None,
                url: Some(format!("https://modrinth.com/{}/{}", url_path, hit.slug)),
                updated_at: parse_date(&hit.date_modified),
                created_at: parse_date(&hit.date_created),
            })
        })
        .collect();

    Ok(ContentSearchResult {
        items,
        total_count: response.total_hits,
        page,
        page_size,
    })
}

/// Get content by ID or slug
pub async fn get_content(client: &Client, id_or_slug: &str) -> Result<Content, AppError> {
    let url = format!("{}/project/{}", MODRINTH_API_BASE, id_or_slug);

    let project: ModrinthProject = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| {
            if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                AppError::ContentNotFound(id_or_slug.to_string())
            } else {
                AppError::ApiError(e.to_string())
            }
        })?
        .json()
        .await?;

    let content_type = project_type_to_content_type(&project.project_type).ok_or_else(|| {
        AppError::ContentNotFound(format!("{} is a modpack, not content", id_or_slug))
    })?;

    let author = get_project_author(client, &project.team)
        .await
        .unwrap_or_else(|_| "Unknown".to_string());

    let url_path = match content_type {
        ContentType::Mod => "mod",
        ContentType::Shader => "shader",
        ContentType::ResourcePack => "resourcepack",
    };

    // Project endpoint returns loaders, but fallback to categories just in case
    let loaders = if project.loaders.is_empty() {
        extract_loaders_from_categories(&project.categories)
    } else {
        parse_loaders(&project.loaders)
    };
    let categories = filter_categories(project.categories);
    let gallery = project
        .gallery
        .unwrap_or_default()
        .into_iter()
        .map(|image| ContentGalleryImage {
            url: image.url,
            raw_url: image.raw_url,
            title: image.title,
            description: image.description,
            featured: image.featured.unwrap_or(false),
        })
        .collect();

    Ok(Content {
        id: project.id.clone(),
        slug: project.slug.clone(),
        name: project.title,
        author,
        description: project.description,
        body: Some(project.body),
        icon_url: project.icon_url,
        downloads: project.downloads,
        platform: ContentPlatform::Modrinth,
        content_type,
        categories,
        gallery,
        mc_versions: project.game_versions,
        loaders,
        latest_version: None,
        url: Some(format!(
            "https://modrinth.com/{}/{}",
            url_path, project.slug
        )),
        updated_at: parse_date(&project.updated),
        created_at: parse_date(&project.published),
    })
}

/// Get versions for content
pub async fn get_content_versions(
    client: &Client,
    id_or_slug: &str,
    mc_version: Option<&str>,
    loader: Option<&LoaderType>,
) -> Result<Vec<ContentVersion>, AppError> {
    let mut url = format!("{}/project/{}/version", MODRINTH_API_BASE, id_or_slug);

    // Add filters
    let mut params = vec![];
    if let Some(mc) = mc_version {
        params.push(format!("game_versions=[\"{}\"]", mc));
    }
    if let Some(l) = loader {
        if let Some(loader_str) = loader_to_string(l) {
            params.push(format!("loaders=[\"{}\"]", loader_str));
        }
    }
    if !params.is_empty() {
        url.push_str(&format!("?{}", params.join("&")));
    }

    let versions: Vec<ModrinthVersion> = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(versions
        .into_iter()
        .map(|v| ContentVersion {
            id: v.id,
            project_id: v.project_id,
            name: v.name,
            version_number: v.version_number,
            mc_versions: v.game_versions,
            loaders: parse_loaders(&v.loaders),
            released_at: parse_date(&v.date_published),
            downloads: Some(v.downloads),
            files: v
                .files
                .into_iter()
                .map(|f| ContentFile {
                    url: f.url,
                    hash: f.hashes.sha512.clone().or(f.hashes.sha1.clone()),
                    hash_algorithm: if f.hashes.sha512.is_some() {
                        Some("sha512".to_string())
                    } else {
                        Some("sha1".to_string())
                    },
                    size: f.size,
                    filename: f.filename,
                    primary: f.primary,
                })
                .collect(),
            dependencies: v
                .dependencies
                .into_iter()
                .filter_map(|d| {
                    // Use project_id if available, otherwise use version_id as fallback
                    let id = d
                        .project_id
                        .clone()
                        .or_else(|| d.version_id.clone().map(|v| format!("version:{}", v)))?;
                    let dep_type = match d.dependency_type.as_str() {
                        "required" => DependencyType::Required,
                        "optional" => DependencyType::Optional,
                        "incompatible" => DependencyType::Incompatible,
                        "embedded" => DependencyType::Embedded,
                        _ => return None,
                    };
                    Some(ContentDependency {
                        id,
                        name: None,
                        dependency_type: dep_type,
                        version_req: d.version_id,
                    })
                })
                .collect(),
            changelog: v.changelog,
        })
        .collect())
}

/// Get a specific version by ID
pub async fn get_version(client: &Client, version_id: &str) -> Result<ContentVersion, AppError> {
    let url = format!("{}/version/{}", MODRINTH_API_BASE, version_id);

    let v: ModrinthVersion = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(ContentVersion {
        id: v.id,
        project_id: v.project_id,
        name: v.name,
        version_number: v.version_number,
        mc_versions: v.game_versions,
        loaders: parse_loaders(&v.loaders),
        released_at: parse_date(&v.date_published),
        downloads: Some(v.downloads),
        files: v
            .files
            .into_iter()
            .map(|f| ContentFile {
                url: f.url,
                hash: f.hashes.sha512.clone().or(f.hashes.sha1.clone()),
                hash_algorithm: if f.hashes.sha512.is_some() {
                    Some("sha512".to_string())
                } else {
                    Some("sha1".to_string())
                },
                size: f.size,
                filename: f.filename,
                primary: f.primary,
            })
            .collect(),
        dependencies: v
            .dependencies
            .into_iter()
            .filter_map(|d| {
                // Use project_id if available, otherwise use version_id as fallback
                let id = d
                    .project_id
                    .clone()
                    .or_else(|| d.version_id.clone().map(|v| format!("version:{}", v)))?;
                let dep_type = match d.dependency_type.as_str() {
                    "required" => DependencyType::Required,
                    "optional" => DependencyType::Optional,
                    "incompatible" => DependencyType::Incompatible,
                    "embedded" => DependencyType::Embedded,
                    _ => return None,
                };
                Some(ContentDependency {
                    id,
                    name: None,
                    dependency_type: dep_type,
                    version_req: d.version_id,
                })
            })
            .collect(),
        changelog: v.changelog,
    })
}

/// Look up versions by file hashes (batch)
/// Uses POST /v2/version_files with body { "hashes": [...], "algorithm": "sha512" }
/// Returns a map from hash to version info
pub async fn get_versions_from_hashes(
    client: &Client,
    hashes: &[String],
) -> Result<std::collections::HashMap<String, ModrinthVersion>, AppError> {
    if hashes.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let url = format!("{}/version_files", MODRINTH_API_BASE);

    let body = serde_json::json!({
        "hashes": hashes,
        "algorithm": "sha512"
    });

    let response = client
        .post(&url)
        .header("User-Agent", USER_AGENT)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(AppError::ApiError(format!(
            "Modrinth API error: {}",
            response.status()
        )));
    }

    let result: std::collections::HashMap<String, ModrinthVersion> = response.json().await?;

    Ok(result)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModrinthIndex {
    #[serde(default)]
    files: Vec<ModrinthIndexFile>,
}

#[derive(Debug, Deserialize)]
struct ModrinthIndexFile {
    path: String,
    #[serde(default)]
    hashes: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModrinthProjectLite {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub icon_url: Option<String>,
}

/// Batch fetch projects by ID (best-effort)
pub async fn get_projects_by_ids(
    client: &Client,
    ids: &[String],
) -> Result<Vec<ModrinthProjectLite>, AppError> {
    if ids.is_empty() {
        return Ok(vec![]);
    }

    let url = format!("{}/projects", MODRINTH_API_BASE);
    let ids_json = serde_json::to_string(ids)?;

    let response = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .query(&[("ids", ids_json)])
        .send()
        .await?
        .error_for_status()?;

    Ok(response.json::<Vec<ModrinthProjectLite>>().await?)
}

async fn download_bytes(client: &Client, url: &str) -> Result<Vec<u8>, AppError> {
    let resp = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .error_for_status()?;
    Ok(resp.bytes().await?.to_vec())
}

/// Get a mod list for a Modrinth modpack version (best-effort)
pub async fn get_modpack_mods(
    client: &Client,
    version_id: &str,
) -> Result<Vec<ModpackMod>, AppError> {
    let version = get_modpack_version(client, version_id).await?;

    let mrpack_file = version
        .files
        .iter()
        .find(|f| f.path.ends_with(".mrpack"))
        .or_else(|| version.files.first())
        .ok_or_else(|| AppError::ContentNotFound("No modpack file found".to_string()))?;

    let mrpack_bytes = download_bytes(client, &mrpack_file.url).await?;
    let cursor = std::io::Cursor::new(&mrpack_bytes);
    let mut archive = zip::ZipArchive::new(cursor)?;

    let index: ModrinthIndex = {
        let mut index_file = archive.by_name("modrinth.index.json")?;
        let mut contents = String::new();
        index_file.read_to_string(&mut contents)?;
        serde_json::from_str(&contents)?
    };

    // Extract sha512 hashes for mod files (mods/*)
    let mut sha512_hashes: Vec<String> = Vec::new();
    for f in index.files {
        if !f.path.starts_with("mods/") {
            continue;
        }
        if let Some(h) = f.hashes.get("sha512") {
            sha512_hashes.push(h.clone());
        }
    }
    sha512_hashes.sort();
    sha512_hashes.dedup();

    let versions_by_hash = get_versions_from_hashes(client, &sha512_hashes)
        .await
        .unwrap_or_default();
    let mut project_ids: Vec<String> = versions_by_hash
        .values()
        .map(|v| v.project_id.clone())
        .collect();
    project_ids.sort();
    project_ids.dedup();

    let projects = get_projects_by_ids(client, &project_ids)
        .await
        .unwrap_or_default();
    let project_map: HashMap<String, ModrinthProjectLite> =
        projects.into_iter().map(|p| (p.id.clone(), p)).collect();

    // Build mod list, dedupe by project id when possible.
    let mut seen: HashSet<String> = HashSet::new();
    let mut mods: Vec<ModpackMod> = Vec::new();

    for v in versions_by_hash.values() {
        if !seen.insert(v.project_id.clone()) {
            continue;
        }
        if let Some(p) = project_map.get(&v.project_id) {
            mods.push(ModpackMod {
                id: p.id.clone(),
                name: p.title.clone(),
                icon_url: p.icon_url.clone(),
                author: None,
                url: Some(format!("https://modrinth.com/mod/{}", p.slug)),
            });
        } else {
            mods.push(ModpackMod {
                id: v.project_id.clone(),
                name: v.project_id.clone(),
                icon_url: None,
                author: None,
                url: Some(format!("https://modrinth.com/mod/{}", v.project_id)),
            });
        }
    }

    mods.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(mods)
}
