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
use std::io::Read;

const CURSEFORGE_API_BASE: &str = "https://api.curseforge.com/v1";
const MINECRAFT_GAME_ID: u32 = 432;

/// CurseForge class IDs for different content types
#[derive(Debug, Clone, Copy)]
pub enum CurseForgeClassId {
    Mods = 6,
    ResourcePacks = 12,
    Modpacks = 4471,
    Shaders = 6552,
}

impl CurseForgeClassId {
    fn from_content_type(ct: &ContentType) -> Self {
        match ct {
            ContentType::Mod => CurseForgeClassId::Mods,
            ContentType::Shader => CurseForgeClassId::Shaders,
            ContentType::ResourcePack => CurseForgeClassId::ResourcePacks,
        }
    }
}

/// Map category name to CurseForge modpack category ID
fn modpack_category_to_id(category: &str) -> Option<u32> {
    match category.to_lowercase().as_str() {
        "adventure and rpg" | "adventure-and-rpg" => Some(4475),
        "combat / pvp" | "combat-pvp" | "combat/pvp" => Some(4483),
        "expert" => Some(9243),
        "exploration" => Some(4476),
        "extra large" | "extra-large" => Some(4482),
        "ftb official pack" | "ftb-official-pack" => Some(4487),
        "hardcore" => Some(4479),
        "horror" => Some(7418),
        "magic" => Some(4473),
        "map based" | "map-based" => Some(4480),
        "mini game" | "mini-game" => Some(4477),
        "multiplayer" => Some(4484),
        "quests" => Some(4478),
        "sci-fi" | "scifi" => Some(4474),
        "skyblock" => Some(4736),
        "small / light" | "small-light" | "small/light" => Some(4481),
        "tech" | "technology" => Some(4472),
        "vanilla+" | "vanilla" => Some(5128),
        _ => None,
    }
}

/// CurseForge mod loader type
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub enum CurseForgeModLoaderType {
    Any = 0,
    Forge = 1,
    Cauldron = 2,
    LiteLoader = 3,
    Fabric = 4,
    Quilt = 5,
    NeoForge = 6,
}

#[allow(dead_code)]
impl CurseForgeModLoaderType {
    fn from_loader_type(lt: &LoaderType) -> Self {
        match lt {
            LoaderType::Vanilla | LoaderType::Unknown => CurseForgeModLoaderType::Any,
            LoaderType::Forge => CurseForgeModLoaderType::Forge,
            LoaderType::NeoForge => CurseForgeModLoaderType::NeoForge,
            LoaderType::Fabric => CurseForgeModLoaderType::Fabric,
            LoaderType::Quilt => CurseForgeModLoaderType::Quilt,
            LoaderType::LiteLoader => CurseForgeModLoaderType::LiteLoader,
        }
    }

    fn to_loader_type(self) -> Option<LoaderType> {
        match self {
            CurseForgeModLoaderType::Forge => Some(LoaderType::Forge),
            CurseForgeModLoaderType::NeoForge => Some(LoaderType::NeoForge),
            CurseForgeModLoaderType::Fabric => Some(LoaderType::Fabric),
            CurseForgeModLoaderType::Quilt => Some(LoaderType::Quilt),
            CurseForgeModLoaderType::LiteLoader => Some(LoaderType::LiteLoader),
            _ => None,
        }
    }
}

/// CurseForge sort field
#[derive(Debug, Clone, Copy)]
pub enum CurseForgeSortField {
    Featured = 1,
    Popularity = 2,
    LastUpdated = 3,
    Name = 4,
    Author = 5,
    TotalDownloads = 6,
    Category = 7,
    GameVersion = 8,
}

impl CurseForgeSortField {
    fn from_sort_by(sort: &ModpackSortBy) -> Self {
        match sort {
            ModpackSortBy::Downloads => CurseForgeSortField::TotalDownloads,
            ModpackSortBy::RecentlyUpdated => CurseForgeSortField::LastUpdated,
            ModpackSortBy::Name => CurseForgeSortField::Name,
            ModpackSortBy::Relevance => CurseForgeSortField::Featured,
        }
    }
}

/// CurseForge release type
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub enum CurseForgeReleaseType {
    Release = 1,
    Beta = 2,
    Alpha = 3,
}

