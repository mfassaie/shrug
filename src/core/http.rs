//! HTTP client, request execution, retry logic, and error mapping.
//!
//! Provides the generic HTTP infrastructure for entity command handlers.
//! Entity modules use `execute_request` for single requests and
//! `execute_paginated` for paginated list operations.

use std::collections::HashMap;
use std::time::Duration;

use rand::Rng;
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::Method;

use crate::auth::credentials::{AuthScheme, ResolvedCredential};
use crate::core::error::ShrugError;

const MAX_RETRIES: u32 = 4;
const BACKOFF_BASE_SECS: f64 = 1.0;
const RETRY_AFTER_CAP_SECS: u64 = 60;

/// Create an HTTP client with reasonable defaults.
pub fn create_client() -> Result<Client, ShrugError> {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent(format!("shrug/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(ShrugError::NetworkError)
}

/// Build the base URL for an Atlassian API request from a credential's site.
pub fn build_base_url(credential: &ResolvedCredential) -> String {
    let site = credential.site.trim_end_matches('/');
    if site.starts_with("http://") || site.starts_with("https://") {
        site.to_string()
    } else {
        format!("https://{}", site)
    }
}

/// Execute an HTTP request with authentication and retry logic.
///
/// Returns the response body as a JSON value, or None for 204 No Content.
pub fn execute_request(
    client: &Client,
    method: Method,
    url: &str,
    credential: Option<&ResolvedCredential>,
    body: Option<&serde_json::Value>,
    extra_headers: &[(&str, &str)],
) -> Result<Option<serde_json::Value>, ShrugError> {
    let max_attempts = MAX_RETRIES + 1;

    for attempt in 0..max_attempts {
        let is_final = attempt == max_attempts - 1;

        tracing::debug!(url = %url, attempt = attempt, "Sending request");

        let mut req = client.request(method.clone(), url);

        // Apply authentication
        if let Some(cred) = credential {
            req = apply_auth(req, cred);
        }

        // Apply extra headers (e.g., CSRF bypass)
        for (key, value) in extra_headers {
            req = req.header(*key, *value);
        }

        // Apply JSON body
        if let Some(json_body) = body {
            req = req.header("Content-Type", "application/json");
            req = req.json(json_body);
        }

        match req.send() {
            Ok(response) => {
                let status = response.status().as_u16();

                if status == 204 {
                    return Ok(None);
                }

                if (200..300).contains(&status) {
                    let text = response
                        .text()
                        .map_err(ShrugError::NetworkError)?;
                    if text.is_empty() {
                        return Ok(None);
                    }
                    let json: serde_json::Value = serde_json::from_str(&text)
                        .unwrap_or_else(|_| serde_json::Value::String(text));
                    return Ok(Some(json));
                }

                if is_retryable_status(status) && !is_final {
                    let retry_after = parse_retry_after(&response);
                    let delay = calculate_delay(attempt, retry_after);
                    tracing::info!(
                        delay_secs = delay.as_secs_f64(),
                        attempt = attempt + 1,
                        status = status,
                        "Retrying request"
                    );
                    std::thread::sleep(delay);
                    continue;
                }

                let error_body = response.text().unwrap_or_default();
                return Err(map_status_to_error(status, error_body));
            }
            Err(err) => {
                if is_retryable_network_error(&err) && !is_final {
                    let delay = calculate_delay(attempt, None);
                    tracing::info!(
                        delay_secs = delay.as_secs_f64(),
                        attempt = attempt + 1,
                        error = %err,
                        "Retrying after network error"
                    );
                    std::thread::sleep(delay);
                    continue;
                }
                return Err(ShrugError::NetworkError(err));
            }
        }
    }

    Err(ShrugError::ServerError {
        status: 0,
        message: "Request failed after retries".into(),
    })
}

/// Print a dry-run representation of a request without executing it.
///
/// Writes method, URL, and optional body to stderr, then returns.
pub fn dry_run_request(method: &Method, url: &str, body: Option<&serde_json::Value>) {
    eprintln!("{} {}", method, url);
    if let Some(b) = body {
        if let Ok(pretty) = serde_json::to_string_pretty(b) {
            eprintln!("{}", pretty);
        }
    }
}

/// Execute a paginated GET request, accumulating results across pages.
///
/// For offset-based pagination (Jira, JSW): increments startAt query param.
/// For cursor-based pagination (Confluence v2): follows cursor from response.
#[allow(clippy::too_many_arguments)]
pub fn execute_paginated_get(
    client: &Client,
    url_without_query: &str,
    credential: &ResolvedCredential,
    query_params: &[(String, String)],
    extra_headers: &[(&str, &str)],
    limit: Option<u32>,
    page_size: u32,
    cursor_based: bool,
) -> Result<Vec<serde_json::Value>, ShrugError> {
    use crate::core::pagination;

    let mut all_results: Vec<serde_json::Value> = Vec::new();
    let mut offset: u64 = 0;
    let mut cursor: Option<String> = None;
    let effective_limit = limit.unwrap_or(u32::MAX) as usize;

    // For cursor-based APIs, extract the base site URL for _links.next resolution
    let site_prefix = if cursor_based {
        // Extract scheme + host from url_without_query (e.g. "https://site.atlassian.net")
        url::Url::parse(url_without_query)
            .ok()
            .map(|u| format!("{}://{}", u.scheme(), u.host_str().unwrap_or("")))
            .unwrap_or_default()
    } else {
        String::new()
    };

    loop {
        // Determine URL for this page
        let url = if cursor_based {
            if let Some(ref next_url) = cursor {
                // _links.next is a relative URL like /wiki/api/v2/pages?cursor=...
                // Resolve it against the site prefix
                if next_url.starts_with("http://") || next_url.starts_with("https://") {
                    next_url.clone()
                } else {
                    format!("{}{}", site_prefix, next_url)
                }
            } else {
                // First page: construct from base URL + query params + limit
                let mut first_params = query_params.to_vec();
                first_params.push(("limit".to_string(), page_size.to_string()));
                build_url(url_without_query, "", &std::collections::HashMap::new(), &first_params)
            }
        } else {
            let mut page_params = query_params.to_vec();
            page_params.push(("startAt".to_string(), offset.to_string()));
            page_params.push(("maxResults".to_string(), page_size.to_string()));
            build_url(url_without_query, "", &std::collections::HashMap::new(), &page_params)
        };

        let result = execute_request(client, Method::GET, &url, Some(credential), None, extra_headers)?;

        let json = match result {
            Some(v) => v,
            None => break,
        };

        let page_results = pagination::extract_results(&json)
            .cloned()
            .unwrap_or_default();
        let page_count = page_results.len() as u32;

        if page_count == 0 {
            break;
        }

        all_results.extend(page_results);

        if all_results.len() >= effective_limit {
            all_results.truncate(effective_limit);
            break;
        }

        if cursor_based {
            match pagination::extract_cursor(&json) {
                Some(c) => cursor = Some(c),
                None => break,
            }
        } else {
            offset += page_count as u64;
            if !pagination::has_more_offset(&json, offset - page_count as u64, page_count) {
                break;
            }
        }
    }

    Ok(all_results)
}

/// Execute a request that returns a simple text/empty response (e.g., DELETE, POST with no body).
pub fn execute_request_no_response(
    client: &Client,
    method: Method,
    url: &str,
    credential: Option<&ResolvedCredential>,
    body: Option<&serde_json::Value>,
    extra_headers: &[(&str, &str)],
) -> Result<(), ShrugError> {
    execute_request(client, method, url, credential, body, extra_headers)?;
    Ok(())
}

/// Build a full URL with path parameters substituted and query parameters appended.
pub fn build_url(
    base_url: &str,
    path_template: &str,
    path_params: &HashMap<String, String>,
    query_params: &[(String, String)],
) -> String {
    let mut path = path_template.to_string();
    for (key, value) in path_params {
        path = path.replace(&format!("{{{}}}", key), value);
    }

    let url = format!("{}{}", base_url.trim_end_matches('/'), path);

    if query_params.is_empty() {
        url
    } else {
        let qs: String = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(query_params.iter().map(|(k, v)| (k.as_str(), v.as_str())))
            .finish();
        format!("{}?{}", url, qs)
    }
}

/// Apply authentication to a request builder.
fn apply_auth(req: RequestBuilder, credential: &ResolvedCredential) -> RequestBuilder {
    match &credential.scheme {
        AuthScheme::Basic { email, api_token } => req.basic_auth(email, Some(api_token)),
        AuthScheme::Bearer { access_token } => req.bearer_auth(access_token),
    }
}

/// Check if an HTTP status code is retryable.
pub fn is_retryable_status(status: u16) -> bool {
    matches!(status, 429 | 500 | 502 | 503 | 504)
}

/// Check if a network error is transient and worth retrying.
pub fn is_retryable_network_error(err: &reqwest::Error) -> bool {
    err.is_timeout() || err.is_connect()
}

/// Calculate retry delay with exponential backoff and jitter.
pub fn calculate_delay(attempt: u32, retry_after: Option<u64>) -> Duration {
    let base_secs = match retry_after {
        Some(secs) => secs as f64,
        None => BACKOFF_BASE_SECS * 2.0_f64.powi(attempt as i32),
    };
    let jitter = rand::thread_rng().gen_range(0.0..=(base_secs * 0.5));
    Duration::from_secs_f64(base_secs + jitter)
}

/// Parse the Retry-After header from a response (integer seconds only).
fn parse_retry_after(response: &Response) -> Option<u64> {
    response
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|secs| secs.min(RETRY_AFTER_CAP_SECS))
}

/// Map an HTTP status code to the appropriate ShrugError.
pub fn map_status_to_error(status: u16, error_body: String) -> ShrugError {
    let detail = if error_body.is_empty() {
        String::new()
    } else {
        format!(": {}", error_body)
    };

    match status {
        400 => ShrugError::UsageError(format!("Bad request (HTTP 400){detail}")),
        401 => ShrugError::AuthError(format!("Authentication failed (HTTP 401){detail}")),
        403 => ShrugError::PermissionDenied(format!("Access denied (HTTP 403){detail}")),
        404 => ShrugError::NotFound(format!("Not found (HTTP 404){detail}")),
        429 => ShrugError::RateLimited { retry_after: None },
        500..=599 => ShrugError::ServerError {
            status,
            message: format!("Server error{detail}"),
        },
        _ => ShrugError::ServerError {
            status,
            message: format!("HTTP {status}{detail}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_retryable_status() {
        assert!(is_retryable_status(429));
        assert!(is_retryable_status(500));
        assert!(is_retryable_status(502));
        assert!(is_retryable_status(503));
        assert!(is_retryable_status(504));
        assert!(!is_retryable_status(200));
        assert!(!is_retryable_status(400));
        assert!(!is_retryable_status(401));
        assert!(!is_retryable_status(404));
    }

    #[test]
    fn test_calculate_delay_no_retry_after() {
        let delay = calculate_delay(0, None);
        assert!(delay.as_secs_f64() >= 1.0);
        assert!(delay.as_secs_f64() <= 1.5);
    }

    #[test]
    fn test_calculate_delay_with_retry_after() {
        let delay = calculate_delay(0, Some(5));
        assert!(delay.as_secs_f64() >= 5.0);
        assert!(delay.as_secs_f64() <= 7.5);
    }

    #[test]
    fn test_calculate_delay_exponential_backoff() {
        let d0 = calculate_delay(0, None);
        let d1 = calculate_delay(1, None);
        let d2 = calculate_delay(2, None);
        // Each attempt doubles the base (1, 2, 4)
        assert!(d1.as_secs_f64() > d0.as_secs_f64());
        assert!(d2.as_secs_f64() > d1.as_secs_f64());
    }

    #[test]
    fn test_map_status_to_error() {
        match map_status_to_error(401, "bad token".into()) {
            ShrugError::AuthError(msg) => assert!(msg.contains("401")),
            other => panic!("Expected AuthError, got: {:?}", other),
        }
        match map_status_to_error(404, String::new()) {
            ShrugError::NotFound(msg) => assert!(msg.contains("404")),
            other => panic!("Expected NotFound, got: {:?}", other),
        }
    }

    #[test]
    fn test_build_url_with_params() {
        let mut path_params = HashMap::new();
        path_params.insert("issueIdOrKey".into(), "TEAM-123".into());
        let url = build_url(
            "https://site.atlassian.net",
            "/rest/api/3/issue/{issueIdOrKey}",
            &path_params,
            &[],
        );
        assert_eq!(url, "https://site.atlassian.net/rest/api/3/issue/TEAM-123");
    }

    #[test]
    fn test_build_url_with_query() {
        let url = build_url(
            "https://site.atlassian.net",
            "/rest/api/3/search",
            &HashMap::new(),
            &[("maxResults".into(), "50".into()), ("startAt".into(), "0".into())],
        );
        assert!(url.contains("maxResults=50"));
        assert!(url.contains("startAt=0"));
    }

    #[test]
    fn test_build_base_url() {
        let cred = ResolvedCredential {
            site: "mysite.atlassian.net".into(),
            scheme: AuthScheme::Basic {
                email: "x@y.com".into(),
                api_token: "tok".into(),
            },
            source: crate::auth::credentials::CredentialSource::Environment,
        };
        assert_eq!(build_base_url(&cred), "https://mysite.atlassian.net");
    }

    #[test]
    fn test_build_base_url_with_scheme() {
        let cred = ResolvedCredential {
            site: "https://mysite.atlassian.net".into(),
            scheme: AuthScheme::Basic {
                email: "x@y.com".into(),
                api_token: "tok".into(),
            },
            source: crate::auth::credentials::CredentialSource::Environment,
        };
        assert_eq!(build_base_url(&cred), "https://mysite.atlassian.net");
    }
}
