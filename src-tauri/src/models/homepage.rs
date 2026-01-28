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
    /// Game mode (survival, creative, adventure, spectator)
    pub game_mode: Option<String>,
    /// Whether this world supports quick play (MC >= 1.20)
    pub supports_quick_play: bool,
}

/// Server data with instance context for homepage display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomepageServer {
    pub instance_id: String,
    pub instance_name: String,
    pub name: String,
    pub ip: String,
    pub icon_base64: Option<String>,
}

/// Aggregated stats for homepage display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomepageStats {
    /// Total play time across all instances in seconds
    pub total_play_time: u64,
    /// Total number of instances
    pub instance_count: u32,
    /// Total number of worlds across all instances
    pub world_count: u32,
    /// Total number of screenshots across all instances
    pub screenshot_count: u32,
}

/// Aggregated homepage data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomepageData {
    pub recent_screenshots: Vec<HomepageScreenshot>,
    pub most_played_instances: Vec<Instance>,
    pub most_played_worlds: Vec<HomepageWorld>,
    /// Last played instance for "Continue Playing" section
    pub continue_instance: Option<Instance>,
    /// Aggregated servers from all instances
    pub favorite_servers: Vec<HomepageServer>,
    /// Aggregated stats
    pub stats: HomepageStats,
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
