//! Field name and user display name resolution with file-based caching.
//!
//! Provides site-scoped caches that map human-readable names to Atlassian
//! internal identifiers (customfield_ID for fields, accountId for users).
//! Caches expire after 24 hours.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::ShrugError;

const CACHE_TTL_HOURS: u64 = 24;

/// Cache for Jira custom field name → customfield_ID resolution.
pub struct FieldCache {
    base_dir: PathBuf,
}

impl FieldCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            base_dir: cache_dir.join("resolve"),
        }
    }

    /// Resolve a human-readable field name to its customfield_ID.
    ///
    /// Returns None if no cache exists, the cache is expired, or the name is not found.
    /// Matching is case-insensitive.
    pub fn resolve(&self, site: &str, human_name: &str) -> Option<String> {
        let path = cache_path(&self.base_dir, site, "fields.json");
        let entries = load_cache(&path, CACHE_TTL_HOURS)?;
        let lower = human_name.to_lowercase();
        entries
            .iter()
            .find(|(k, _)| k.to_lowercase() == lower)
            .map(|(_, v)| v.clone())
    }

    /// Populate the field cache with name → ID mappings.
    pub fn populate(&self, site: &str, fields: &[(String, String)]) -> Result<(), ShrugError> {
        let path = cache_path(&self.base_dir, site, "fields.json");
        save_cache(&path, fields)
    }
}

/// Cache for user display name → accountId resolution.
pub struct UserCache {
    base_dir: PathBuf,
}

impl UserCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            base_dir: cache_dir.join("resolve"),
        }
    }

    /// Resolve a display name to an accountId.
    ///
    /// Returns None if no cache exists, the cache is expired, or the name is not found.
    /// Matching is case-insensitive.
    pub fn resolve(&self, site: &str, display_name: &str) -> Option<String> {
        let path = cache_path(&self.base_dir, site, "users.json");
        let entries = load_cache(&path, CACHE_TTL_HOURS)?;
        let lower = display_name.to_lowercase();
        entries
            .iter()
            .find(|(k, _)| k.to_lowercase() == lower)
            .map(|(_, v)| v.clone())
    }

    /// Populate the user cache with display_name → accountId mappings.
    pub fn populate(&self, site: &str, users: &[(String, String)]) -> Result<(), ShrugError> {
        let path = cache_path(&self.base_dir, site, "users.json");
        save_cache(&path, users)
    }
}

// --- Cache utilities ---

/// Build a site-scoped cache file path.
///
/// Uses the first 16 characters of the SHA-256 hash of the site URL
/// as the directory name, avoiding path issues with URL characters.
fn cache_path(base: &Path, site: &str, filename: &str) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(site.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    let site_dir = &hash[..16.min(hash.len())];
    base.join(site_dir).join(filename)
}

/// Check if a cache file is fresh (within TTL).
fn is_cache_fresh(path: &Path, ttl_hours: u64) -> bool {
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return false,
    };

    let modified = match metadata.modified() {
        Ok(t) => t,
        Err(_) => return false,
    };

    let age = std::time::SystemTime::now()
        .duration_since(modified)
        .unwrap_or_default();

    age.as_secs() < ttl_hours * 3600
}

/// Load a cache file and return its entries if fresh.
fn load_cache(path: &Path, ttl_hours: u64) -> Option<HashMap<String, String>> {
    if !is_cache_fresh(path, ttl_hours) {
        return None;
    }

    let content = fs::read_to_string(path).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
    let entries = parsed.get("entries")?.as_object()?;

    let mut result = HashMap::new();
    for (key, value) in entries {
        if let Some(v) = value.as_str() {
            result.insert(key.clone(), v.to_string());
        }
    }

    Some(result)
}

