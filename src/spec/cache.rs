use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::core::error::ShrugError;
use crate::spec::model::ApiSpec;

/// Metadata stored alongside cached specs for TTL and version tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub cached_at: DateTime<Utc>,
    pub spec_version: String,
    pub etag: Option<String>,
}

/// A cached spec entry: metadata + the parsed ApiSpec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub metadata: CacheMetadata,
    pub spec: ApiSpec,
}

/// JSON file-based cache for parsed API specs.
pub struct SpecCache {
    specs_dir: PathBuf,
}

impl SpecCache {
    /// Create a new SpecCache rooted at the given cache directory.
    /// Creates the {cache_dir}/specs/ subdirectory if it doesn't exist.
    pub fn new(cache_dir: PathBuf) -> Result<Self, ShrugError> {
        let specs_dir = cache_dir.join("specs");
        fs::create_dir_all(&specs_dir).map_err(|e| {
            ShrugError::SpecError(format!(
                "Failed to create spec cache directory '{}': {e}",
                specs_dir.display()
            ))
        })?;
        Ok(Self { specs_dir })
    }

    /// Save a parsed spec to the cache with metadata.
    /// Writes both JSON (for metadata/TTL) and binary (for fast loading).
    pub fn save(&self, product: &str, spec: &ApiSpec) -> Result<(), ShrugError> {
        self.save_with_etag(product, spec, None)
    }

    /// Save a parsed spec with an optional ETag from the HTTP response.
    /// Writes both JSON (for metadata/TTL) and binary (for fast loading).
    pub fn save_with_etag(
        &self,
        product: &str,
        spec: &ApiSpec,
        etag: Option<String>,
    ) -> Result<(), ShrugError> {
        validate_cache_key(product)?;

        let entry = CacheEntry {
            metadata: CacheMetadata {
                cached_at: Utc::now(),
                spec_version: spec.version.clone(),
                etag,
            },
            spec: spec.clone(),
        };

        self.write_json_entry(product, &entry)?;

        // Dual-write: also save binary cache for fast loading
        if let Err(e) = self.save_binary(product, spec) {
            tracing::warn!(
                product = product,
                "Failed to write binary cache (non-fatal): {e}"
            );
        }

        Ok(())
    }

    /// Load a cached spec if it exists and TTL has not expired.
    /// Tries binary cache first (fast), falls back to JSON cache.
    /// Returns None on cache miss or expiration (stale file is preserved).
    pub fn load(&self, product: &str, ttl_hours: u32) -> Result<Option<ApiSpec>, ShrugError> {
        // Check TTL via JSON metadata first
        let entry = match self.load_entry(product)? {
            Some(e) => e,
            None => return Ok(None),
        };

        let ttl = chrono::Duration::hours(i64::from(ttl_hours));
        if Utc::now() - entry.metadata.cached_at > ttl {
            return Ok(None);
        }

        // TTL is fresh — try binary cache (fast path)
        match self.load_binary(product) {
            Ok(Some(spec)) => {
                tracing::debug!(product = product, "Loaded spec from binary cache");
                return Ok(Some(spec));
            }
            Ok(None) => {
                tracing::debug!(product = product, "No binary cache, using JSON");
            }
            Err(e) => {
                tracing::debug!(product = product, "Binary cache failed, using JSON: {e}");
            }
        }

        // Fall back to JSON
        Ok(Some(entry.spec))
    }

    /// Load a cached spec regardless of TTL (serve-stale pattern).
    /// Tries binary cache first, falls back to JSON.
    /// Returns None only if no cache file exists.
    pub fn load_stale(&self, product: &str) -> Result<Option<ApiSpec>, ShrugError> {
        // Try binary first
        if let Ok(Some(spec)) = self.load_binary(product) {
            return Ok(Some(spec));
        }
        // Fall back to JSON
        Ok(self.load_entry(product)?.map(|e| e.spec))
    }

