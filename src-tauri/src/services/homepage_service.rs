use crate::cache::{load_disk_cache, save_disk_cache};
use crate::error::AppError;
use crate::models::homepage::{
    HomepageData, HomepageScreenshot, HomepageWorld, MojangNewsResponse, NewsArticle, NewsResponse,
};
use crate::models::Instance;
use crate::services::{instance_detail_service, instance_service};
use crate::state::AppState;
use std::cmp::Ordering;
use std::time::Duration;

/// Maximum number of recent screenshots to return
const MAX_SCREENSHOTS: usize = 12;

/// Maximum number of most played instances to return
const MAX_INSTANCES: usize = 4;

/// Maximum number of most played worlds to return
const MAX_WORLDS: usize = 4;

/// Cache duration for news (15 minutes)
const NEWS_CACHE_TTL_SECS: u64 = 900;

/// Minecraft news API URL (v2 has more recent content)
const MINECRAFT_NEWS_URL: &str = "https://launchercontent.mojang.com/v2/news.json";

/// Base URL for Mojang CDN (for relative image URLs)
const MOJANG_CDN_BASE: &str = "https://launchercontent.mojang.com";

/// Get aggregated homepage data
pub fn get_homepage_data(state: &AppState) -> Result<HomepageData, AppError> {
    let instances = instance_service::get_all_instances(state)?;

    let recent_screenshots = get_aggregated_screenshots(state, &instances)?;
    let most_played_instances = get_most_played_instances(&instances);
    let most_played_worlds = get_most_played_worlds(state, &instances)?;

    Ok(HomepageData {
        recent_screenshots,
        most_played_instances,
        most_played_worlds,
    })
}

/// Get recent screenshots from all instances, sorted by date
fn get_aggregated_screenshots(
    state: &AppState,
    instances: &[Instance],
) -> Result<Vec<HomepageScreenshot>, AppError> {
    let mut all_screenshots = Vec::new();

    for instance in instances {
        let game_dir = instance_service::get_game_directory(state, &instance.id);
        if let Ok(response) = instance_detail_service::get_screenshots(&game_dir) {
            for screenshot in response.screenshots {
                all_screenshots.push(HomepageScreenshot {
                    instance_id: instance.id.clone(),
                    instance_name: instance.name.clone(),
                    filename: screenshot.filename,
                    path: screenshot.path,
                    taken_at: screenshot.taken_at,
                });
            }
        }
    }

    // Sort by taken_at descending (newest first)
    all_screenshots.sort_by(|a, b| b.taken_at.cmp(&a.taken_at));

    // Limit to MAX_SCREENSHOTS
    all_screenshots.truncate(MAX_SCREENSHOTS);

    Ok(all_screenshots)
}

/// Get most played instances, sorted by total play time
fn get_most_played_instances(instances: &[Instance]) -> Vec<Instance> {
    let mut sorted_instances: Vec<Instance> = instances
        .iter()
        .filter(|i| i.total_play_time > 0) // Only include instances that have been played
        .cloned()
        .collect();

    // Sort by total_play_time descending
    sorted_instances.sort_by(|a, b| b.total_play_time.cmp(&a.total_play_time));

    // Limit to MAX_INSTANCES
    sorted_instances.truncate(MAX_INSTANCES);

    sorted_instances
}

/// Get most played worlds from all instances, sorted by last_played
fn get_most_played_worlds(
    state: &AppState,
    instances: &[Instance],
) -> Result<Vec<HomepageWorld>, AppError> {
    let mut all_worlds = Vec::new();

    for instance in instances {
        let game_dir = instance_service::get_game_directory(state, &instance.id);
        if let Ok(response) = instance_detail_service::get_worlds(&game_dir) {
            let supports_quick_play = supports_quick_play(&instance.minecraft_version);

            for world in response.worlds {
                all_worlds.push(HomepageWorld {
                    instance_id: instance.id.clone(),
                    instance_name: instance.name.clone(),
                    minecraft_version: instance.minecraft_version.clone(),
                    folder_name: world.folder_name,
                    name: world.name,
                    last_played: world.last_played,
                    icon_base64: world.icon_base64,
                    supports_quick_play,
                });
            }
        }
    }

    // Sort by last_played descending (most recent first), None values at the end
    all_worlds.sort_by(|a, b| match (&b.last_played, &a.last_played) {
        (Some(b_time), Some(a_time)) => b_time.cmp(a_time),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    });

    // Limit to MAX_WORLDS
    all_worlds.truncate(MAX_WORLDS);

    Ok(all_worlds)
}

