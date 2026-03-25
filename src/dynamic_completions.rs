//! Dynamic tab-completion for Atlassian resource keys.
//!
//! Fetches project keys, space keys, and issue keys from live Atlassian
//! instances, caching results for fast interactive shell completion.

use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::auth::credentials::ResolvedCredential;
use crate::core::error::ShrugError;

/// Parse --key value pairs from args.
fn parse_flag_args(args: &[String]) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if let Some(key) = arg.strip_prefix("--") {
            i += 1;
            if i < args.len() {
                result.insert(key.to_string(), args[i].clone());
            }
        }
        i += 1;
    }
    result
}

const CACHE_TTL_SECONDS: i64 = 300; // 5 minutes

#[derive(Debug, Serialize, Deserialize)]
struct CachedCompletions {
    values: Vec<String>,
    cached_at: DateTime<Utc>,
}

/// File-based cache for completion values with short TTL.
pub struct CompletionCache {
    cache_dir: PathBuf,
}

impl CompletionCache {
    pub fn new(cache_dir: PathBuf) -> Result<Self, ShrugError> {
        let completions_dir = cache_dir.join("completions");
        fs::create_dir_all(&completions_dir).map_err(|e| {
            ShrugError::SpecError(format!("Failed to create completions cache directory: {e}"))
        })?;
        Ok(Self {
            cache_dir: completions_dir,
        })
    }

    pub fn load(&self, completion_type: &str) -> Option<Vec<String>> {
        let path = self.cache_path(completion_type);
        let json = fs::read_to_string(&path).ok()?;
        let cached: CachedCompletions = serde_json::from_str(&json).ok()?;
        let age = Utc::now() - cached.cached_at;
        if age.num_seconds() > CACHE_TTL_SECONDS {
            return None;
        }
        Some(cached.values)
    }

    pub fn save(&self, completion_type: &str, values: &[String]) {
        let cached = CachedCompletions {
            values: values.to_vec(),
            cached_at: Utc::now(),
        };
        if let Ok(json) = serde_json::to_string(&cached) {
            let _ = fs::write(self.cache_path(completion_type), json);
        }
    }

    fn cache_path(&self, completion_type: &str) -> PathBuf {
        self.cache_dir.join(format!("{completion_type}.json"))
    }
}

/// Run completion for the given type. Returns values to print (one per line).
/// Never fails — errors are silently swallowed to avoid breaking tab-completion.
pub fn complete(
    completion_type: &str,
    client: &Client,
    credential: Option<&ResolvedCredential>,
    cache: &CompletionCache,
    extra_args: &[String],
) -> Vec<String> {
    // Check cache first
    if let Some(values) = cache.load(completion_type) {
        return values;
    }

    // Fetch from API
    let result = match completion_type {
        "projects" => fetch_projects(client, credential),
        "spaces" => fetch_spaces(client, credential),
        "issues" => {
            let parsed = parse_flag_args(extra_args);
            let project = parsed.get("project").cloned().unwrap_or_default();
            if project.is_empty() {
                return Vec::new();
            }
            fetch_issues(client, credential, &project)
        }
        _ => return Vec::new(),
    };

    match result {
        Ok(values) => {
            cache.save(completion_type, &values);
            values
        }
        Err(_) => Vec::new(),
    }
}

fn resolve_base_url(credential: Option<&ResolvedCredential>) -> String {
    if let Some(cred) = credential {
        let site = &cred.site;
        if site.starts_with("http://") || site.starts_with("https://") {
            return site.clone();
        }
        return format!("https://{}", site);
    }
    "https://your-domain.atlassian.net".to_string()
}

fn apply_auth(
    request: reqwest::blocking::RequestBuilder,
    credential: Option<&ResolvedCredential>,
) -> reqwest::blocking::RequestBuilder {
    use crate::auth::credentials::AuthScheme;
    use base64::Engine;
    match credential {
        Some(cred) => match &cred.scheme {
            AuthScheme::Basic { email, api_token } => {
                let encoded = base64::engine::general_purpose::STANDARD
                    .encode(format!("{}:{}", email, api_token));
                request.header("Authorization", format!("Basic {}", encoded))
            }
            AuthScheme::Bearer { access_token } => {
                request.header("Authorization", format!("Bearer {}", access_token))
            }
        },
        None => request,
    }
}

fn fetch_projects(
    client: &Client,
    credential: Option<&ResolvedCredential>,
) -> Result<Vec<String>, ShrugError> {
    let base_url = resolve_base_url(credential);
    let url = format!("{}/rest/api/3/project?maxResults=200", base_url);

    let request = client.get(&url).header("Accept", "application/json");
    let request = apply_auth(request, credential);
    let response = request.send().map_err(ShrugError::NetworkError)?;

    if !response.status().is_success() {
        return Err(ShrugError::SpecError("Failed to fetch projects".into()));
    }

    let body: serde_json::Value = response
        .json()
        .map_err(|e| ShrugError::SpecError(format!("Failed to parse projects: {e}")))?;

    let mut keys: Vec<String> = body
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|p| p.get("key").and_then(|k| k.as_str()).map(String::from))
        .collect();
    keys.sort();
    Ok(keys)
}