    /// Delete the cached spec for a product (both JSON and binary). Idempotent.
    pub fn invalidate(&self, product: &str) -> Result<(), ShrugError> {
        let path = self.spec_path(product);
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| ShrugError::SpecError(format!("Failed to delete cached spec: {e}")))?;
        }
        self.invalidate_binary(product)?;
        Ok(())
    }

    /// Get the stored ETag for a product's cached spec.
    /// Returns None if no cache exists or no ETag was stored.
    pub fn load_etag(&self, product: &str) -> Result<Option<String>, ShrugError> {
        Ok(self.load_entry(product)?.and_then(|e| e.metadata.etag))
    }

    /// Refresh the cache TTL without re-saving the spec.
    /// Updates cached_at to now, preserving the spec and ETag.
    pub fn touch_ttl(&self, product: &str) -> Result<(), ShrugError> {
        let mut entry = match self.load_entry(product)? {
            Some(e) => e,
            None => return Ok(()),
        };
        entry.metadata.cached_at = Utc::now();
        self.write_json_entry(product, &entry)
    }

    /// List product names that have cached spec files.
    pub fn list_cached(&self) -> Vec<String> {
        fs::read_dir(&self.specs_dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        name.strip_suffix(".json").map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Load cache metadata without the full spec (for cache list display).
    pub fn load_metadata(&self, product: &str) -> Result<Option<CacheMetadata>, ShrugError> {
        Ok(self.load_entry(product)?.map(|e| e.metadata))
    }

    /// Get the cached spec version without loading the full spec.
    pub fn cached_version(&self, product: &str) -> Result<Option<String>, ShrugError> {
        Ok(self.load_entry(product)?.map(|e| e.metadata.spec_version))
    }

    /// Check if a new spec has a different version than what's cached.
    /// Returns true if versions differ or no cache exists.
    /// Logs version changes via tracing::info.
    pub fn has_version_changed(
        &self,
        product: &str,
        new_spec: &ApiSpec,
    ) -> Result<bool, ShrugError> {
        match self.cached_version(product)? {
            Some(old_version) => {
                if old_version != new_spec.version {
                    tracing::info!(
                        product = %product,
                        old_version = %old_version,
                        new_version = %new_spec.version,
                        "Spec version changed for {product}: {old_version} → {}",
                        new_spec.version
                    );
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            None => Ok(true),
        }
    }

    /// Get the cache directory (parent of specs/).
    /// Used by background refresh to create its own SpecCache instance.
    pub fn cache_dir(&self) -> PathBuf {
        self.specs_dir
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| self.specs_dir.clone())
    }

    /// Get the specs directory path (used by tests in registry module).
    #[cfg(test)]
    pub(crate) fn specs_dir(&self) -> &PathBuf {
        &self.specs_dir
    }

    /// Save a parsed spec to binary (rkyv) cache for fast loading.
    pub fn save_binary(&self, product: &str, spec: &ApiSpec) -> Result<(), ShrugError> {
        validate_cache_key(product)?;

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(spec)
            .map_err(|e| ShrugError::SpecError(format!("Failed to serialize spec to rkyv: {e}")))?;

        let target = self.binary_path(product);
        let tmp = self
            .specs_dir
            .join(format!("{product}.rkyv.tmp.{}", std::process::id()));
        fs::write(&tmp, &bytes).map_err(|e| {
            ShrugError::SpecError(format!("Failed to write binary spec cache: {e}"))
        })?;
        fs::rename(&tmp, &target).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            ShrugError::SpecError(format!("Failed to rename binary spec cache: {e}"))
        })?;

        Ok(())
    }

    /// Load a spec from binary (rkyv) cache. Returns None if no binary cache exists.
    pub fn load_binary(&self, product: &str) -> Result<Option<ApiSpec>, ShrugError> {
        let path = self.binary_path(product);
        if !path.exists() {
            return Ok(None);
        }

        let bytes = fs::read(&path).map_err(|e| {
            ShrugError::SpecError(format!(
                "Failed to read binary spec cache '{}': {e}",
                path.display()
            ))
        })?;

        let spec = rkyv::from_bytes::<ApiSpec, rkyv::rancor::Error>(&bytes).map_err(|e| {
            tracing::warn!(
                product = product,
                "Corrupted binary spec cache, will fall back to JSON: {e}"
            );
            ShrugError::SpecError(format!(
                "Corrupted binary spec cache '{}': {e}",
                path.display()
            ))
        })?;

        Ok(Some(spec))
    }

    /// Delete binary cache for a product. Idempotent.
    pub fn invalidate_binary(&self, product: &str) -> Result<(), ShrugError> {
        let path = self.binary_path(product);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| {
                ShrugError::SpecError(format!("Failed to delete binary spec cache: {e}"))
            })?;
        }
        Ok(())
    }

    fn spec_path(&self, product: &str) -> PathBuf {
        self.specs_dir.join(format!("{product}.json"))
    }

    fn binary_path(&self, product: &str) -> PathBuf {
        self.specs_dir.join(format!("{product}.rkyv"))
    }

    fn write_json_entry(&self, product: &str, entry: &CacheEntry) -> Result<(), ShrugError> {
        let json = serde_json::to_string_pretty(entry)
            .map_err(|e| ShrugError::SpecError(format!("Failed to serialize spec cache: {e}")))?;

        let target = self.spec_path(product);
        let tmp = self
            .specs_dir
            .join(format!("{product}.json.tmp.{}", std::process::id()));
        fs::write(&tmp, &json).map_err(|e| {
            ShrugError::SpecError(format!("Failed to write spec cache temp file: {e}"))
        })?;
        fs::rename(&tmp, &target).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            ShrugError::SpecError(format!("Failed to rename spec cache file: {e}"))
        })?;

        Ok(())
    }

    fn load_entry(&self, product: &str) -> Result<Option<CacheEntry>, ShrugError> {
        let path = self.spec_path(product);
        if !path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&path).map_err(|e| {
            ShrugError::SpecError(format!(
                "Failed to read cached spec '{}': {e}",
                path.display()
            ))
        })?;

        let entry: CacheEntry = serde_json::from_str(&json).map_err(|e| {
            ShrugError::SpecError(format!("Corrupted spec cache '{}': {e}", path.display()))
        })?;

        Ok(Some(entry))
    }
}