/// Save entries to a cache file with atomic write (temp file + rename).
fn save_cache(path: &Path, entries: &[(String, String)]) -> Result<(), ShrugError> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            ShrugError::ConfigError(format!("Failed to create cache directory: {}", e))
        })?;
    }

    let mut map = serde_json::Map::new();
    for (key, value) in entries {
        map.insert(key.clone(), serde_json::Value::String(value.clone()));
    }

    let now = chrono::Utc::now().to_rfc3339();
    let cache_doc = serde_json::json!({
        "updated_at": now,
        "entries": serde_json::Value::Object(map)
    });

    let json_str = serde_json::to_string_pretty(&cache_doc)
        .map_err(|e| ShrugError::ConfigError(format!("Failed to serialise cache: {}", e)))?;

    // Atomic write: write to temp file then rename
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, &json_str)
        .map_err(|e| ShrugError::ConfigError(format!("Failed to write cache temp file: {}", e)))?;
    fs::rename(&temp_path, path)
        .map_err(|e| ShrugError::ConfigError(format!("Failed to rename cache file: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_cache_resolve_returns_none_when_no_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache = FieldCache::new(dir.path().to_path_buf());
        assert!(cache
            .resolve("test.atlassian.net", "Story Points")
            .is_none());
    }

    #[test]
    fn field_cache_populate_then_resolve() {
        let dir = tempfile::tempdir().unwrap();
        let cache = FieldCache::new(dir.path().to_path_buf());
        let site = "test.atlassian.net";

        cache
            .populate(
                site,
                &[
                    ("Story Points".to_string(), "customfield_10016".to_string()),
                    ("Sprint".to_string(), "customfield_10020".to_string()),
                ],
            )
            .unwrap();

        assert_eq!(
            cache.resolve(site, "Story Points").unwrap(),
            "customfield_10016"
        );
        assert_eq!(cache.resolve(site, "Sprint").unwrap(), "customfield_10020");
        assert!(cache.resolve(site, "NonExistent").is_none());
    }

    #[test]
    fn field_cache_resolve_is_case_insensitive() {
        let dir = tempfile::tempdir().unwrap();
        let cache = FieldCache::new(dir.path().to_path_buf());
        let site = "test.atlassian.net";

        cache
            .populate(
                site,
                &[("Story Points".to_string(), "customfield_10016".to_string())],
            )
            .unwrap();

        assert_eq!(
            cache.resolve(site, "story points").unwrap(),
            "customfield_10016"
        );
        assert_eq!(
            cache.resolve(site, "STORY POINTS").unwrap(),
            "customfield_10016"
        );
    }

    #[test]
    fn user_cache_resolve_returns_none_when_no_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache = UserCache::new(dir.path().to_path_buf());
        assert!(cache.resolve("test.atlassian.net", "Jane Smith").is_none());
    }

    #[test]
    fn user_cache_populate_then_resolve() {
        let dir = tempfile::tempdir().unwrap();
        let cache = UserCache::new(dir.path().to_path_buf());
        let site = "test.atlassian.net";

        cache
            .populate(
                site,
                &[(
                    "Jane Smith".to_string(),
                    "5b10a2844c20165700ede21g".to_string(),
                )],
            )
            .unwrap();

        assert_eq!(
            cache.resolve(site, "Jane Smith").unwrap(),
            "5b10a2844c20165700ede21g"
        );
    }

    #[test]
    fn user_cache_resolve_is_case_insensitive() {
        let dir = tempfile::tempdir().unwrap();
        let cache = UserCache::new(dir.path().to_path_buf());
        let site = "test.atlassian.net";

        cache
            .populate(
                site,
                &[(
                    "Jane Smith".to_string(),
                    "5b10a2844c20165700ede21g".to_string(),
                )],
            )
            .unwrap();

        assert_eq!(
            cache.resolve(site, "jane smith").unwrap(),
            "5b10a2844c20165700ede21g"
        );
    }

    #[test]
    fn cache_path_produces_site_scoped_paths() {
        let base = Path::new("/tmp/cache");
        let path1 = cache_path(base, "site-a.atlassian.net", "fields.json");
        let path2 = cache_path(base, "site-b.atlassian.net", "fields.json");
        assert_ne!(
            path1, path2,
            "Different sites should produce different paths"
        );
        assert!(
            path1.to_string_lossy().contains("fields.json"),
            "Should contain filename"
        );
    }

    #[test]
    fn cache_path_same_site_same_path() {
        let base = Path::new("/tmp/cache");
        let path1 = cache_path(base, "test.atlassian.net", "fields.json");
        let path2 = cache_path(base, "test.atlassian.net", "fields.json");
        assert_eq!(path1, path2, "Same site should produce same path");
    }

    #[test]
    fn save_and_load_cache_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test").join("cache.json");

        let entries = vec![
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ];

        save_cache(&path, &entries).unwrap();

        let loaded = load_cache(&path, 24).unwrap();
        assert_eq!(loaded.get("key1").unwrap(), "value1");
        assert_eq!(loaded.get("key2").unwrap(), "value2");
    }

    #[test]
    fn load_cache_returns_none_for_missing_file() {
        let result = load_cache(Path::new("/nonexistent/path/cache.json"), 24);
        assert!(result.is_none());
    }
}
