//! API caching infrastructure to reduce redundant API calls.
//!
//! Provides generic in-memory caching with TTL-based expiration and LRU eviction,
//! plus disk caching utilities for persistent storage.

use parking_lot::RwLock;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::utils::paths::get_cache_dir;

/// A cache entry containing data and metadata.
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    /// The cached data
    pub data: T,
    /// When this entry was created
    pub created_at: Instant,
    /// Time-to-live for this entry
    pub ttl: Duration,
    /// Last access time for LRU eviction
    pub last_accessed: Instant,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry with the given TTL.
    pub fn new(data: T, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            data,
            created_at: now,
            ttl,
            last_accessed: now,
        }
    }

    /// Check if this entry has expired.
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// Thread-safe in-memory cache with TTL and LRU eviction.
pub struct Cache<K, V> {
    entries: RwLock<HashMap<K, CacheEntry<V>>>,
    default_ttl: Duration,
    max_entries: usize,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Create a new cache with the given default TTL and maximum entry count.
    pub fn new(default_ttl: Duration, max_entries: usize) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            default_ttl,
            max_entries,
        }
    }

    /// Get a value from the cache if it exists and hasn't expired.
    pub fn get(&self, key: &K) -> Option<V> {
        let mut entries = self.entries.write();

        if let Some(entry) = entries.get_mut(key) {
            if entry.is_expired() {
                entries.remove(key);
                return None;
            }
            entry.last_accessed = Instant::now();
            return Some(entry.data.clone());
        }

        None
    }

    /// Insert a value into the cache with the default TTL.
    pub fn insert(&self, key: K, value: V) {
        self.insert_with_ttl(key, value, self.default_ttl);
    }

    /// Insert a value into the cache with a custom TTL.
    pub fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) {
        let mut entries = self.entries.write();

        // Evict expired entries and LRU if needed
        if entries.len() >= self.max_entries {
            self.evict_entries(&mut entries);
        }

        entries.insert(key, CacheEntry::new(value, ttl));
    }

    /// Remove a specific entry from the cache.
    pub fn remove(&self, key: &K) -> Option<V> {
        self.entries.write().remove(key).map(|e| e.data)
    }

    /// Clear all entries from the cache.
    pub fn clear(&self) {
        self.entries.write().clear();
    }

    /// Get the current number of entries in the cache.
    pub fn len(&self) -> usize {
        self.entries.read().len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.read().is_empty()
    }

    /// Evict expired entries and LRU entries if still over capacity.
    fn evict_entries(&self, entries: &mut HashMap<K, CacheEntry<V>>) {
        // First, remove all expired entries
        entries.retain(|_, entry| !entry.is_expired());

        // If still over capacity, remove the oldest accessed entries
        while entries.len() >= self.max_entries {
            if let Some(oldest_key) = entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_accessed)
                .map(|(k, _)| k.clone())
            {
                entries.remove(&oldest_key);
            } else {
                break;
            }
        }
    }

    /// Remove all expired entries from the cache.
    pub fn cleanup_expired(&self) {
        let mut entries = self.entries.write();
        entries.retain(|_, entry| !entry.is_expired());
    }
}

impl<K, V> Default for Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        // Default to 5 minute TTL and 100 max entries
        Self::new(Duration::from_secs(300), 100)
    }
}

// ============================================================================
// Disk Cache Utilities
// ============================================================================

/// Get the API cache directory.
pub fn get_api_cache_dir() -> PathBuf {
    get_cache_dir().join("api")
}

/// Disk cache entry with timestamp for TTL checking.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct DiskCacheEntry<T> {
    /// The cached data
    pub data: T,
    /// Unix timestamp when this entry was created
    pub created_at: i64,
    /// TTL in seconds
    pub ttl_secs: u64,
}

impl<T> DiskCacheEntry<T> {
    /// Create a new disk cache entry.
    pub fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            created_at: chrono::Utc::now().timestamp(),
            ttl_secs: ttl.as_secs(),
        }
    }

    /// Check if this entry has expired.
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        (now - self.created_at) as u64 > self.ttl_secs
    }
}

/// Load a value from disk cache.
pub fn load_disk_cache<T: DeserializeOwned>(cache_name: &str, key: &str) -> Option<T> {
    let cache_path = get_api_cache_dir()
        .join(cache_name)
        .join(format!("{}.json", key));

    if !cache_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&cache_path).ok()?;
    let entry: DiskCacheEntry<T> = serde_json::from_str(&content).ok()?;

    if entry.is_expired() {
        // Clean up expired entry
        let _ = std::fs::remove_file(&cache_path);
        return None;
    }

    Some(entry.data)
}

/// Save a value to disk cache.
pub fn save_disk_cache<T: Serialize>(
    cache_name: &str,
    key: &str,
    data: &T,
    ttl: Duration,
) -> std::io::Result<()> {
    let cache_dir = get_api_cache_dir().join(cache_name);
    std::fs::create_dir_all(&cache_dir)?;

    let cache_path = cache_dir.join(format!("{}.json", key));
    let entry = DiskCacheEntry::new(data, ttl);
    let content = serde_json::to_string(&entry)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    std::fs::write(&cache_path, content)
}

/// Clear all entries in a disk cache.
pub fn clear_disk_cache(cache_name: &str) -> std::io::Result<()> {
    let cache_dir = get_api_cache_dir().join(cache_name);
    if cache_dir.exists() {
        std::fs::remove_dir_all(&cache_dir)?;
    }
    Ok(())
}

/// Clear all disk caches.
pub fn clear_all_disk_caches() -> std::io::Result<()> {
    let cache_dir = get_api_cache_dir();
    if cache_dir.exists() {
        std::fs::remove_dir_all(&cache_dir)?;
    }
    Ok(())
}

// ============================================================================
// Hash utilities for cache keys
// ============================================================================

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

/// Generate a hash key from search parameters.
pub fn hash_params<T: Hash>(params: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    params.hash(&mut hasher);
    hasher.finish()
}

/// Sanitize a string for use as a cache file name.
pub fn sanitize_cache_key(key: &str) -> String {
    key.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic() {
        let cache: Cache<String, i32> = Cache::new(Duration::from_secs(60), 10);

        cache.insert("key1".to_string(), 42);
        assert_eq!(cache.get(&"key1".to_string()), Some(42));
        assert_eq!(cache.get(&"nonexistent".to_string()), None);
    }

    #[test]
    fn test_cache_expiration() {
        let cache: Cache<String, i32> = Cache::new(Duration::from_millis(10), 10);

        cache.insert("key1".to_string(), 42);
        assert_eq!(cache.get(&"key1".to_string()), Some(42));

        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache: Cache<String, i32> = Cache::new(Duration::from_secs(60), 3);

        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);
        cache.insert("key3".to_string(), 3);

        // Access key1 to make it more recently used
        cache.get(&"key1".to_string());

        // Insert key4, should evict key2 (least recently used)
        cache.insert("key4".to_string(), 4);

        assert!(cache.get(&"key1".to_string()).is_some());
        assert!(cache.get(&"key3".to_string()).is_some());
        assert!(cache.get(&"key4".to_string()).is_some());
    }

    #[test]
    fn test_sanitize_cache_key() {
        assert_eq!(sanitize_cache_key("hello-world_123"), "hello-world_123");
        assert_eq!(sanitize_cache_key("hello/world:test"), "hello_world_test");
    }
}
