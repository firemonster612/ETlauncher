use crate::error::CommandError;
use crate::models::homepage::{HomepageData, NewsResponse};
use crate::services::homepage_service;
use crate::state::AppState;
use tauri::State;

/// Get aggregated homepage data (screenshots, instances, worlds)
#[tauri::command]
pub fn get_homepage_data(state: State<'_, AppState>) -> Result<HomepageData, CommandError> {
    homepage_service::get_homepage_data(&state).map_err(CommandError::from)
}

/// Get Minecraft news articles
#[tauri::command]
pub async fn get_minecraft_news(state: State<'_, AppState>) -> Result<NewsResponse, CommandError> {
    homepage_service::fetch_minecraft_news(&state.http_client)
        .await
        .map_err(CommandError::from)
}
