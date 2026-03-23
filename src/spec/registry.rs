use std::path::PathBuf;
use std::time::Duration;

use crate::error::ShrugError;
use crate::spec::cache::SpecCache;
use crate::spec::model::ApiSpec;
use crate::spec::parse_spec;

/// Supported Atlassian Cloud products.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Product {
    Jira,
    JiraSoftware,
    Confluence,
    JiraServiceManagement,
    BitBucket,
}

/// API spec format version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecFormat {
    V3,
    V2,
}

/// Metadata for an Atlassian product's API spec.
pub struct ProductInfo {
    pub product: Product,
    pub display_name: &'static str,
    pub cli_prefix: &'static str,
    pub spec_url: &'static str,
    pub spec_format: SpecFormat,
    pub cache_key: &'static str,
}

static ALL_PRODUCTS: [Product; 5] = [
    Product::Jira,
    Product::JiraSoftware,
    Product::Confluence,
    Product::JiraServiceManagement,
    Product::BitBucket,
];

static JIRA_INFO: ProductInfo = ProductInfo {
    product: Product::Jira,
    display_name: "Jira Platform",
    cli_prefix: "jira",
    spec_url: "https://dac-static.atlassian.com/cloud/jira/platform/swagger-v3.v3.json",
    spec_format: SpecFormat::V3,
    cache_key: "jira-platform",
};

static JIRA_SOFTWARE_INFO: ProductInfo = ProductInfo {
    product: Product::JiraSoftware,
    display_name: "Jira Software",
    cli_prefix: "jira-software",
    spec_url: "https://dac-static.atlassian.com/cloud/jira/software/swagger.v3.json",
    spec_format: SpecFormat::V3,
    cache_key: "jira-software",
};

static CONFLUENCE_INFO: ProductInfo = ProductInfo {
    product: Product::Confluence,
    display_name: "Confluence",
    cli_prefix: "confluence",
    spec_url: "https://dac-static.atlassian.com/cloud/confluence/openapi-v2.v3.json",
    spec_format: SpecFormat::V3,
    cache_key: "confluence",
};

static JSM_INFO: ProductInfo = ProductInfo {
    product: Product::JiraServiceManagement,
    display_name: "Jira Service Management",
    cli_prefix: "jsm",
    spec_url: "https://dac-static.atlassian.com/cloud/jira/service-desk/swagger.v3.json",
    spec_format: SpecFormat::V3,
    cache_key: "jira-service-management",
};

static BITBUCKET_INFO: ProductInfo = ProductInfo {
    product: Product::BitBucket,
    display_name: "Bitbucket",
    cli_prefix: "bitbucket",
    spec_url: "https://api.bitbucket.org/swagger.json",
    spec_format: SpecFormat::V2,
    cache_key: "bitbucket",
};

impl Product {
    /// Get the product's metadata.
    pub fn info(&self) -> &'static ProductInfo {
        match self {
            Product::Jira => &JIRA_INFO,
            Product::JiraSoftware => &JIRA_SOFTWARE_INFO,
            Product::Confluence => &CONFLUENCE_INFO,
            Product::JiraServiceManagement => &JSM_INFO,
            Product::BitBucket => &BITBUCKET_INFO,
        }
    }

    /// Look up a product by its CLI prefix.
    pub fn from_cli_prefix(prefix: &str) -> Option<Product> {
        match prefix {
            "jira" => Some(Product::Jira),
            "jira-software" => Some(Product::JiraSoftware),
            "confluence" => Some(Product::Confluence),
            "jsm" => Some(Product::JiraServiceManagement),
            "bitbucket" => Some(Product::BitBucket),
            _ => None,
        }
    }

    /// All supported products.
    pub fn all() -> &'static [Product] {
        &ALL_PRODUCTS
    }
}

/// Return the bundled fallback spec for a product (compiled into the binary).
pub fn bundled_spec(product: &Product) -> &'static str {
    match product.info().spec_format {
        SpecFormat::V3 => match product {
            Product::Jira => include_str!("bundled/jira-platform.json"),
            Product::JiraSoftware => include_str!("bundled/jira-software.json"),
            Product::Confluence => include_str!("bundled/confluence.json"),
            Product::JiraServiceManagement => {
                include_str!("bundled/jira-service-management.json")
            }
            _ => unreachable!(),
        },
        SpecFormat::V2 => include_str!("bundled/bitbucket.json"),
    }
}

/// Loads specs with tiered strategy: cache → bundled → error.
pub struct SpecLoader {
    cache: SpecCache,
    ttl_hours: u32,
}

impl SpecLoader {
    pub fn new(cache: SpecCache, ttl_hours: u32) -> Self {
        Self { cache, ttl_hours }
    }