/// Check if a Minecraft version supports quick play (1.20+)
fn supports_quick_play(minecraft_version: &str) -> bool {
    // Parse version like "1.20", "1.20.1", "1.21.4", etc.
    let parts: Vec<&str> = minecraft_version.split('.').collect();
    if parts.len() < 2 {
        return false;
    }

    let major: u32 = match parts[0].parse() {
        Ok(v) => v,
        Err(_) => return false,
    };

    let minor: u32 = match parts[1].parse() {
        Ok(v) => v,
        Err(_) => return false,
    };

    // Version 1.20 or later supports quick play
    major > 1 || (major == 1 && minor >= 20)
}

/// Fetch Minecraft news from Mojang API with caching
pub async fn fetch_minecraft_news(http_client: &reqwest::Client) -> Result<NewsResponse, AppError> {
    // Try to get from disk cache first
    if let Some(cached) = load_disk_cache::<NewsResponse>("homepage", "news") {
        return Ok(cached);
    }

    // Fetch from API
    let response = http_client
        .get(MINECRAFT_NEWS_URL)
        .send()
        .await
        .map_err(|e| AppError::ApiError(format!("Failed to fetch news: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::ApiError(format!(
            "News API returned status {}",
            response.status()
        )));
    }

    let mojang_response: MojangNewsResponse = response
        .json()
        .await
        .map_err(|e| AppError::ApiError(format!("Failed to parse news response: {}", e)))?;

    // Transform Mojang's format to our format
    let articles: Vec<NewsArticle> = mojang_response
        .entries
        .into_iter()
        .filter_map(|entry| {
            // Get the image URL, preferring playPageImage over newsPageImage
            let image_url = entry
                .play_page_image
                .map(|img| img.url)
                .or_else(|| entry.news_page_image.map(|img| img.url))?;

            // Convert relative URLs to absolute URLs
            let image_url = if image_url.starts_with('/') {
                format!("{}{}", MOJANG_CDN_BASE, image_url)
            } else {
                image_url
            };

            // Get the article URL from readMoreLink or linkButton
            let article_url = entry
                .read_more_link
                .or_else(|| {
                    entry
                        .link_button
                        .and_then(|lb| lb.primary.and_then(|p| p.url))
                })
                .unwrap_or_else(|| "https://www.minecraft.net/en-us/articles".to_string());

            Some(NewsArticle {
                id: entry.id,
                title: entry.title,
                description: entry.text,
                image_url,
                article_url,
                date: entry.date,
                category: entry.category,
            })
        })
        .take(10) // Limit to 10 articles
        .collect();

    let news_response = NewsResponse { articles };

    // Save to disk cache
    let _ = save_disk_cache(
        "homepage",
        "news",
        &news_response,
        Duration::from_secs(NEWS_CACHE_TTL_SECS),
    );

    Ok(news_response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supports_quick_play() {
        // Versions that should support quick play
        assert!(supports_quick_play("1.20"));
        assert!(supports_quick_play("1.20.1"));
        assert!(supports_quick_play("1.20.4"));
        assert!(supports_quick_play("1.21"));
        assert!(supports_quick_play("1.21.4"));
        assert!(supports_quick_play("2.0")); // Future versions

        // Versions that should NOT support quick play
        assert!(!supports_quick_play("1.19"));
        assert!(!supports_quick_play("1.19.4"));
        assert!(!supports_quick_play("1.18"));
        assert!(!supports_quick_play("1.16.5"));
        assert!(!supports_quick_play("1.12.2"));

        // Edge cases
        assert!(!supports_quick_play("")); // Empty string
        assert!(!supports_quick_play("invalid")); // Invalid format
    }
}
