use crate::error::CommandError;
use crate::services::{instance_service, launch_service};
use crate::state::AppState;
use tauri::{AppHandle, State};

/// Launch a Minecraft instance
#[tauri::command]
pub async fn launch_instance(
    instance_id: String,
    account_id: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<u32, CommandError> {
    // Check if already running
    if state.is_instance_running(&instance_id) {
        return Err(CommandError {
            code: "ALREADY_RUNNING".to_string(),
            message: "Instance is already running".to_string(),
        });
    }

    // Get the instance
    let instance = instance_service::get_instance(&state, &instance_id).map_err(CommandError::from)?;

    // Launch
    launch_service::launch_instance(&instance, &account_id, &app_handle, None)
        .await
        .map_err(CommandError::from)
}

/// Check if an instance is running
#[tauri::command]
pub fn is_instance_running(instance_id: String, state: State<'_, AppState>) -> bool {
    state.is_instance_running(&instance_id)
}

/// Get all running instances
#[tauri::command]
pub fn get_running_instances(state: State<'_, AppState>) -> Vec<String> {
    let running = state.running_instances.read();
    running.keys().cloned().collect()
}

/// Kill a running Minecraft instance
#[tauri::command]
pub fn kill_instance(instance_id: String, state: State<'_, AppState>) -> Result<(), CommandError> {
    let running = state.running_instances.read();

    if let Some(instance) = running.get(&instance_id) {
        let pid = instance.pid;
        drop(running); // Release lock before killing

        // Kill the process
        #[cfg(unix)]
        {
            use std::process::Command;
            let _ = Command::new("kill")
                .args(["-9", &pid.to_string()])
                .output();
        }

        #[cfg(windows)]
        {
            use std::process::Command;
            let _ = Command::new("taskkill")
                .args(["/F", "/PID", &pid.to_string()])
                .output();
        }

        // Unregister the instance
        state.unregister_running_instance(&instance_id);

        Ok(())
    } else {
        Err(CommandError {
            code: "NOT_RUNNING".to_string(),
            message: "Instance is not running".to_string(),
        })
    }
}
