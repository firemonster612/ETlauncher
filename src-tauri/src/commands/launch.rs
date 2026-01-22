use crate::error::CommandError;
use crate::services::{instance_service, launch_service};
use crate::state::AppState;
use tauri::{AppHandle, State};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

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
    let instance =
        instance_service::get_instance(&state, &instance_id).map_err(CommandError::from)?;

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
/// First tries graceful shutdown (SIGTERM), then forces kill (SIGKILL) if needed
#[tauri::command]
pub fn kill_instance(instance_id: String, state: State<'_, AppState>) -> Result<(), CommandError> {
    let running = state.running_instances.read();

    if let Some(instance) = running.get(&instance_id) {
        let pid = instance.pid;
        drop(running); // Release lock before killing

        #[cfg(unix)]
        {
            use std::process::Command;
            use std::thread;
            use std::time::Duration;

            // First try SIGTERM (graceful shutdown)
            let _ = Command::new("kill")
                .args(["-15", &pid.to_string()])
                .output();

            // Wait up to 3 seconds for graceful exit
            for _ in 0..6 {
                thread::sleep(Duration::from_millis(500));
                // Check if process is still running
                let check = Command::new("kill").args(["-0", &pid.to_string()]).output();
                if check.is_err() || !check.unwrap().status.success() {
                    // Process has exited
                    state.unregister_running_instance(&instance_id);
                    return Ok(());
                }
            }

            // Process still running, force kill with SIGKILL
            let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
        }

        #[cfg(windows)]
        {
            use std::process::Command;
            use std::thread;
            use std::time::Duration;

            // First try graceful shutdown (sends WM_CLOSE)
            let mut cmd = Command::new("taskkill");
            cmd.args(["/PID", &pid.to_string()])
                .creation_flags(CREATE_NO_WINDOW);
            let _ = cmd.output();

            // Wait up to 3 seconds for graceful exit
            for _ in 0..6 {
                thread::sleep(Duration::from_millis(500));
                // Check if process is still running
                let mut check = Command::new("tasklist");
                check
                    .args(["/FI", &format!("PID eq {}", pid), "/NH"])
                    .creation_flags(CREATE_NO_WINDOW);
                if let Ok(output) = check.output() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    if !output_str.contains(&pid.to_string()) {
                        // Process has exited
                        state.unregister_running_instance(&instance_id);
                        return Ok(());
                    }
                }
            }

            // Process still running, force kill
            let mut cmd = Command::new("taskkill");
            cmd.args(["/F", "/PID", &pid.to_string()])
                .creation_flags(CREATE_NO_WINDOW);
            let _ = cmd.output();
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
