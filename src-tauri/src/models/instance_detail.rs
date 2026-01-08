use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Screenshot {
    pub filename: String,
    pub path: String,
    pub size: u64,
    pub taken_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct World {
    pub folder_name: String,
    pub name: String,
    pub path: String,
    pub last_played: Option<i64>,
    pub game_mode: Option<String>,
    pub cheats_enabled: bool,
    pub version_name: Option<String>,
    pub icon_base64: Option<String>,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub name: String,
    pub ip: String,
    pub icon_base64: Option<String>,
    pub hidden: bool,
    pub accept_textures: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotsResponse {
    pub screenshots: Vec<Screenshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldsResponse {
    pub worlds: Vec<World>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServersResponse {
    pub servers: Vec<Server>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceDetail {
    pub total_play_time: u64,
    pub recent_screenshots: Vec<Screenshot>,
    pub recent_worlds: Vec<World>,
    pub saved_servers: Vec<Server>,
}
