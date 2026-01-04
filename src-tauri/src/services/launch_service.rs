use crate::error::AppError;
use crate::models::instance::{Instance, LaunchStatus};
use crate::models::minecraft::{GameLogLine, LogLevel};
use crate::services::{account_service, download_service, instance_service};
use crate::state::AppState;
use crate::utils::paths::{get_assets_dir, get_instance_natives_dir_with_base};
use crate::utils::platform::classpath_separator;
use chrono::Utc;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use tauri::{AppHandle, Emitter, Manager};

/// Launch a Minecraft instance
pub async fn launch_instance(
    instance: &Instance,
    account_id: &str,
    app_handle: &AppHandle,
) -> Result<u32, AppError> {
    let instance_id = instance.id.clone();
    let state: tauri::State<'_, AppState> = app_handle.state();

    // Check if already running
    if state.is_instance_running(&instance_id) {
        return Err(AppError::LaunchError("Instance is already running".to_string()));
    }

    // Emit preparing status
    emit_launch_status(
        app_handle,
        &instance_id,
        LaunchStatus::Preparing {
            message: "Checking account...".to_string(),
        },
    );

    // Get and verify account
    let account = account_service::get_account(account_id)?;

    // Get valid access token (will refresh if needed)
    emit_launch_status(
        app_handle,
        &instance_id,
        LaunchStatus::Preparing {
            message: "Getting access token...".to_string(),
        },
    );

    let http_client = reqwest::Client::new();
    let access_token = account_service::get_valid_access_token(&http_client, account_id).await?;

    // Get paths
    let game_dir = instance_service::get_game_directory(&state, &instance.id);

    // Get version info (with loader support) - do this FIRST so we know all libraries needed
    emit_launch_status(
        app_handle,
        &instance_id,
        LaunchStatus::Preparing {
            message: "Loading version info...".to_string(),
        },
    );

    let version_info = download_service::get_version_info_with_loader(
        &instance.minecraft_version,
        &instance.loader_type,
        instance.loader_version.as_deref(),
        &game_dir,
    ).await?;


    // Emit downloading status
    emit_launch_status(
        app_handle,
        &instance_id,
        LaunchStatus::Preparing {
            message: "Downloading game files...".to_string(),
        },
    );

    // Download game files using the merged version info (includes loader libraries)
    download_service::download_game_files_with_version(
        &instance.id,
        &instance.minecraft_version,
        &version_info,
        Some(app_handle),
    ).await?;

    // Emit launching status
    emit_launch_status(app_handle, &instance_id, LaunchStatus::Launching);

    // Get Java path
    let java_path = instance
        .java_path
        .clone()
        .or_else(find_java)
        .ok_or(AppError::JavaNotFound)?;

    // Build classpath (pass game_dir for Forge libraries)
    let classpath = download_service::get_classpath(&version_info, &instance.minecraft_version, Some(&game_dir));
    let classpath_str = classpath
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(classpath_separator());

    // Get more paths
    let natives_dir = get_instance_natives_dir_with_base(&state.settings.read().instances_path, &instance.id);
    let assets_dir = get_assets_dir();

    // Memory settings
    let min_mem = instance.memory_min_mb.unwrap_or(512);
    let max_mem = instance.memory_max_mb.unwrap_or(2048);

    // Build replacements map
    let mut replacements: HashMap<String, String> = HashMap::new();
    replacements.insert("auth_player_name".to_string(), account.username.clone());
    replacements.insert("version_name".to_string(), instance.minecraft_version.clone());
    replacements.insert("game_directory".to_string(), game_dir.to_string_lossy().to_string());
    replacements.insert(
        "assets_root".to_string(),
        assets_dir.to_string_lossy().to_string(),
    );
    let asset_index_id = version_info.asset_index.as_ref()
        .map(|ai| ai.id.clone())
        .unwrap_or_else(|| instance.minecraft_version.clone());
    replacements.insert(
        "assets_index_name".to_string(),
        asset_index_id,
    );
    replacements.insert("auth_uuid".to_string(), account.uuid.replace('-', ""));
    replacements.insert("auth_access_token".to_string(), access_token);
    replacements.insert("user_type".to_string(), "msa".to_string());
    replacements.insert("version_type".to_string(), "release".to_string());
    replacements.insert(
        "natives_directory".to_string(),
        natives_dir.to_string_lossy().to_string(),
    );
    // NeoForge/Forge need library_directory for their mod loader
    let library_directory = game_dir.join("libraries");
    replacements.insert(
        "library_directory".to_string(),
        library_directory.to_string_lossy().to_string(),
    );
    replacements.insert("launcher_name".to_string(), "ETLauncher".to_string());
    replacements.insert("launcher_version".to_string(), "0.1.0".to_string());
    replacements.insert("classpath".to_string(), classpath_str.clone());

    // Build JVM arguments
    let mut jvm_args = vec![
        format!("-Xms{}M", min_mem),
        format!("-Xmx{}M", max_mem),
        format!("-Djava.library.path={}", natives_dir.to_string_lossy()),
    ];

    // Add version-specific JVM args
    jvm_args.extend(download_service::build_jvm_arguments(&version_info, &replacements));

    // Add custom JVM args from instance
    if let Some(ref custom_jvm) = instance.jvm_args {
        jvm_args.extend(custom_jvm.split_whitespace().map(String::from));
    }

    // Add classpath
    jvm_args.push("-cp".to_string());
    jvm_args.push(classpath_str);

    // Main class
    jvm_args.push(version_info.main_class.clone());

    // Build game arguments
    let game_args = download_service::build_game_arguments(&version_info, &replacements);

    // Add custom game args from instance
    let mut all_game_args = game_args;
    if let Some(ref custom_game) = instance.game_args {
        all_game_args.extend(custom_game.split_whitespace().map(String::from));
    }

    // Combine all arguments
    let mut all_args = jvm_args;
    all_args.extend(all_game_args);


    // Spawn the process
    let mut child = Command::new(&java_path)
        .args(&all_args)
        .current_dir(&game_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::LaunchError(format!("Failed to spawn Java: {}", e)))?;

    let pid = child.id();

    // Register as running
    state.register_running_instance(instance_id.clone(), pid);

    // Emit running status
    emit_launch_status(app_handle, &instance_id, LaunchStatus::Running { pid });

    // Capture stdout/stderr in background threads
    let app_handle_stdout = app_handle.clone();
    let instance_id_stdout = instance_id.clone();

    if let Some(stdout) = child.stdout.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                emit_game_log(&app_handle_stdout, &instance_id_stdout, &line, LogLevel::Info);
            }
        });
    }

    let app_handle_stderr = app_handle.clone();
    let instance_id_stderr = instance_id.clone();

    if let Some(stderr) = child.stderr.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                let level = if line.contains("ERROR") || line.contains("Exception") {
                    LogLevel::Error
                } else if line.contains("WARN") {
                    LogLevel::Warn
                } else {
                    LogLevel::Info
                };
                emit_game_log(&app_handle_stderr, &instance_id_stderr, &line, level);
            }
        });
    }

    // Spawn thread to wait for process and update status
    let app_handle_wait = app_handle.clone();
    let instance_id_wait = instance_id.clone();
    let start_time = std::time::Instant::now();

    thread::spawn(move || {
        let exit_status = child.wait();
        let play_duration = start_time.elapsed().as_secs();

        // Get state for updating
        let state = app_handle_wait.try_state::<AppState>();

        // Update instance play time
        if let Some(ref state) = state {
            let _ = instance_service::update_play_time(state, &instance_id_wait, play_duration);
        }

        // Unregister from running instances
        if let Some(state) = state {
            state.unregister_running_instance(&instance_id_wait);
        }

        // Emit final status
        match exit_status {
            Ok(status) => {
                let exit_code = status.code().unwrap_or(-1);
                if exit_code == 0 {
                    emit_launch_status(
                        &app_handle_wait,
                        &instance_id_wait,
                        LaunchStatus::Stopped { exit_code },
                    );
                } else {
                    emit_launch_status(
                        &app_handle_wait,
                        &instance_id_wait,
                        LaunchStatus::Crashed {
                            message: format!("Game exited with code {}", exit_code),
                        },
                    );
                }
            }
            Err(e) => {
                emit_launch_status(
                    &app_handle_wait,
                    &instance_id_wait,
                    LaunchStatus::Crashed {
                        message: format!("Failed to wait for process: {}", e),
                    },
                );
            }
        }
    });

    Ok(pid)
}

