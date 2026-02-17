//! Global in-memory ring buffer for recent application logs.
//!
//! Provides [`app_info!`] and [`app_error!`] macros that simultaneously write to
//! stderr/stdout (preserving existing behaviour) and store entries in a fixed-size
//! ring buffer that can be included in debug-info dumps.

use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex};

use serde::Serialize;

/// Maximum number of log entries retained.
const MAX_ENTRIES: usize = 200;

/// A single log entry.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    /// ISO-8601 UTC timestamp.
    pub timestamp: String,
    /// `"info"` or `"error"`.
    pub level: &'static str,
    /// The formatted message.
    pub message: String,
}

/// The global ring buffer.
static BUFFER: LazyLock<Mutex<VecDeque<LogEntry>>> =
    LazyLock::new(|| Mutex::new(VecDeque::with_capacity(MAX_ENTRIES)));

/// Push an entry into the ring buffer (called by the macros).
pub fn push(level: &'static str, message: String) {
    if let Ok(mut buf) = BUFFER.lock() {
        if buf.len() >= MAX_ENTRIES {
            buf.pop_front();
        }
        buf.push_back(LogEntry {
            timestamp: chrono::Utc::now()
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
            level,
            message,
        });
    }
}

/// Return a snapshot of all buffered entries (oldest first).
pub fn snapshot() -> Vec<LogEntry> {
    BUFFER
        .lock()
        .map(|buf| buf.iter().cloned().collect())
        .unwrap_or_default()
}

/// Log an informational message to stdout **and** the ring buffer.
#[macro_export]
macro_rules! app_info {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        println!("{}", msg);
        $crate::log_buffer::push("info", msg);
    }};
}

/// Log an error message to stderr **and** the ring buffer.
#[macro_export]
macro_rules! app_error {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        eprintln!("{}", msg);
        $crate::log_buffer::push("error", msg);
    }};
}