fn fetch_spaces(
    client: &Client,
    credential: Option<&ResolvedCredential>,
) -> Result<Vec<String>, ShrugError> {
    let base_url = resolve_base_url(credential);
    let url = format!("{}/wiki/api/v2/spaces?limit=250", base_url);

    let request = client.get(&url).header("Accept", "application/json");
    let request = apply_auth(request, credential);
    let response = request.send().map_err(ShrugError::NetworkError)?;

    if !response.status().is_success() {
        return Err(ShrugError::SpecError("Failed to fetch spaces".into()));
    }

    let body: serde_json::Value = response
        .json()
        .map_err(|e| ShrugError::SpecError(format!("Failed to parse spaces: {e}")))?;

    let mut keys: Vec<String> = body
        .get("results")
        .and_then(|r| r.as_array())
        .into_iter()
        .flatten()
        .filter_map(|s| s.get("key").and_then(|k| k.as_str()).map(String::from))
        .collect();
    keys.sort();
    Ok(keys)
}

fn fetch_issues(
    client: &Client,
    credential: Option<&ResolvedCredential>,
    project: &str,
) -> Result<Vec<String>, ShrugError> {
    let base_url = resolve_base_url(credential);
    let url = format!(
        "{}/rest/api/3/search?jql=project%3D{}&fields=key&maxResults=50",
        base_url, project
    );

    let request = client.get(&url).header("Accept", "application/json");
    let request = apply_auth(request, credential);
    let response = request.send().map_err(ShrugError::NetworkError)?;

    if !response.status().is_success() {
        return Err(ShrugError::SpecError("Failed to fetch issues".into()));
    }

    let body: serde_json::Value = response
        .json()
        .map_err(|e| ShrugError::SpecError(format!("Failed to parse issues: {e}")))?;

    let mut keys: Vec<String> = body
        .get("issues")
        .and_then(|r| r.as_array())
        .into_iter()
        .flatten()
        .filter_map(|i| i.get("key").and_then(|k| k.as_str()).map(String::from))
        .collect();
    keys.sort();
    Ok(keys)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_cache() -> (TempDir, CompletionCache) {
        let tmp = TempDir::new().unwrap();
        let cache = CompletionCache::new(tmp.path().to_path_buf()).unwrap();
        (tmp, cache)
    }

    #[test]
    fn cache_save_and_load() {
        let (_tmp, cache) = make_cache();
        let values = vec!["FOO".to_string(), "BAR".to_string()];
        cache.save("projects", &values);
        let loaded = cache.load("projects").unwrap();
        assert_eq!(loaded, values);
    }

    #[test]
    fn cache_returns_none_when_missing() {
        let (_tmp, cache) = make_cache();
        assert!(cache.load("nonexistent").is_none());
    }

    #[test]
    fn cache_returns_none_when_expired() {
        let (_tmp, cache) = make_cache();
        let cached = CachedCompletions {
            values: vec!["OLD".to_string()],
            cached_at: Utc::now() - chrono::Duration::seconds(CACHE_TTL_SECONDS + 10),
        };
        let json = serde_json::to_string(&cached).unwrap();
        fs::write(cache.cache_path("projects"), json).unwrap();
        assert!(cache.load("projects").is_none());
    }

    #[test]
    fn cache_returns_values_within_ttl() {
        let (_tmp, cache) = make_cache();
        let cached = CachedCompletions {
            values: vec!["FRESH".to_string()],
            cached_at: Utc::now() - chrono::Duration::seconds(CACHE_TTL_SECONDS - 10),
        };
        let json = serde_json::to_string(&cached).unwrap();
        fs::write(cache.cache_path("projects"), json).unwrap();
        let loaded = cache.load("projects").unwrap();
        assert_eq!(loaded, vec!["FRESH".to_string()]);
    }

    #[test]
    fn complete_returns_empty_for_unknown_type() {
        let (_tmp, cache) = make_cache();
        let client = Client::new();
        let result = complete("unknown", &client, None, &cache, &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn complete_uses_cached_values() {
        let (_tmp, cache) = make_cache();
        let values = vec!["CACHED".to_string()];
        cache.save("projects", &values);
        let client = Client::new();
        let result = complete("projects", &client, None, &cache, &[]);
        assert_eq!(result, vec!["CACHED".to_string()]);
    }
}
