pub mod commands;
pub mod error;
pub mod models;
pub mod services;
pub mod state;
pub mod utils;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Ensure data directories exist
    if let Err(e) = utils::paths::ensure_directories() {
        eprintln!("Failed to create data directories: {}", e);
    }

    // Create application state
    let app_state = AppState::new();

    // Load settings
    if let Err(e) = app_state.load_settings() {
        eprintln!("Failed to load settings: {}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Settings commands
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::reset_settings,
            commands::settings::get_default_instances_path,
            commands::settings::detect_java,
            // Auth commands
            commands::auth::start_device_auth,
            commands::auth::poll_device_auth,
            commands::auth::refresh_account_token,
            // Account commands
            commands::account::get_accounts,
            commands::account::get_account,
            commands::account::get_active_account,
            commands::account::set_active_account,
            commands::account::delete_account,
            commands::account::get_minecraft_profile,
            commands::account::upload_skin,
            commands::account::set_cape,
            commands::account::hide_cape,
            // Instance commands
            commands::instance::get_instances,
            commands::instance::get_instance,
            commands::instance::create_instance,
            commands::instance::update_instance,
            commands::instance::delete_instance,
            commands::instance::duplicate_instance,
            // Minecraft commands
            commands::minecraft::fetch_version_manifest,
            commands::minecraft::get_versions,
            commands::minecraft::get_version_info,
            commands::minecraft::download_game_files,
            // Launch commands
            commands::launch::launch_instance,
            commands::launch::is_instance_running,
            commands::launch::get_running_instances,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
