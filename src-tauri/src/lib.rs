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
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Settings commands
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::reset_settings,
            commands::settings::get_default_instances_path,
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
            commands::instance::open_instance_folder,
            commands::instance::export_instance,
            // Instance detail commands
            commands::instance_detail::get_instance_detail,
            commands::instance_detail::get_instance_screenshots,
            commands::instance_detail::get_instance_worlds,
            commands::instance_detail::get_instance_servers,
            commands::instance_detail::get_screenshot_data,
            commands::instance_detail::launch_into_world,
            commands::instance_detail::launch_into_server,
            commands::instance_detail::open_world_folder,
            commands::instance_detail::delete_screenshot,
            commands::instance_detail::delete_world,
            // Minecraft commands
            commands::minecraft::fetch_version_manifest,
            commands::minecraft::get_versions,
            commands::minecraft::get_version_info,
            commands::minecraft::download_game_files,
            // Launch commands
            commands::launch::launch_instance,
            commands::launch::is_instance_running,
            commands::launch::get_running_instances,
            commands::launch::kill_instance,
            // Loader commands
            commands::loader::get_loader_versions,
            commands::loader::install_loader,
            commands::loader::check_loader_installed,
            // Modpack commands
            commands::modpack::search_modpacks,
            commands::modpack::get_modpack,
            commands::modpack::get_modpack_versions,
            commands::modpack::get_modpack_mods,
            commands::modpack::install_modpack,
            commands::modpack::import_modpack_file,
            commands::modpack::cancel_modpack_install,
            commands::modpack::get_modpack_install_status,
            // Content commands
            commands::content::search_content,
            commands::content::get_content,
            commands::content::get_content_versions,
            commands::content::get_content_version,
            commands::content::install_content,
            commands::content::resolve_content_dependencies,
            commands::content::install_content_with_dependencies,
            commands::content::scan_installed_content,
            commands::content::uninstall_content_by_filename,
            commands::content::disable_content,
            commands::content::enable_content,
            // Update commands
            commands::update::check_modpack_update,
            commands::update::check_content_updates,
            commands::update::preview_version_migration,
            commands::update::update_instance_content,
            commands::update::migrate_instance_version,
            commands::update::get_content_manifest,
            // New update system commands
            commands::update::check_modpack_instance_updates,
            commands::update::execute_modpack_update,
            commands::update::check_instance_updates,
            commands::update::execute_instance_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
