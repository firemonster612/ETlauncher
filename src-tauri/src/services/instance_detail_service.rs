use crate::error::AppError;
use crate::models::instance_detail::{
    InstanceDetail, Screenshot, ScreenshotsResponse, Server, ServersResponse, World, WorldsResponse,
};
use base64::Engine;
use chrono::NaiveDateTime;
use quartz_nbt::io::Flavor;
use quartz_nbt::serde::deserialize_from;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Get all screenshots from the instance game directory
pub fn get_screenshots(game_dir: &Path) -> Result<ScreenshotsResponse, AppError> {
    let screenshots_dir = game_dir.join("screenshots");
    if !screenshots_dir.exists() {
        return Ok(ScreenshotsResponse {
            screenshots: Vec::new(),
        });
    }

    let mut screenshots = Vec::new();
    for entry in fs::read_dir(&screenshots_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if !ext.eq_ignore_ascii_case("png") && !ext.eq_ignore_ascii_case("jpg") {
                    continue;
                }
            }

            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
            let size = path.metadata().map(|m| m.len()).unwrap_or(0);

            // Try to parse timestamp from filename pattern: YYYY-MM-DD_HH.MM.SS
            let taken_at = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|stem| NaiveDateTime::parse_from_str(stem, "%Y-%m-%d_%H.%M.%S").ok())
                .map(|dt| dt.and_utc().timestamp_millis())
                .or_else(|| {
                    path.metadata()
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| {
                            t.duration_since(std::time::UNIX_EPOCH)
                                .ok()
                                .map(|d| d.as_millis() as i64)
                        })
                })
                .unwrap_or(0);

            screenshots.push(Screenshot {
                filename,
                path: path.to_string_lossy().to_string(),
                size,
                taken_at,
            });
        }
    }

    // Sort newest first
    screenshots.sort_by(|a, b| b.taken_at.cmp(&a.taken_at));

    Ok(ScreenshotsResponse { screenshots })
}

/// Get all worlds from the instance game directory
pub fn get_worlds(game_dir: &Path) -> Result<WorldsResponse, AppError> {
    let saves_dir = game_dir.join("saves");
    if !saves_dir.exists() {
        return Ok(WorldsResponse { worlds: Vec::new() });
    }

    let mut worlds = Vec::new();

    for entry in fs::read_dir(&saves_dir)? {
        let entry = entry?;
        let world_dir = entry.path();

        if !world_dir.is_dir() {
            continue;
        }

        let folder_name = world_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();

        let (mut name, mut last_played, mut game_mode, mut cheats_enabled, mut version_name) =
            (folder_name.clone(), None, None, false, None);

        // Parse level.dat (gz compressed NBT)
        let level_dat = world_dir.join("level.dat");
        if level_dat.exists() {
            if let Ok((level, _)) = deserialize_from::<_, LevelDatRoot>(
                &mut File::open(&level_dat)?,
                Flavor::GzCompressed,
            ) {
                let data = level.data;
                if let Some(level_name) = data.level_name {
                    name = level_name;
                }
                last_played = data.last_played;
                game_mode = data.game_type.map(map_game_mode);
                cheats_enabled = data.allow_commands.unwrap_or(0) != 0;
                version_name = data.version.and_then(|v| v.name);
            }
        }

        let size = get_directory_size(&world_dir).unwrap_or(0);
        let icon_base64 = read_base64_file(&world_dir.join("icon.png"))?;

        worlds.push(World {
            folder_name,
            name,
            path: world_dir.to_string_lossy().to_string(),
            last_played,
            game_mode,
            cheats_enabled,
            version_name,
            icon_base64,
            size,
        });
    }

    // Sort by last played (desc), fallback to size as a stable-ish order
    worlds.sort_by(|a, b| b.last_played.cmp(&a.last_played));

    Ok(WorldsResponse { worlds })
}

/// Get saved multiplayer servers
pub fn get_servers(game_dir: &Path) -> Result<ServersResponse, AppError> {
    let servers_file = game_dir.join("servers.dat");
    if !servers_file.exists() {
        return Ok(ServersResponse {
            servers: Vec::new(),
        });
    }

    let (data, _) = deserialize_from::<_, ServersDatRoot>(
        &mut File::open(&servers_file)?,
        Flavor::Uncompressed,
    )
    .map_err(|e| AppError::InvalidInput(format!("Failed to decode servers.dat: {}", e)))?;

    let servers = data
        .servers
        .into_iter()
        .map(|s| Server {
            name: s.name.unwrap_or_else(|| "Unknown Server".to_string()),
            ip: s.ip.unwrap_or_default(),
            icon_base64: s.icon,
            hidden: s.hide_address.unwrap_or(false),
            accept_textures: s.accept_textures.unwrap_or(false),
        })
        .collect();

    Ok(ServersResponse { servers })
}

/// Convenience wrapper that returns dashboard-friendly slice of detail
pub fn get_instance_detail(
    game_dir: &Path,
    total_play_time: u64,
) -> Result<InstanceDetail, AppError> {
    let screenshots = get_screenshots(game_dir)?.screenshots;
    let worlds = get_worlds(game_dir)?.worlds;
    let servers = get_servers(game_dir)?.servers;

    Ok(InstanceDetail {
        total_play_time,
        recent_screenshots: screenshots.into_iter().take(6).collect(),
        recent_worlds: worlds.into_iter().take(5).collect(),
        saved_servers: servers.into_iter().take(5).collect(),
    })
}

/// Return base64 encoded screenshot data
pub fn get_screenshot_data(game_dir: &Path, filename: &str) -> Result<String, AppError> {
    let safe_name = PathBuf::from(filename);
    let file_name = safe_name
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AppError::InvalidInput("Invalid screenshot filename".to_string()))?;

    let screenshot_path = game_dir.join("screenshots").join(file_name);
    let data = fs::read(&screenshot_path)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(data))
}

fn map_game_mode(id: i32) -> String {
    match id {
        0 => "survival",
        1 => "creative",
        2 => "adventure",
        3 => "spectator",
        _ => "unknown",
    }
    .to_string()
}

fn get_directory_size(path: &Path) -> Result<u64, AppError> {
    let mut size: u64 = 0;
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                size += get_directory_size(&entry_path)?;
            } else {
                size += entry_path.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    Ok(size)
}

fn read_base64_file(path: &Path) -> Result<Option<String>, AppError> {
    if !path.exists() {
        return Ok(None);
    }

    let data = fs::read(path)?;
    Ok(Some(base64::engine::general_purpose::STANDARD.encode(data)))
}

#[derive(Debug, serde::Deserialize)]
struct LevelDatRoot {
    #[serde(rename = "Data")]
    data: LevelDatData,
}

#[derive(Debug, serde::Deserialize)]
struct LevelDatData {
    #[serde(rename = "LevelName")]
    level_name: Option<String>,
    #[serde(rename = "LastPlayed")]
    last_played: Option<i64>,
    #[serde(rename = "GameType")]
    game_type: Option<i32>,
    #[serde(rename = "allowCommands")]
    allow_commands: Option<i8>,
    #[serde(rename = "Version")]
    version: Option<LevelVersion>,
}

#[derive(Debug, serde::Deserialize)]
struct LevelVersion {
    #[serde(rename = "Name")]
    name: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ServersDatRoot {
    servers: Vec<ServerEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct ServerEntry {
    name: Option<String>,
    ip: Option<String>,
    icon: Option<String>,
    #[serde(rename = "hideAddress")]
    hide_address: Option<bool>,
    #[serde(rename = "acceptTextures")]
    accept_textures: Option<bool>,
}