/// Validate that a cache key doesn't contain path traversal characters.
fn validate_cache_key(key: &str) -> Result<(), ShrugError> {
    if key.contains('/') || key.contains('\\') || key.contains("..") {
        return Err(ShrugError::SpecError(format!(
            "Invalid cache key '{key}': must not contain '/', '\\', or '..'"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::model::*;
    use tempfile::TempDir;

    fn test_spec(version: &str) -> ApiSpec {
        ApiSpec {
            title: "Test API".to_string(),
            version: version.to_string(),
            server_url: Some("https://example.com".to_string()),
            tags: vec![Tag {
                name: "test".to_string(),
                description: Some("Test tag".to_string()),
            }],
            operations: vec![Operation {
                operation_id: "testOp".to_string(),
                method: HttpMethod::Get,
                path: "/test".to_string(),
                summary: Some("Test operation".to_string()),
                description: None,
                tags: vec!["test".to_string()],
                deprecated: false,
                parameters: vec![Parameter {
                    name: "id".to_string(),
                    location: ParameterLocation::Path,
                    required: true,
                    description: None,
                    schema_type: Some("string".to_string()),
                }],
                request_body: None,
            }],
        }
    }

    fn make_cache() -> (TempDir, SpecCache) {
        let tmp = TempDir::new().unwrap();
        let cache = SpecCache::new(tmp.path().to_path_buf()).unwrap();
        (tmp, cache)
    }

    #[test]
    fn save_and_load_roundtrip() {
        let (_tmp, cache) = make_cache();
        let spec = test_spec("1.0.0");

        cache.save("test-api", &spec).unwrap();
        let loaded = cache.load("test-api", 24).unwrap();

        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.title, "Test API");
        assert_eq!(loaded.version, "1.0.0");
        assert_eq!(loaded.operations.len(), 1);
        assert_eq!(loaded.operations[0].operation_id, "testOp");
    }

    #[test]
    fn load_returns_none_when_no_cache_file() {
        let (_tmp, cache) = make_cache();
        let loaded = cache.load("nonexistent", 24).unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn load_returns_none_when_ttl_expired() {
        let (_tmp, cache) = make_cache();
        let spec = test_spec("1.0.0");

        // Save, then manually overwrite with old timestamp
        cache.save("test-api", &spec).unwrap();
        let mut entry = cache.load_entry("test-api").unwrap().unwrap();
        entry.metadata.cached_at = Utc::now() - chrono::Duration::hours(25);
        let json = serde_json::to_string_pretty(&entry).unwrap();
        fs::write(cache.spec_path("test-api"), json).unwrap();

        let loaded = cache.load("test-api", 24).unwrap();
        assert!(loaded.is_none(), "Should return None for expired cache");
    }

    #[test]
    fn load_stale_returns_spec_even_when_expired() {
        let (_tmp, cache) = make_cache();
        let spec = test_spec("1.0.0");

        cache.save("test-api", &spec).unwrap();
        let mut entry = cache.load_entry("test-api").unwrap().unwrap();
        entry.metadata.cached_at = Utc::now() - chrono::Duration::hours(100);
        let json = serde_json::to_string_pretty(&entry).unwrap();
        fs::write(cache.spec_path("test-api"), json).unwrap();

        let loaded = cache.load_stale("test-api").unwrap();
        assert!(loaded.is_some(), "load_stale should ignore TTL");
        assert_eq!(loaded.unwrap().title, "Test API");
    }

    #[test]
    fn invalidate_deletes_file() {
        let (_tmp, cache) = make_cache();
        let spec = test_spec("1.0.0");

        cache.save("test-api", &spec).unwrap();
        assert!(cache.load("test-api", 24).unwrap().is_some());

        cache.invalidate("test-api").unwrap();
        assert!(cache.load("test-api", 24).unwrap().is_none());
    }

    #[test]
    fn invalidate_is_idempotent() {
        let (_tmp, cache) = make_cache();
        // Should not error even if file doesn't exist
        cache.invalidate("nonexistent").unwrap();
        cache.invalidate("nonexistent").unwrap();
    }

    #[test]
    fn list_cached_returns_correct_products() {
        let (_tmp, cache) = make_cache();
        let spec = test_spec("1.0.0");

        cache.save("jira-platform", &spec).unwrap();
        cache.save("bitbucket", &spec).unwrap();

        let mut cached = cache.list_cached();
        cached.sort();
        assert_eq!(cached, vec!["bitbucket", "jira-platform"]);
    }

    #[test]
    fn corrupted_json_returns_error() {
        let (_tmp, cache) = make_cache();
        fs::write(cache.spec_path("corrupt"), "not valid json{{{").unwrap();

        let result = cache.load("corrupt", 24);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("Corrupted"), "Should report corruption: {err}");
    }

    #[test]
    fn cached_version_returns_version() {
        let (_tmp, cache) = make_cache();
        let spec = test_spec("2.5.0");

        cache.save("test-api", &spec).unwrap();
        let version = cache.cached_version("test-api").unwrap();
        assert_eq!(version, Some("2.5.0".to_string()));
    }

    #[test]
    fn cached_version_returns_none_when_no_cache() {
        let (_tmp, cache) = make_cache();
        let version = cache.cached_version("nonexistent").unwrap();
        assert_eq!(version, None);
    }

    #[test]
    fn has_version_changed_returns_true_when_different() {
        let (_tmp, cache) = make_cache();
        let old_spec = test_spec("1.0.0");
        let new_spec = test_spec("1.1.0");

        cache.save("test-api", &old_spec).unwrap();
        let changed = cache.has_version_changed("test-api", &new_spec).unwrap();
        assert!(changed, "Should detect version change");
    }

    #[test]
    fn has_version_changed_returns_false_when_same() {
        let (_tmp, cache) = make_cache();
        let spec = test_spec("1.0.0");

        cache.save("test-api", &spec).unwrap();
        let changed = cache.has_version_changed("test-api", &spec).unwrap();
        assert!(!changed, "Same version should not report change");
    }

    #[test]
    fn has_version_changed_returns_true_when_no_cache() {
        let (_tmp, cache) = make_cache();
        let spec = test_spec("1.0.0");

        let changed = cache.has_version_changed("test-api", &spec).unwrap();
        assert!(changed, "No cache should report as changed");
    }

    #[test]
    fn save_rejects_path_traversal_keys() {
        let (_tmp, cache) = make_cache();
        let spec = test_spec("1.0.0");

        assert!(cache.save("../escape", &spec).is_err());
        assert!(cache.save("some/path", &spec).is_err());
        assert!(cache.save("some\\path", &spec).is_err());
    }

    #[test]
    fn new_returns_error_for_invalid_path() {
        // Use a path that can't be created on any OS
        let result = SpecCache::new(PathBuf::from("\0invalid\0path\0that\0cannot\0exist"));
        assert!(result.is_err());
    }

    #[test]
    fn binary_save_and_load_roundtrip() {
        let (_tmp, cache) = make_cache();
        let spec = test_spec("1.0.0");

        cache.save_binary("test-api", &spec).unwrap();
        let loaded = cache.load_binary("test-api").unwrap();

        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.title, "Test API");
        assert_eq!(loaded.version, "1.0.0");
        assert_eq!(loaded.operations.len(), 1);
        assert_eq!(loaded.operations[0].operation_id, "testOp");
    }

    #[test]
    fn binary_load_returns_none_when_no_file() {
        let (_tmp, cache) = make_cache();
        let loaded = cache.load_binary("nonexistent").unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn save_writes_both_json_and_binary() {
        let (_tmp, cache) = make_cache();
        let spec = test_spec("1.0.0");

        cache.save("test-api", &spec).unwrap();

        // Both files should exist
        assert!(
            cache.spec_path("test-api").exists(),
            "JSON cache should exist"
        );
        assert!(
            cache.binary_path("test-api").exists(),
            "Binary cache should exist"
        );
    }

    #[test]
    fn load_prefers_binary_over_json() {
        let (_tmp, cache) = make_cache();
        let spec = test_spec("1.0.0");

        // Save both formats
        cache.save("test-api", &spec).unwrap();

        // Load should succeed (uses binary internally)
        let loaded = cache.load("test-api", 24).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().title, "Test API");
    }

    #[test]
    fn load_falls_back_to_json_when_no_binary() {
        let (_tmp, cache) = make_cache();
        let spec = test_spec("1.0.0");

        // Save only JSON (bypass dual-write by writing directly)
        let entry = CacheEntry {
            metadata: CacheMetadata {
                cached_at: Utc::now(),
                spec_version: "1.0.0".to_string(),
                etag: None,
            },
            spec: spec.clone(),
        };
        let json = serde_json::to_string_pretty(&entry).unwrap();
        fs::write(cache.spec_path("test-api"), json).unwrap();

        // No binary file
        assert!(!cache.binary_path("test-api").exists());

        // Should still load via JSON fallback
        let loaded = cache.load("test-api", 24).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().title, "Test API");
    }

    #[test]
    fn invalidate_removes_both_json_and_binary() {
        let (_tmp, cache) = make_cache();
        let spec = test_spec("1.0.0");

        cache.save("test-api", &spec).unwrap();
        assert!(cache.spec_path("test-api").exists());
        assert!(cache.binary_path("test-api").exists());

        cache.invalidate("test-api").unwrap();
        assert!(!cache.spec_path("test-api").exists());
        assert!(!cache.binary_path("test-api").exists());
    }

    #[test]
    fn corrupted_binary_falls_back_to_json() {
        let (_tmp, cache) = make_cache();
        let spec = test_spec("1.0.0");

        // Save properly (both formats)
        cache.save("test-api", &spec).unwrap();

        // Corrupt the binary file
        fs::write(cache.binary_path("test-api"), b"corrupted data").unwrap();

        // load should still work via JSON fallback
        let loaded = cache.load("test-api", 24).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().title, "Test API");
    }

    #[test]
    fn save_with_etag_stores_etag() {
        let (_tmp, cache) = make_cache();
        let spec = test_spec("1.0.0");

        cache
            .save_with_etag("test-api", &spec, Some("W/\"abc123\"".to_string()))
            .unwrap();

        let etag = cache.load_etag("test-api").unwrap();
        assert_eq!(etag, Some("W/\"abc123\"".to_string()));
    }

    #[test]
    fn save_without_etag_stores_none() {
        let (_tmp, cache) = make_cache();
        let spec = test_spec("1.0.0");

        cache.save("test-api", &spec).unwrap();

        let etag = cache.load_etag("test-api").unwrap();
        assert_eq!(etag, None);
    }

    #[test]
    fn load_etag_returns_none_when_no_cache() {
        let (_tmp, cache) = make_cache();
        let etag = cache.load_etag("nonexistent").unwrap();
        assert_eq!(etag, None);
    }

    #[test]
    fn touch_ttl_refreshes_timestamp() {
        let (_tmp, cache) = make_cache();
        let spec = test_spec("1.0.0");

        // Save with old timestamp
        cache.save("test-api", &spec).unwrap();
        let mut entry = cache.load_entry("test-api").unwrap().unwrap();
        entry.metadata.cached_at = Utc::now() - chrono::Duration::hours(25);
        cache.write_json_entry("test-api", &entry).unwrap();

        // Confirm it's expired
        assert!(cache.load("test-api", 24).unwrap().is_none());

        // Touch TTL
        cache.touch_ttl("test-api").unwrap();

        // Should now be fresh
        let loaded = cache.load("test-api", 24).unwrap();
        assert!(loaded.is_some(), "touch_ttl should refresh the TTL");
    }

    #[test]
    fn touch_ttl_preserves_etag() {
        let (_tmp, cache) = make_cache();
        let spec = test_spec("1.0.0");

        cache
            .save_with_etag("test-api", &spec, Some("W/\"xyz\"".to_string()))
            .unwrap();

        cache.touch_ttl("test-api").unwrap();

        let etag = cache.load_etag("test-api").unwrap();
        assert_eq!(etag, Some("W/\"xyz\"".to_string()));
    }

    #[test]
    fn touch_ttl_is_noop_when_no_cache() {
        let (_tmp, cache) = make_cache();
        // Should not error
        cache.touch_ttl("nonexistent").unwrap();
    }
}