/// Emit launch status event
fn emit_launch_status(app_handle: &AppHandle, instance_id: &str, status: LaunchStatus) {
    #[derive(serde::Serialize, Clone)]
    struct StatusEvent {
        instance_id: String,
        status: LaunchStatus,
    }

    let _ = app_handle.emit(
        "launch_status",
        StatusEvent {
            instance_id: instance_id.to_string(),
            status,
        },
    );
}

/// Emit game log event
fn emit_game_log(app_handle: &AppHandle, instance_id: &str, line: &str, level: LogLevel) {
    let log = GameLogLine {
        instance_id: instance_id.to_string(),
        line: line.to_string(),
        level,
        timestamp: Utc::now().timestamp_millis(),
    };

    let _ = app_handle.emit("game_log", log);
}

/// Find Java executable
fn find_java() -> Option<String> {
    // Check JAVA_HOME
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let java_path = std::path::PathBuf::from(&java_home)
            .join("bin")
            .join(if cfg!(windows) { "java.exe" } else { "java" });
        if java_path.exists() {
            return Some(java_path.to_string_lossy().to_string());
        }
    }

    // Check PATH
    if cfg!(windows) {
        if let Ok(output) = Command::new("where").arg("java").output() {
            if output.status.success() {
                if let Ok(path) = String::from_utf8(output.stdout) {
                    if let Some(first_line) = path.lines().next() {
                        return Some(first_line.to_string());
                    }
                }
            }
        }
    } else if let Ok(output) = Command::new("which").arg("java").output() {
        if output.status.success() {
            if let Ok(path) = String::from_utf8(output.stdout) {
                return Some(path.trim().to_string());
            }
        }
    }

    None
}
