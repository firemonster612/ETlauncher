use serde::{Deserialize, Serialize};

use super::Instance;

/// Screenshot data with instance context for homepage display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomepageScreenshot {
    pub instance_id: String,
    pub instance_name: String,
    pub filename: String,
    pub path: String,
    pub taken_at: i64,
}

/// World data with instance context for homepage display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomepageWorld {
    pub instance_id: String,
    pub instance_name: String,
    pub minecraft_version: String,
    pub folder_name: String,
    pub name: String,
    pub last_played: Option<i64>,
    pub icon_base64: Option<String>,
    /// Whether this world supports quick play (MC >= 1.20)
    pub supports_quick_play: bool,
}

/// Aggregated homepage data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomepageData {
    pub recent_screenshots: Vec<HomepageScreenshot>,
    pub most_played_instances: Vec<Instance>,
    pub most_played_worlds: Vec<HomepageWorld>,
}

/// A news article from Minecraft launcher news API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsArticle {
    pub id: String,
    pub title: String,
    pub description: String,
    pub image_url: String,
    pub article_url: String,
    pub date: String,
    pub category: String,
}

/// Response from the Minecraft news API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsResponse {
    pub articles: Vec<NewsArticle>,
}

// Raw API response structures for deserializing Mojang's news API
#[derive(Debug, Deserialize)]
pub struct MojangNewsResponse {
    pub entries: Vec<MojangNewsEntry>,
}

#[derive(Debug, Deserialize)]
pub struct MojangNewsEntry {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub text: String,
    #[serde(rename = "playPageImage")]
    pub play_page_image: Option<MojangImageInfo>,
    #[serde(rename = "newsPageImage")]
    pub news_page_image: Option<MojangImageInfo>,
    #[serde(rename = "readMoreLink")]
    pub read_more_link: Option<String>,
    #[serde(rename = "linkButton")]
    pub link_button: Option<MojangLinkButton>,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct MojangImageInfo {
    pub url: String,
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct MojangLinkButton {
    pub primary: Option<MojangLink>,
}

#[derive(Debug, Deserialize)]
pub struct MojangLink {
    pub url: Option<String>,
}
