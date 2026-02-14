use crate::error::CommandError;
use crate::state::AppState;
use crate::task_registry::TrackedTaskInfo;

#[tauri::command]
pub async fn list_tasks(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<TrackedTaskInfo>, CommandError> {
    Ok(state.task_registry.list())
}

#[tauri::command]
pub async fn cancel_task(
    state: tauri::State<'_, AppState>,
    task_id: String,
) -> Result<bool, CommandError> {
    Ok(state.task_registry.cancel(&task_id))
}

#[tauri::command]
pub async fn dismiss_task(
    state: tauri::State<'_, AppState>,
    task_id: String,
) -> Result<(), CommandError> {
    state.task_registry.remove(&task_id);
    Ok(())
}