    /// Load a spec with serve-stale pattern.
    ///
    /// 1. Fresh cache: return immediately.
    /// 2. Stale cache: return immediately, spawn background thread to refresh.
    /// 3. No cache: synchronous fetch, then bundled fallback.
    pub fn load(&self, product: &Product) -> Result<ApiSpec, ShrugError> {
        let cache_key = product.info().cache_key;

        // Tier 1: Try fresh cache (within TTL)
        if let Some(spec) = self.cache.load(cache_key, self.ttl_hours)? {
            tracing::debug!(product = cache_key, "Spec loaded from fresh cache");
            return Ok(spec);
        }

        // Tier 2: Serve stale cache + background refresh
        if let Some(spec) = self.cache.load_stale(cache_key)? {
            tracing::debug!(product = cache_key, "Serving stale cache, spawning background refresh");
            let cache_dir = self.cache.cache_dir();
            let spec_url = product.info().spec_url.to_string();
            let key = cache_key.to_string();
            let spec_format = product.info().spec_format;
            let etag = self.cache.load_etag(cache_key).unwrap_or(None);
            // Fire-and-forget background refresh
            let _ = std::thread::spawn(move || {
                background_refresh(cache_dir, spec_url, key, spec_format, etag);
            });
            return Ok(spec);
        }

        // Tier 3: No cache at all, synchronous fetch
        match self.fetch_spec(product) {
            Ok(spec) => return Ok(spec),
            Err(e) => {
                tracing::warn!(
                    product = cache_key,
                    error = %e,
                    "Network fetch failed, falling back to bundled"
                );
            }
        }

        // Tier 4: Bundled fallback
        self.load_bundled(product)
    }

    /// Load a spec preferring stale cache over bundled (serve-stale pattern).
    pub fn load_or_stale(&self, product: &Product) -> Result<ApiSpec, ShrugError> {
        let cache_key = product.info().cache_key;

        // Tier 1: Try stale cache (ignores TTL)
        if let Some(spec) = self.cache.load_stale(cache_key)? {
            tracing::debug!(product = cache_key, "Spec loaded from stale cache");
            return Ok(spec);
        }

        // Tier 2: Bundled fallback
        self.load_bundled(product)
    }

    /// Invalidate cached spec for a product.
    pub fn invalidate(&self, product: &Product) -> Result<(), ShrugError> {
        self.cache.invalidate(product.info().cache_key)
    }

    /// Check if a new spec has a different version than cached.
    pub fn check_version(&self, product: &Product, new_spec: &ApiSpec) -> Result<bool, ShrugError> {
        self.cache
            .has_version_changed(product.info().cache_key, new_spec)
    }

