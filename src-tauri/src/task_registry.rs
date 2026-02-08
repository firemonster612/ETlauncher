use parking_lot::RwLock;
use serde::Serialize;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;

/// Type of background task
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TaskType {
    GameDownload,
    ContentInstall,
    ModpackInstall,
    InstanceUpdate,
    InstanceSetup,
    LauncherUpdate,
}

/// Simple status kind for serialization
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TaskStatusKind {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Progress data for a task
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskProgress {
    pub current: u64,
    pub total: u64,
    /// Override percentage (0-100). When set, the frontend uses this instead of current/total.
    pub percent: Option<f64>,
    pub speed_bytes_per_sec: Option<u64>,
    pub current_item: Option<String>,
    pub stage: Option<String>,
}

/// Internal tracked task (holds CancellationToken, not serializable)
pub struct TrackedTask {
    pub id: String,
    pub task_type: TaskType,
    pub label: String,
    pub status: TaskStatusKind,
    pub error: Option<String>,
    pub progress: Option<TaskProgress>,
    pub instance_id: Option<String>,
    pub created_at: i64,
    pub cancel_token: Option<CancellationToken>,
}

/// Serializable task info for frontend (no CancellationToken)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackedTaskInfo {
    pub id: String,
    pub task_type: TaskType,
    pub label: String,
    pub status: TaskStatusKind,
    pub error: Option<String>,
    pub progress: Option<TaskProgress>,
    pub instance_id: Option<String>,
    pub created_at: i64,
}

impl TrackedTask {
    fn to_info(&self) -> TrackedTaskInfo {
        TrackedTaskInfo {
            id: self.id.clone(),
            task_type: self.task_type.clone(),
            label: self.label.clone(),
            status: self.status.clone(),
            error: self.error.clone(),
            progress: self.progress.clone(),
            instance_id: self.instance_id.clone(),
            created_at: self.created_at,
        }
    }
}

/// Centralized registry for all background tasks
pub struct TaskRegistry {
    tasks: RwLock<HashMap<String, TrackedTask>>,
    app_handle: RwLock<Option<AppHandle>>,
}

impl TaskRegistry {
    pub fn new() -> Self {
        Self {
            tasks: RwLock::new(HashMap::new()),
            app_handle: RwLock::new(None),
        }
    }

    /// Store the AppHandle for event emission (called during app setup)
    pub fn set_app_handle(&self, handle: AppHandle) {
        *self.app_handle.write() = Some(handle);
    }

    /// Register a new task with Pending status
    pub fn register(
        &self,
        id: String,
        task_type: TaskType,
        label: String,
        instance_id: Option<String>,
        cancel_token: Option<CancellationToken>,
    ) -> TrackedTaskInfo {
        let task = TrackedTask {
            id: id.clone(),
            task_type,
            label,
            status: TaskStatusKind::Pending,
            error: None,
            progress: None,
            instance_id,
            created_at: chrono::Utc::now().timestamp_millis(),
            cancel_token,
        };
        let info = task.to_info();
        self.tasks.write().insert(id, task);
        self.emit_task_update(&info);
        info
    }

    /// Transition a task to Running
    pub fn start(&self, id: &str) {
        let mut tasks = self.tasks.write();
        if let Some(task) = tasks.get_mut(id) {
            task.status = TaskStatusKind::Running;
            let info = task.to_info();
            drop(tasks);
            self.emit_task_update(&info);
        }
    }

    /// Update the progress of a task
    pub fn update_progress(&self, id: &str, progress: TaskProgress) {
        let mut tasks = self.tasks.write();
        if let Some(task) = tasks.get_mut(id) {
            task.progress = Some(progress);
            let info = task.to_info();
            drop(tasks);
            self.emit_task_update(&info);
        }
    }

    /// Update just the stage string of a task
    pub fn update_stage(&self, id: &str, stage: String) {
        let mut tasks = self.tasks.write();
        if let Some(task) = tasks.get_mut(id) {
            if let Some(ref mut progress) = task.progress {
                progress.stage = Some(stage);
            } else {
                task.progress = Some(TaskProgress {
                    current: 0,
                    total: 0,
                    percent: None,
                    speed_bytes_per_sec: None,
                    current_item: None,
                    stage: Some(stage),
                });
            }
            let info = task.to_info();
            drop(tasks);
            self.emit_task_update(&info);
        }
    }

    /// Transition a task to Completed, remove cancel token
    pub fn complete(&self, id: &str) {
        let mut tasks = self.tasks.write();
        if let Some(task) = tasks.get_mut(id) {
            task.status = TaskStatusKind::Completed;
            task.cancel_token = None;
            let info = task.to_info();
            drop(tasks);
            self.emit_task_update(&info);
        }
    }

    /// Transition a task to Failed, remove cancel token
    pub fn fail(&self, id: &str, reason: String) {
        let mut tasks = self.tasks.write();
        if let Some(task) = tasks.get_mut(id) {
            task.status = TaskStatusKind::Failed;
            task.error = Some(reason);
            task.cancel_token = None;
            let info = task.to_info();
            drop(tasks);
            self.emit_task_update(&info);
        }
    }

    /// Cancel a task: trigger CancellationToken and transition to Cancelled.
    /// Returns true if the task was found.
    pub fn cancel(&self, id: &str) -> bool {
        let mut tasks = self.tasks.write();
        if let Some(task) = tasks.get_mut(id) {
            if let Some(ref token) = task.cancel_token {
                token.cancel();
            }
            task.status = TaskStatusKind::Cancelled;
            task.cancel_token = None;
            let info = task.to_info();
            drop(tasks);
            self.emit_task_update(&info);
            true
        } else {
            false
        }
    }

    /// Remove a task from the registry entirely
    pub fn remove(&self, id: &str) {
        let removed = self.tasks.write().remove(id);
        if removed.is_some() {
            self.emit_task_removed(id);
        }
    }

    /// List all tasks as serializable info
    pub fn list(&self) -> Vec<TrackedTaskInfo> {
        self.tasks.read().values().map(|t| t.to_info()).collect()
    }

    /// Get a single task as serializable info
    pub fn get(&self, id: &str) -> Option<TrackedTaskInfo> {
        self.tasks.read().get(id).map(|t| t.to_info())
    }

    /// Emit a task_update event to the frontend
    fn emit_task_update(&self, info: &TrackedTaskInfo) {
        let handle = self.app_handle.read();
        if let Some(ref handle) = *handle {
            let _ = handle.emit("task_update", info);
        }
    }

    /// Emit a task_removed event to the frontend
    fn emit_task_removed(&self, id: &str) {
        let handle = self.app_handle.read();
        if let Some(ref handle) = *handle {
            let _ = handle.emit("task_removed", id);
        }
    }
}

impl Default for TaskRegistry {
    fn default() -> Self {
        Self::new()
    }
}
