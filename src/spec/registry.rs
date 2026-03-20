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
    spec_url: "https://dac-static.atlassian.com/cloud/confluence/swagger.v3.json",
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

    /// Load a spec: try cache first, then bundled fallback.
    pub fn load(&self, product: &Product) -> Result<ApiSpec, ShrugError> {
        let cache_key = product.info().cache_key;

        // Tier 1: Try cache
        if let Some(spec) = self.cache.load(cache_key, self.ttl_hours)? {
            tracing::debug!(product = cache_key, "Spec loaded from cache");
            return Ok(spec);
        }

        // Tier 2: Bundled fallback
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
            .contains("confluence/swagger"));
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
    fn loader_falls_back_to_bundled_on_cache_miss() {
        let (_tmp, loader) = make_loader();

        // No cache — should load bundled
        let loaded = loader.load(&Product::Jira).unwrap();
        assert_eq!(loaded.title, "Jira Platform");
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