    /// Fetch a spec from Atlassian CDN, parse it, and save to cache with ETag.
    fn fetch_spec(&self, product: &Product) -> Result<ApiSpec, ShrugError> {
        let info = product.info();

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(format!("shrug/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(ShrugError::NetworkError)?;

        // Send conditional request if we have a stored ETag
        let mut request = client.get(info.spec_url);
        if let Ok(Some(etag)) = self.cache.load_etag(info.cache_key) {
            request = request.header("If-None-Match", &etag);
        }

        let response = request.send().map_err(ShrugError::NetworkError)?;

        let status = response.status();

        // 304 Not Modified: spec hasn't changed, just refresh TTL
        if status == reqwest::StatusCode::NOT_MODIFIED {
            tracing::debug!(product = info.cache_key, "Spec not modified (304), refreshing TTL");
            self.cache.touch_ttl(info.cache_key)?;
            // Return the cached spec
            if let Some(spec) = self.cache.load_stale(info.cache_key)? {
                return Ok(spec);
            }
            // Edge case: 304 but no cache (shouldn't happen). Fall through to error.
            return Err(ShrugError::SpecError(
                "Received 304 but no cached spec exists".to_string(),
            ));
        }

        if !status.is_success() {
            return Err(ShrugError::SpecError(format!(
                "HTTP {} fetching spec from {}",
                status, info.spec_url
            )));
        }

        // Capture ETag from response headers
        let etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let body = response
            .text()
            .map_err(|e| ShrugError::SpecError(format!("Failed to read response body: {e}")))?;

        let spec = parse_spec(&body)?;

        if let Err(e) = self.cache.save_with_etag(info.cache_key, &spec, etag) {
            tracing::warn!(
                product = info.cache_key,
                error = %e,
                "Failed to cache fetched spec (non-fatal)"
            );
        }

        tracing::debug!(
            product = info.cache_key,
            operations = spec.operations.len(),
            "Spec fetched from network"
        );

        Ok(spec)
    }

    /// Always fetch a spec from network (ignoring cache). Saves to cache on success.
    /// Prints progress to stderr for user feedback.
    pub fn refresh(&self, product: &Product) -> Result<ApiSpec, ShrugError> {
        eprintln!("Fetching {}...", product.info().display_name);
        self.fetch_spec(product)
    }

    /// Refresh all product specs from network. Returns results for each product.
    /// Prints progress to stderr for user feedback.
    pub fn refresh_all(&self) -> Vec<(Product, Result<ApiSpec, ShrugError>)> {
        Product::all()
            .iter()
            .map(|p| {
                eprintln!("Fetching {}...", p.info().display_name);
                (*p, self.fetch_spec(p))
            })
            .collect()
    }

    fn load_bundled(&self, product: &Product) -> Result<ApiSpec, ShrugError> {
        let json = bundled_spec(product);
        let spec = parse_spec(json)?;
        // Save to cache for future loads
        if let Err(e) = self.cache.save(product.info().cache_key, &spec) {
            tracing::warn!(
                product = product.info().cache_key,
                error = %e,
                "Failed to cache bundled spec (non-fatal)"
            );
        }
        tracing::debug!(
            product = product.info().cache_key,
            "Spec loaded from bundled fallback"
        );
        Ok(spec)
    }
}

/// Background spec refresh (runs in a detached thread).
/// Creates its own HTTP client and cache instance to avoid Send/Sync issues.
/// Best-effort: all errors are logged but never propagated.
fn background_refresh(
    cache_dir: PathBuf,
    spec_url: String,
    cache_key: String,
    spec_format: SpecFormat,
    etag: Option<String>,
) {
    let result = (|| -> Result<(), ShrugError> {
        let cache = SpecCache::new(cache_dir)?;

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(format!("shrug/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(ShrugError::NetworkError)?;

        let mut request = client.get(&spec_url);
        if let Some(ref etag_val) = etag {
            request = request.header("If-None-Match", etag_val);
        }

        let response = request.send().map_err(ShrugError::NetworkError)?;
        let status = response.status();

        if status == reqwest::StatusCode::NOT_MODIFIED {
            tracing::debug!(product = %cache_key, "Background refresh: 304 Not Modified, touching TTL");
            cache.touch_ttl(&cache_key)?;
            return Ok(());
        }

        if !status.is_success() {
            return Err(ShrugError::SpecError(format!(
                "Background refresh: HTTP {} from {}",
                status, spec_url
            )));
        }

        let new_etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let body = response
            .text()
            .map_err(|e| ShrugError::SpecError(format!("Failed to read response body: {e}")))?;

        // parse_spec handles both V3 and V2 internally (it tries V3 first, falls back to V2)
        let _ = spec_format; // Format is auto-detected by parse_spec
        let spec = parse_spec(&body)?;
        cache.save_with_etag(&cache_key, &spec, new_etag)?;

        tracing::debug!(
            product = %cache_key,
            operations = spec.operations.len(),
            "Background refresh: spec updated"
        );

        Ok(())
    })();

    if let Err(e) = result {
        tracing::debug!(error = %e, "Background spec refresh failed (non-fatal)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::model::*;
    use tempfile::TempDir;

    fn test_spec(title: &str, version: &str) -> ApiSpec {
        ApiSpec {
            title: title.to_string(),
            version: version.to_string(),
            server_url: Some("https://example.com".to_string()),
            tags: vec![],
            operations: vec![Operation {
                operation_id: "testOp".to_string(),
                method: HttpMethod::Get,
                path: "/test".to_string(),
                summary: None,
                description: None,
                tags: vec![],
                deprecated: false,
                parameters: vec![],
                request_body: None,
            }],
        }
    }

    fn make_loader() -> (TempDir, SpecLoader) {
        let tmp = TempDir::new().unwrap();
        let cache = SpecCache::new(tmp.path().to_path_buf()).unwrap();
        let loader = SpecLoader::new(cache, 24);
        (tmp, loader)
    }

    #[test]
    fn from_cli_prefix_maps_all_products() {
        assert_eq!(Product::from_cli_prefix("jira"), Some(Product::Jira));
        assert_eq!(
            Product::from_cli_prefix("jira-software"),
            Some(Product::JiraSoftware)
        );
        assert_eq!(
            Product::from_cli_prefix("confluence"),
            Some(Product::Confluence)
        );
        assert_eq!(
            Product::from_cli_prefix("jsm"),
            Some(Product::JiraServiceManagement)
        );
        assert_eq!(
            Product::from_cli_prefix("bitbucket"),
            Some(Product::BitBucket)
        );
    }

    #[test]
    fn from_cli_prefix_returns_none_for_unknown() {
        assert_eq!(Product::from_cli_prefix("github"), None);
        assert_eq!(Product::from_cli_prefix(""), None);
    }

    #[test]
    fn all_returns_five_products() {
        assert_eq!(Product::all().len(), 5);
    }

    #[test]
    fn product_info_has_correct_spec_urls() {
        assert!(Product::Jira
            .info()
            .spec_url
            .contains("jira/platform/swagger-v3"));
        assert!(Product::JiraSoftware
            .info()
            .spec_url
            .contains("jira/software/swagger"));
        assert!(Product::Confluence
            .info()
            .spec_url
            .contains("confluence/openapi"));
        assert!(Product::JiraServiceManagement
            .info()
            .spec_url
            .contains("service-desk/swagger"));
        assert!(Product::BitBucket.info().spec_url.contains("bitbucket.org"));
    }

    #[test]
    fn bundled_spec_is_valid_for_all_products() {
        for product in Product::all() {
            let json = bundled_spec(product);
            let result = parse_spec(json);
            assert!(
                result.is_ok(),
                "Bundled spec for {:?} should be valid: {:?}",
                product,
                result.err()
            );
        }
    }

    #[test]
    fn loader_loads_from_cache_when_available() {
        let (_tmp, loader) = make_loader();
        let spec = test_spec("Cached API", "1.0.0");

        // Pre-populate cache
        loader
            .cache
            .save(Product::Jira.info().cache_key, &spec)
            .unwrap();

        let loaded = loader.load(&Product::Jira).unwrap();
        assert_eq!(loaded.title, "Cached API");
    }

    #[test]
    fn loader_loads_spec_on_cache_miss() {
        let (_tmp, loader) = make_loader();

        // No cache — load() tries network (may succeed or fail) then falls back to bundled.
        // Either way, a spec should be returned.
        let loaded = loader.load(&Product::Jira).unwrap();
        assert!(
            !loaded.title.is_empty(),
            "Loaded spec should have a title"
        );
    }

    #[test]
    fn loader_saves_bundled_to_cache_after_loading() {
        let (_tmp, loader) = make_loader();

        // Load from bundled (caches it)
        loader.load(&Product::Jira).unwrap();

        // Should now be in cache
        let cached = loader
            .cache
            .load(Product::Jira.info().cache_key, 24)
            .unwrap();
        assert!(cached.is_some(), "Bundled spec should be cached after load");
    }

    #[test]
    fn loader_serves_stale_when_expired() {
        let (_tmp, loader) = make_loader();
        let spec = test_spec("Stale Serve API", "0.8.0");

        // Save to cache, then expire it
        loader
            .cache
            .save(Product::Jira.info().cache_key, &spec)
            .unwrap();
        let path = loader
            .cache
            .specs_dir()
            .join(format!("{}.json", Product::Jira.info().cache_key));
        let mut entry: crate::spec::cache::CacheEntry =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        entry.metadata.cached_at = chrono::Utc::now() - chrono::Duration::hours(100);
        std::fs::write(&path, serde_json::to_string_pretty(&entry).unwrap()).unwrap();

        // load() should serve the stale spec (not None, not error)
        let loaded = loader.load(&Product::Jira).unwrap();
        assert_eq!(
            loaded.title, "Stale Serve API",
            "Should serve stale cache immediately"
        );
    }

    #[test]
    fn loader_fresh_cache_returns_without_background_work() {
        let (_tmp, loader) = make_loader();
        let spec = test_spec("Fresh API", "2.0.0");

        // Save to cache (fresh)
        loader
            .cache
            .save(Product::Jira.info().cache_key, &spec)
            .unwrap();

        let loaded = loader.load(&Product::Jira).unwrap();
        assert_eq!(loaded.title, "Fresh API");
    }

    #[test]
    fn loader_load_or_stale_prefers_stale_cache() {
        let (_tmp, loader) = make_loader();
        let spec = test_spec("Stale API", "0.9.0");

        // Save with old timestamp (expired TTL)
        loader
            .cache
            .save(Product::Jira.info().cache_key, &spec)
            .unwrap();
        // Manually expire it
        let path = loader
            .cache
            .specs_dir()
            .join(format!("{}.json", Product::Jira.info().cache_key));
        let mut entry: crate::spec::cache::CacheEntry =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        entry.metadata.cached_at = chrono::Utc::now() - chrono::Duration::hours(100);
        std::fs::write(&path, serde_json::to_string_pretty(&entry).unwrap()).unwrap();

        // Regular load would miss (expired), but load_or_stale should return it
        let loaded = loader.load_or_stale(&Product::Jira).unwrap();
        assert_eq!(loaded.title, "Stale API", "Should prefer stale cache");
    }
}