/// CurseForge file status
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub enum CurseForgeFileStatus {
    Processing = 1,
    ChangesRequired = 2,
    UnderReview = 3,
    Approved = 4,
    Rejected = 5,
    MalwareDetected = 6,
    Deleted = 7,
    Archived = 8,
    Testing = 9,
    Released = 10,
    ReadyForReview = 11,
    Deprecated = 12,
    Baking = 13,
    AwaitingPublishing = 14,
    FailedPublishing = 15,
}

/// CurseForge dependency type
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub enum CurseForgeRelationType {
    EmbeddedLibrary = 1,
    OptionalDependency = 2,
    RequiredDependency = 3,
    Tool = 4,
    Incompatible = 5,
    Include = 6,
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct CurseForgeResponse<T> {
    pub data: T,
    pub pagination: Option<CurseForgePagination>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgePagination {
    pub index: u32,
    pub page_size: u32,
    pub result_count: u32,
    pub total_count: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeMod {
    pub id: u64,
    pub game_id: u32,
    pub name: String,
    pub slug: String,
    pub summary: String,
    pub download_count: u64,
    #[serde(default)]
    pub class_id: Option<u32>,
    pub authors: Vec<CurseForgeAuthor>,
    pub logo: Option<CurseForgeLogo>,
    pub categories: Vec<CurseForgeCategory>,
    pub date_created: String,
    pub date_modified: String,
    pub date_released: String,
    pub latest_files: Vec<CurseForgeFile>,
    #[serde(default)]
    pub latest_files_indexes: Vec<CurseForgeFileIndex>,
    #[serde(default)]
    pub screenshots: Vec<CurseForgeScreenshot>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeAuthor {
    pub id: u64,
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeLogo {
    pub id: u64,
    pub mod_id: u64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeCategory {
    pub id: u64,
    pub game_id: u32,
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub class_id: Option<u32>,
    #[serde(default)]
    pub parent_category_id: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeScreenshot {
    pub id: u64,
    pub mod_id: u64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CurseForgeDescriptionResponse {
    pub data: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeFile {
    pub id: u64,
    pub game_id: u32,
    pub mod_id: u64,
    pub is_available: bool,
    pub display_name: String,
    pub file_name: String,
    pub release_type: u32,
    pub file_status: u32,
    pub file_date: String,
    pub file_length: u64,
    pub download_count: u64,
    pub download_url: Option<String>,
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub sortable_game_versions: Vec<CurseForgeSortableGameVersion>,
    #[serde(default)]
    pub dependencies: Vec<CurseForgeFileDependency>,
    #[serde(default)]
    pub hashes: Vec<CurseForgeFileHash>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeSortableGameVersion {
    pub game_version_name: String,
    pub game_version_padded: String,
    pub game_version: String,
    pub game_version_release_date: String,
    pub game_version_type_id: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeFileDependency {
    pub mod_id: u64,
    pub relation_type: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeFileHash {
    pub value: String,
    pub algo: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeFileIndex {
    pub game_version: String,
    pub file_id: u64,
    pub filename: String,
    pub release_type: u32,
    #[serde(default)]
    pub game_version_type_id: Option<u32>,
    #[serde(default)]
    pub mod_loader: Option<u32>,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse ISO 8601 date to Unix timestamp
fn parse_date(date: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(date)
        .ok()
        .map(|dt| dt.timestamp())
}

/// Extract loaders from game versions and file indexes
fn extract_loaders(
    game_versions: &[String],
    file_indexes: &[CurseForgeFileIndex],
) -> Vec<LoaderType> {
    let mut loaders = Vec::new();

    // Check game versions for loader names
    for v in game_versions {
        let lower = v.to_lowercase();
        if lower.contains("forge") && !lower.contains("neoforge") {
            if !loaders.contains(&LoaderType::Forge) {
                loaders.push(LoaderType::Forge);
            }
        } else if lower.contains("neoforge") {
            if !loaders.contains(&LoaderType::NeoForge) {
                loaders.push(LoaderType::NeoForge);
            }
        } else if lower.contains("fabric") && !loaders.contains(&LoaderType::Fabric) {
            loaders.push(LoaderType::Fabric);
        } else if lower.contains("quilt") && !loaders.contains(&LoaderType::Quilt) {
            loaders.push(LoaderType::Quilt);
        }
    }

    // Also check file indexes for mod loader type
    for idx in file_indexes {
        if let Some(loader_type) = idx.mod_loader {
            let cf_loader = match loader_type {
                1 => Some(LoaderType::Forge),
                4 => Some(LoaderType::Fabric),
                5 => Some(LoaderType::Quilt),
                6 => Some(LoaderType::NeoForge),
                _ => None,
            };
            if let Some(l) = cf_loader {
                if !loaders.contains(&l) {
                    loaders.push(l);
                }
            }
        }
    }

    loaders
}

/// Extract MC versions from game versions (filter out loader names)
fn extract_mc_versions(game_versions: &[String]) -> Vec<String> {
    game_versions
        .iter()
        .filter(|v| {
            let lower = v.to_lowercase();
            // Keep if it looks like a version number (starts with digit or is a known MC version pattern)
            !lower.contains("forge")
                && !lower.contains("fabric")
                && !lower.contains("quilt")
                && !lower.contains("neoforge")
                && !lower.contains("liteloader")
                && (v
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                    || v.starts_with("Snapshot"))
        })
        .cloned()
        .collect()
}

// ============================================================================
// Modpack Functions
// ============================================================================

/// Search for modpacks on CurseForge
pub async fn search_modpacks(
    client: &Client,
    api_key: &str,
    params: &ModpackSearchParams,
) -> Result<ModpackSearchResult, AppError> {
    let page = params.page.unwrap_or(0);
    let page_size = params.page_size.unwrap_or(20).min(50);
    let index = page * page_size;

    let mut url = format!(
        "{}/mods/search?gameId={}&classId={}&index={}&pageSize={}",
        CURSEFORGE_API_BASE,
        MINECRAFT_GAME_ID,
        CurseForgeClassId::Modpacks as u32,
        index,
        page_size
    );

    if let Some(ref query) = params.query {
        if !query.is_empty() {
            url.push_str(&format!("&searchFilter={}", urlencoding::encode(query)));
        }
    }

    if let Some(ref mc_version) = params.mc_version {
        url.push_str(&format!("&gameVersion={}", urlencoding::encode(mc_version)));
    }

    if let Some(ref loader) = params.loader {
        let loader_type = CurseForgeModLoaderType::from_loader_type(loader);
        if loader_type != CurseForgeModLoaderType::Any {
            url.push_str(&format!("&modLoaderType={}", loader_type as u32));
        }
    }

    if let Some(ref category) = params.category {
        if let Some(category_id) = modpack_category_to_id(category) {
            url.push_str(&format!("&categoryId={}", category_id));
        }
    }

    let sort_field = params
        .sort_by
        .as_ref()
        .map(CurseForgeSortField::from_sort_by)
        .unwrap_or(CurseForgeSortField::TotalDownloads);
    url.push_str(&format!("&sortField={}&sortOrder=desc", sort_field as u32));

    let response: CurseForgeResponse<Vec<CurseForgeMod>> = client
        .get(&url)
        .header("x-api-key", api_key)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let total_count = response
        .pagination
        .as_ref()
        .map(|p| p.total_count)
        .unwrap_or(0);

    let modpacks = response
        .data
        .into_iter()
        .map(|m| {
            let author = m
                .authors
                .first()
                .map(|a| a.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            let mc_versions = extract_mc_versions(
                &m.latest_files
                    .iter()
                    .flat_map(|f| f.game_versions.clone())
                    .collect::<Vec<_>>(),
            );
            let loaders = extract_loaders(
                &m.latest_files
                    .iter()
                    .flat_map(|f| f.game_versions.clone())
                    .collect::<Vec<_>>(),
                &m.latest_files_indexes,
            );

            Modpack {
                id: m.id.to_string(),
                slug: m.slug.clone(),
                name: m.name,
                author,
                description: m.summary,
                body: None,
                icon_url: m.logo.map(|l| l.url),
                banner_url: None,
                downloads: m.download_count,
                platform: ModpackPlatform::CurseForge,
                categories: m.categories.into_iter().map(|c| c.name).collect(),
                gallery: m
                    .screenshots
                    .into_iter()
                    .map(|s| ContentGalleryImage {
                        url: s.thumbnail_url.unwrap_or_else(|| s.url.clone()),
                        raw_url: Some(s.url),
                        title: s.title,
                        description: s.description,
                        featured: false,
                    })
                    .collect(),
                mc_versions,
                loaders,
                latest_version: None,
                url: Some(format!(
                    "https://www.curseforge.com/minecraft/modpacks/{}",
                    m.slug
                )),
                updated_at: parse_date(&m.date_modified),
                created_at: parse_date(&m.date_created),
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

/// Get a modpack by ID
pub async fn get_modpack(
    client: &Client,
    api_key: &str,
    mod_id: &str,
) -> Result<Modpack, AppError> {
    let url = format!("{}/mods/{}", CURSEFORGE_API_BASE, mod_id);

    let response: CurseForgeResponse<CurseForgeMod> = client
        .get(&url)
        .header("x-api-key", api_key)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| {
            if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                AppError::ModpackNotFound(mod_id.to_string())
            } else {
                AppError::ApiError(e.to_string())
            }
        })?
        .json()
        .await?;

    let m = response.data;
    // Fetch full description (HTML)
    let description_url = format!("{}/mods/{}/description", CURSEFORGE_API_BASE, mod_id);
    let description_html: Option<String> = match client
        .get(&description_url)
        .header("x-api-key", api_key)
        .send()
        .await
    {
        Ok(resp) => resp
            .json::<CurseForgeDescriptionResponse>()
            .await
            .ok()
            .map(|d| d.data),
        Err(_) => None,
    };
    let author = m
        .authors
        .first()
        .map(|a| a.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let summary = m.summary.clone();
    let mc_versions = extract_mc_versions(
        &m.latest_files
            .iter()
            .flat_map(|f| f.game_versions.clone())
            .collect::<Vec<_>>(),
    );
    let loaders = extract_loaders(
        &m.latest_files
            .iter()
            .flat_map(|f| f.game_versions.clone())
            .collect::<Vec<_>>(),
        &m.latest_files_indexes,
    );

    Ok(Modpack {
        id: m.id.to_string(),
        slug: m.slug.clone(),
        name: m.name,
        author,
        description: summary.clone(),
        body: description_html.or(Some(summary)),
        icon_url: m.logo.map(|l| l.url),
        banner_url: None,
        downloads: m.download_count,
        platform: ModpackPlatform::CurseForge,
        categories: m.categories.into_iter().map(|c| c.name).collect(),
        gallery: m
            .screenshots
            .into_iter()
            .map(|s| ContentGalleryImage {
                url: s.thumbnail_url.unwrap_or_else(|| s.url.clone()),
                raw_url: Some(s.url),
                title: s.title,
                description: s.description,
                featured: false,
            })
            .collect(),
        mc_versions,
        loaders,
        latest_version: None,
        url: Some(format!(
            "https://www.curseforge.com/minecraft/modpacks/{}",
            m.slug
        )),
        updated_at: parse_date(&m.date_modified),
        created_at: parse_date(&m.date_created),
    })
}

/// Get versions (files) for a modpack
pub async fn get_modpack_versions(
    client: &Client,
    api_key: &str,
    mod_id: &str,
) -> Result<Vec<ModpackVersion>, AppError> {
    let url = format!("{}/mods/{}/files?pageSize=50", CURSEFORGE_API_BASE, mod_id);

    let response: CurseForgeResponse<Vec<CurseForgeFile>> = client
        .get(&url)
        .header("x-api-key", api_key)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response
        .data
        .into_iter()
        .filter(|f| f.is_available)
        .map(|f| {
            let mc_versions = extract_mc_versions(&f.game_versions);
            let loaders = extract_loaders(&f.game_versions, &[]);
            let loader_type = loaders.first().cloned().unwrap_or(LoaderType::Vanilla);

            // Get sha1 hash
            let hash = f
                .hashes
                .iter()
                .find(|h| h.algo == 1)
                .map(|h| h.value.clone());

            ModpackVersion {
                id: f.id.to_string(),
                name: f.display_name,
                mc_version: mc_versions.first().cloned().unwrap_or_default(),
                loader_type,
                loader_version: None,
                changelog: None, // Would need separate API call for changelog
                released_at: parse_date(&f.file_date),
                downloads: Some(f.download_count),
                files: vec![ModpackFile {
                    url: f.download_url.unwrap_or_default(),
                    hash,
                    hash_algorithm: Some("sha1".to_string()),
                    size: f.file_length,
                    path: f.file_name,
                    required: true,
                }],
            }
        })
        .collect())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseForgeManifest {
    #[serde(default)]
    files: Vec<CurseForgeManifestFile>,
}

#[derive(Debug, Deserialize)]
struct CurseForgeManifestFile {
    #[serde(rename = "projectID")]
    project_id: u32,
    #[serde(rename = "fileID")]
    #[allow(dead_code)]
    file_id: u32,
}

#[derive(Debug, serde::Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseForgeGetModsRequest {
    mod_ids: Vec<u32>,
}

pub async fn get_mods_by_ids(
    client: &Client,
    api_key: &str,
    mod_ids: &[u32],
) -> Result<Vec<CurseForgeMod>, AppError> {
    if mod_ids.is_empty() {
        return Ok(vec![]);
    }
    let url = format!("{}/mods", CURSEFORGE_API_BASE);
    let response: CurseForgeResponse<Vec<CurseForgeMod>> = client
        .post(&url)
        .header("x-api-key", api_key)
        .json(&CurseForgeGetModsRequest {
            mod_ids: mod_ids.to_vec(),
        })
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(response.data)
}

/// Get a mod list for a CurseForge modpack version (best-effort)
pub async fn get_modpack_mods(
    client: &Client,
    api_key: &str,
    modpack_id: &str,
    version_id: &str,
) -> Result<Vec<ModpackMod>, AppError> {
    let modpack_project_id: u32 = modpack_id
        .parse()
        .map_err(|_| AppError::ModpackNotFound(modpack_id.to_string()))?;
    let file_id: u32 = version_id
        .parse()
        .map_err(|_| AppError::ContentNotFound("Invalid version id".to_string()))?;

    let file_info = get_mod_file(client, api_key, modpack_project_id, file_id).await?;
    let zip_bytes = client
        .get(&file_info.download_url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec();

    let cursor = std::io::Cursor::new(&zip_bytes);
    let mut archive = zip::ZipArchive::new(cursor)?;

    // manifest.json contains the mod list for CurseForge packs
    let manifest: CurseForgeManifest = {
        let mut manifest_file = archive.by_name("manifest.json")?;
        let mut contents = String::new();
        manifest_file.read_to_string(&mut contents)?;
        serde_json::from_str(&contents)?
    };

    let mut mod_ids: Vec<u32> = manifest.files.into_iter().map(|f| f.project_id).collect();
    mod_ids.sort();
    mod_ids.dedup();

    // CurseForge API has practical limits; chunk requests
    let mut mods: Vec<ModpackMod> = Vec::new();
    for chunk in mod_ids.chunks(50) {
        let chunk_mods = get_mods_by_ids(client, api_key, chunk)
            .await
            .unwrap_or_default();
        for m in chunk_mods {
            let author = m.authors.first().map(|a| a.name.clone());
            mods.push(ModpackMod {
                id: m.id.to_string(),
                name: m.name.clone(),
                icon_url: m.logo.as_ref().map(|l| l.url.clone()),
                author,
                url: Some(format!(
                    "https://www.curseforge.com/minecraft/mc-mods/{}",
                    m.slug
                )),
            });
        }
    }

    mods.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(mods)
}

// ============================================================================
// Content Functions (Mods, Shaders, Resource Packs)
// ============================================================================

/// Search for content on CurseForge
pub async fn search_content(
    client: &Client,
    api_key: &str,
    params: &ContentSearchParams,
) -> Result<ContentSearchResult, AppError> {
    let page = params.page.unwrap_or(0);
    let page_size = params.page_size.unwrap_or(20).min(50);
    let index = page * page_size;

    let class_id = params
        .content_type
        .as_ref()
        .map(CurseForgeClassId::from_content_type)
        .unwrap_or(CurseForgeClassId::Mods);

    let mut url = format!(
        "{}/mods/search?gameId={}&classId={}&index={}&pageSize={}",
        CURSEFORGE_API_BASE, MINECRAFT_GAME_ID, class_id as u32, index, page_size
    );

    if let Some(ref query) = params.query {
        if !query.is_empty() {
            url.push_str(&format!("&searchFilter={}", urlencoding::encode(query)));
        }
    }

    if let Some(ref mc_version) = params.mc_version {
        url.push_str(&format!("&gameVersion={}", urlencoding::encode(mc_version)));
    }

    if let Some(ref loader) = params.loader {
        let loader_type = CurseForgeModLoaderType::from_loader_type(loader);
        if loader_type != CurseForgeModLoaderType::Any {
            url.push_str(&format!("&modLoaderType={}", loader_type as u32));
        }
    }

    if let Some(ref category) = params.category {
        url.push_str(&format!("&categoryId={}", urlencoding::encode(category)));
    }

    let sort_field = params
        .sort_by
        .as_ref()
        .map(|s| match s {
            crate::models::ContentSortBy::Downloads => CurseForgeSortField::TotalDownloads,
            crate::models::ContentSortBy::RecentlyUpdated => CurseForgeSortField::LastUpdated,
            crate::models::ContentSortBy::Name => CurseForgeSortField::Name,
            crate::models::ContentSortBy::Relevance => CurseForgeSortField::Featured,
        })
        .unwrap_or(CurseForgeSortField::TotalDownloads);
    url.push_str(&format!("&sortField={}&sortOrder=desc", sort_field as u32));

    let response: CurseForgeResponse<Vec<CurseForgeMod>> = client
        .get(&url)
        .header("x-api-key", api_key)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let total_count = response
        .pagination
        .as_ref()
        .map(|p| p.total_count)
        .unwrap_or(0);

    let content_type = params.content_type.unwrap_or(ContentType::Mod);
    let url_path = match content_type {
        ContentType::Mod => "mc-mods",
        ContentType::Shader => "shaders",
        ContentType::ResourcePack => "texture-packs",
    };

    let items = response
        .data
        .into_iter()
        .map(|m| {
            let author = m
                .authors
                .first()
                .map(|a| a.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            let mc_versions = extract_mc_versions(
                &m.latest_files
                    .iter()
                    .flat_map(|f| f.game_versions.clone())
                    .collect::<Vec<_>>(),
            );
            let loaders = extract_loaders(
                &m.latest_files
                    .iter()
                    .flat_map(|f| f.game_versions.clone())
                    .collect::<Vec<_>>(),
                &m.latest_files_indexes,
            );

            Content {
                id: m.id.to_string(),
                slug: m.slug.clone(),
                name: m.name,
                author,
                description: m.summary,
                body: None,
                icon_url: m.logo.map(|l| l.url),
                downloads: m.download_count,
                platform: ContentPlatform::CurseForge,
                content_type,
                categories: m.categories.into_iter().map(|c| c.name).collect(),
                gallery: Vec::new(),
                mc_versions,
                loaders,
                latest_version: None,
                url: Some(format!(
                    "https://www.curseforge.com/minecraft/{}/{}",
                    url_path, m.slug
                )),
                updated_at: parse_date(&m.date_modified),
                created_at: parse_date(&m.date_created),
            }
        })
        .collect();

    Ok(ContentSearchResult {
        items,
        total_count,
        page,
        page_size,
    })
}

/// Get content by ID
pub async fn get_content(
    client: &Client,
    api_key: &str,
    mod_id: &str,
) -> Result<Content, AppError> {
    let url = format!("{}/mods/{}", CURSEFORGE_API_BASE, mod_id);

    let response: CurseForgeResponse<CurseForgeMod> = client
        .get(&url)
        .header("x-api-key", api_key)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| {
            if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                AppError::ContentNotFound(mod_id.to_string())
            } else {
                AppError::ApiError(e.to_string())
            }
        })?
        .json()
        .await?;

    let m = response.data;
    // Fetch full description (HTML)
    let description_url = format!("{}/mods/{}/description", CURSEFORGE_API_BASE, mod_id);
    let description_html: Option<String> = match client
        .get(&description_url)
        .header("x-api-key", api_key)
        .send()
        .await
    {
        Ok(resp) => resp
            .json::<CurseForgeDescriptionResponse>()
            .await
            .ok()
            .map(|d| d.data),
        Err(_) => None,
    };

    // Determine content type from class ID
    let content_type = match m.class_id {
        Some(6) => ContentType::Mod,
        Some(12) => ContentType::ResourcePack,
        Some(6552) => ContentType::Shader,
        _ => ContentType::Mod,
    };

    let url_path = match content_type {
        ContentType::Mod => "mc-mods",
        ContentType::Shader => "shaders",
        ContentType::ResourcePack => "texture-packs",
    };

    let author = m
        .authors
        .first()
        .map(|a| a.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let summary = m.summary.clone();
    let mc_versions = extract_mc_versions(
        &m.latest_files
            .iter()
            .flat_map(|f| f.game_versions.clone())
            .collect::<Vec<_>>(),
    );
    let loaders = extract_loaders(
        &m.latest_files
            .iter()
            .flat_map(|f| f.game_versions.clone())
            .collect::<Vec<_>>(),
        &m.latest_files_indexes,
    );

    Ok(Content {
        id: m.id.to_string(),
        slug: m.slug.clone(),
        name: m.name,
        author,
        description: summary.clone(),
        body: description_html.or(Some(summary)),
        icon_url: m.logo.map(|l| l.url),
        downloads: m.download_count,
        platform: ContentPlatform::CurseForge,
        content_type,
        categories: m.categories.into_iter().map(|c| c.name).collect(),
        gallery: m
            .screenshots
            .into_iter()
            .map(|s| ContentGalleryImage {
                url: s.thumbnail_url.unwrap_or_else(|| s.url.clone()),
                raw_url: Some(s.url),
                title: s.title,
                description: s.description,
                featured: false,
            })
            .collect(),
        mc_versions,
        loaders,
        latest_version: None,
        url: Some(format!(
            "https://www.curseforge.com/minecraft/{}/{}",
            url_path, m.slug
        )),
        updated_at: parse_date(&m.date_modified),
        created_at: parse_date(&m.date_created),
    })
}

/// Get versions (files) for content
pub async fn get_content_versions(
    client: &Client,
    api_key: &str,
    mod_id: &str,
    mc_version: Option<&str>,
    loader: Option<&LoaderType>,
) -> Result<Vec<ContentVersion>, AppError> {
    let mut url = format!("{}/mods/{}/files?pageSize=50", CURSEFORGE_API_BASE, mod_id);

    if let Some(mc) = mc_version {
        url.push_str(&format!("&gameVersion={}", urlencoding::encode(mc)));
    }

    if let Some(l) = loader {
        let loader_type = CurseForgeModLoaderType::from_loader_type(l);
        if loader_type != CurseForgeModLoaderType::Any {
            url.push_str(&format!("&modLoaderType={}", loader_type as u32));
        }
    }

    let response: CurseForgeResponse<Vec<CurseForgeFile>> = client
        .get(&url)
        .header("x-api-key", api_key)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response
        .data
        .into_iter()
        .filter(|f| f.is_available)
        .map(|f| {
            let mc_versions = extract_mc_versions(&f.game_versions);
            let loaders = extract_loaders(&f.game_versions, &[]);

            // Get sha1 hash
            let hash = f
                .hashes
                .iter()
                .find(|h| h.algo == 1)
                .map(|h| h.value.clone());

            // Build download URL - CurseForge sometimes doesn't provide it for third-party distribution
            let download_url = f.download_url.unwrap_or_else(|| {
                // Fallback to edge CDN URL
                format!(
                    "https://edge.forgecdn.net/files/{}/{}/{}",
                    f.id / 1000,
                    f.id % 1000,
                    urlencoding::encode(&f.file_name)
                )
            });

            ContentVersion {
                id: f.id.to_string(),
                project_id: mod_id.to_string(),
                name: f.display_name.clone(),
                version_number: f.display_name,
                mc_versions,
                loaders,
                released_at: parse_date(&f.file_date),
                downloads: Some(f.download_count),
                files: vec![ContentFile {
                    url: download_url,
                    hash,
                    hash_algorithm: Some("sha1".to_string()),
                    size: f.file_length,
                    filename: f.file_name,
                    primary: true,
                }],
                dependencies: f
                    .dependencies
                    .into_iter()
                    .filter_map(|d| {
                        let dep_type = match d.relation_type {
                            3 => DependencyType::Required,
                            2 => DependencyType::Optional,
                            5 => DependencyType::Incompatible,
                            1 | 6 => DependencyType::Embedded,
                            _ => return None,
                        };
                        Some(ContentDependency {
                            id: d.mod_id.to_string(),
                            name: None, // Would need separate API call to get name
                            dependency_type: dep_type,
                            version_req: None,
                        })
                    })
                    .collect(),
                changelog: None,
            }
        })
        .collect())
}

/// Get a specific file (version) by ID
pub async fn get_version(
    client: &Client,
    api_key: &str,
    mod_id: &str,
    file_id: &str,
) -> Result<ContentVersion, AppError> {
    let url = format!("{}/mods/{}/files/{}", CURSEFORGE_API_BASE, mod_id, file_id);

    let response: CurseForgeResponse<CurseForgeFile> = client
        .get(&url)
        .header("x-api-key", api_key)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let f = response.data;
    let mc_versions = extract_mc_versions(&f.game_versions);
    let loaders = extract_loaders(&f.game_versions, &[]);
    let hash = f
        .hashes
        .iter()
        .find(|h| h.algo == 1)
        .map(|h| h.value.clone());

    // Build download URL - CurseForge sometimes doesn't provide it for third-party distribution
    let download_url = f.download_url.unwrap_or_else(|| {
        // Fallback to edge CDN URL
        format!(
            "https://edge.forgecdn.net/files/{}/{}/{}",
            f.id / 1000,
            f.id % 1000,
            urlencoding::encode(&f.file_name)
        )
    });

    Ok(ContentVersion {
        id: f.id.to_string(),
        project_id: mod_id.to_string(),
        name: f.display_name.clone(),
        version_number: f.display_name,
        mc_versions,
        loaders,
        released_at: parse_date(&f.file_date),
        downloads: Some(f.download_count),
        files: vec![ContentFile {
            url: download_url,
            hash,
            hash_algorithm: Some("sha1".to_string()),
            size: f.file_length,
            filename: f.file_name,
            primary: true,
        }],
        dependencies: f
            .dependencies
            .into_iter()
            .filter_map(|d| {
                let dep_type = match d.relation_type {
                    3 => DependencyType::Required,
                    2 => DependencyType::Optional,
                    5 => DependencyType::Incompatible,
                    1 | 6 => DependencyType::Embedded,
                    _ => return None,
                };
                Some(ContentDependency {
                    id: d.mod_id.to_string(),
                    name: None,
                    dependency_type: dep_type,
                    version_req: None,
                })
            })
            .collect(),
        changelog: None,
    })
}

/// File info for modpack installation
#[derive(Debug, Clone)]
pub struct ModFileInfo {
    pub filename: String,
    pub download_url: String,
    pub file_length: u64,
}

/// Get a specific mod file from CurseForge (for modpack installation)
pub async fn get_mod_file(
    client: &Client,
    api_key: &str,
    project_id: u32,
    file_id: u32,
) -> Result<ModFileInfo, AppError> {
    let url = format!(
        "{}/mods/{}/files/{}",
        CURSEFORGE_API_BASE, project_id, file_id
    );

    let response: CurseForgeResponse<CurseForgeFile> = client
        .get(&url)
        .header("x-api-key", api_key)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let f = response.data;

    // Build download URL - CurseForge sometimes has it in the response
    let download_url = f.download_url.unwrap_or_else(|| {
        // Fallback to edge CDN URL if direct URL not available
        format!(
            "https://edge.forgecdn.net/files/{}/{}/{}",
            file_id / 1000,
            file_id % 1000,
            urlencoding::encode(&f.file_name)
        )
    });

    Ok(ModFileInfo {
        filename: f.file_name,
        download_url,
        file_length: f.file_length,
    })
}

/// Fingerprint match result from CurseForge API
#[derive(Debug, Clone)]
pub struct FingerprintMatch {
    pub mod_id: u64,
    pub file_id: u64,
    pub file_name: String,
}

/// Response structure for fingerprint lookup
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FingerprintResponseData {
    exact_matches: Vec<FingerprintExactMatch>,
    #[allow(dead_code)]
    exact_fingerprints: Vec<u32>,
    #[allow(dead_code)]
    #[serde(default)]
    partial_matches: Vec<serde_json::Value>,
    #[allow(dead_code)]
    #[serde(default)]
    installed_fingerprints: Vec<u32>,
    #[allow(dead_code)]
    #[serde(default)]
    unmatched_fingerprints: Vec<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FingerprintExactMatch {
    #[allow(dead_code)]
    id: u64,
    file: FingerprintFile,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FingerprintFile {
    id: u64,
    mod_id: u64,
    file_name: String,
    file_fingerprint: u32,
}

#[derive(Debug, Deserialize)]
struct FingerprintApiResponse {
    data: FingerprintResponseData,
}

/// Look up files by their murmur2 fingerprints (batch)
/// Returns a map of fingerprint -> match info
pub async fn get_files_from_fingerprints(
    client: &Client,
    api_key: &str,
    fingerprints: &[u32],
) -> Result<std::collections::HashMap<u32, FingerprintMatch>, AppError> {
    if fingerprints.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let url = format!("{}/fingerprints", CURSEFORGE_API_BASE);

    let body = serde_json::json!({
        "fingerprints": fingerprints
    });

    let response: FingerprintApiResponse = client
        .post(&url)
        .header("x-api-key", api_key)
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let mut result = std::collections::HashMap::new();
    for exact_match in response.data.exact_matches {
        let fingerprint = exact_match.file.file_fingerprint;
        result.insert(
            fingerprint,
            FingerprintMatch {
                mod_id: exact_match.file.mod_id,
                file_id: exact_match.file.id,
                file_name: exact_match.file.file_name,
            },
        );
    }

    Ok(result)
}
