// Allow some clippy lints that would require extensive refactoring
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(clippy::ptr_arg)]

pub mod cache;
pub mod commands;
pub mod error;
pub mod models;
pub mod services;
pub mod state;
pub mod task_registry;
pub mod utils;

use state::AppState;
use tauri::Manager;

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

    // Auto-migrate existing instances to resource pool if enabled
    // This is fast and idempotent - already-migrated content is skipped
    if app_state.get_settings().resource_pool.enabled {
        if let Err(e) = services::migration_service::migrate_all_instances(&app_state) {
            eprintln!("Auto-migration failed: {}", e);
        }

        // Run garbage collection in background if it's been 24+ hours since last GC
        // This cleans up unused resources automatically
        services::resource_pool_service::maybe_run_gc_background(&app_state);
    }

    tauri::Builder::default()
        // Single instance MUST be registered first to work correctly
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Focus the main window when a second instance is launched
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(app_state)
        .setup(|app| {
            // Set the app handle on the task registry for event emission
            let state = app.state::<AppState>();
            state.task_registry.set_app_handle(app.handle().clone());

            // Auto-rebuild manifests for instances that have content files but no manifest.
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = app_handle.state::<AppState>();
                let instances = services::instance_service::get_all_instances(&state)
                    .unwrap_or_default();
                let needs_rebuild: Vec<String> = instances
                    .iter()
                    .filter(|inst| {
                        services::content_scan_service::needs_manifest_rebuild(&state, &inst.id)
                    })
                    .map(|inst| inst.id.clone())
                    .collect();

                if needs_rebuild.is_empty() {
                    return;
                }

                println!(
                    "[startup] Found {} instance(s) needing manifest rebuild, scanning in background...",
                    needs_rebuild.len()
                );

                for instance_id in &needs_rebuild {
                    match services::content_scan_service::rescan_and_rebuild_manifest(
                        &state,
                        instance_id,
                    )
                    .await
                    {
                        Ok(result) => {
                            println!(
                                "[startup] Rebuilt manifest for {}: {} items ({} identified)",
                                instance_id, result.total_items, result.identified_items
                            );
                        }
                        Err(e) => {
                            eprintln!(
                                "[startup] Failed to rebuild manifest for {}: {}",
                                instance_id, e
                            );
                        }
                    }
                }
                println!("[startup] Background manifest rebuild complete");
            });

            // Start the local Yggdrasil server for offline account skins.
            tauri::async_runtime::spawn(async {
                match services::yggdrasil_server::start_server().await {
                    Ok(port) => eprintln!("[app] Yggdrasil server started on port {}", port),
                    Err(e) => eprintln!("[app] Warning: Failed to start Yggdrasil server: {}", e),
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Settings commands
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::reset_settings,
            commands::settings::get_default_instances_path,
            commands::settings::clear_api_caches,
            commands::settings::get_system_theme,
            commands::settings::copy_background_file,
            commands::settings::delete_background_file,
            commands::settings::get_background_path,
            commands::settings::get_background_data,
            // Auth commands
            commands::auth::start_device_auth,
            commands::auth::poll_device_auth,
            commands::auth::refresh_account_token,
            commands::auth::create_offline_account,
            commands::auth::set_offline_skin,
            commands::auth::set_offline_cape,
            commands::auth::remove_offline_skin,
            commands::auth::remove_offline_cape,
            commands::auth::get_offline_skin_data,
            commands::auth::get_offline_cape_data,
            commands::auth::get_default_skin,
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
            // Skin library commands
            commands::skin::save_skin_to_library,
            commands::skin::get_skin_library,
            commands::skin::delete_skin_from_library,
            commands::skin::apply_skin_from_library,
            commands::skin::get_skin_data,
            commands::skin::read_skin_file,
            // Instance commands
            commands::instance::get_instances,
            commands::instance::get_instance,
            commands::instance::create_instance,
            commands::instance::update_instance,
            commands::instance::delete_instance,
            commands::instance::duplicate_instance,
            commands::instance::open_instance_folder,
            commands::instance::export_instance,
            commands::instance::setup_instance,
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
            // Homepage commands
            commands::homepage::get_homepage_data,
            commands::homepage::get_minecraft_news,
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
            commands::launch::cancel_launch,
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
            // Content commands
            commands::content::search_content,
            commands::content::get_content,
            commands::content::get_content_versions,
            commands::content::get_content_version,
            commands::content::install_content,
            commands::content::resolve_content_dependencies,
            commands::content::install_content_with_dependencies,
            commands::content::scan_installed_content,
            commands::content::rescan_instance_content,
            commands::content::uninstall_content_by_filename,
            commands::content::disable_content,
            commands::content::enable_content,
            commands::content::queue_content_install,
            commands::content::cancel_content_queue_item,
            commands::content::try_process_content_queue,
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
            commands::update::get_exe_path,
            commands::update::is_appimage,
            // OptiFine commands
            commands::optifine::check_optifine_available,
            commands::optifine::install_optifine,
            commands::optifine::get_optifine_version,
            // Resource pool commands
            commands::resource_pool::get_pool_stats,
            commands::resource_pool::garbage_collect_pool,
            commands::resource_pool::verify_pool_integrity,
            commands::resource_pool::check_instance_needs_migration,
            commands::resource_pool::migrate_instance_to_pool,
            commands::resource_pool::migrate_all_instances_to_pool,
            // Import/Export commands
            commands::import::analyze_import_source,
            commands::import::import_from_folder,
            commands::import::import_curseforge_zip,
            commands::import::export_curseforge_modpack,
            // Task registry commands
            commands::tasks::list_tasks,
            commands::tasks::cancel_task,
            commands::tasks::dismiss_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
