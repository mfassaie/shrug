use std::collections::HashMap;
use std::time::Duration;

use base64::Engine;
use rand::Rng;
use reqwest::blocking::Client;

use crate::auth::credentials::{AuthScheme, ResolvedCredential};
use crate::cli::OutputFormat;
use crate::cmd::router::ResolvedCommand;
use crate::error::ShrugError;
use crate::output;
use crate::quirks;
use crate::spec::analysis;
use crate::spec::model::{HttpMethod, Operation, ParameterLocation};
use crate::spec::registry::Product;

const MAX_RETRIES: u32 = 4;
const BACKOFF_BASE_SECS: f64 = 1.0;
const RETRY_AFTER_CAP_SECS: u64 = 60;
const MAX_PAGES: u32 = 1000;

/// Parsed arguments ready for request construction.
#[derive(Debug)]
pub struct ParsedArgs {
    pub path_params: HashMap<String, String>,
    pub query_params: Vec<(String, String)>,
    pub body: Option<String>,
}

/// Create an HTTP client with reasonable defaults.
pub fn create_client() -> Result<Client, ShrugError> {
    Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent(format!("shrug/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(ShrugError::NetworkError)
}

/// Parse remaining CLI args against an operation's parameter definitions.
///
/// Matches `--param-name value` pairs to the operation's declared parameters.
/// Separates path parameters from query parameters.
pub fn parse_args(
    operation: &Operation,
    remaining_args: &[String],
    json_body: Option<String>,
) -> Result<ParsedArgs, ShrugError> {
    let mut path_params = HashMap::new();
    let mut query_params = Vec::new();

    // Build lookup: normalised flag name → (original param name, location)
    let param_lookup: HashMap<String, (String, ParameterLocation)> = operation
        .parameters
        .iter()
        .map(|p| {
            let flag_name = param_to_flag_name(&p.name);
            (flag_name, (p.name.clone(), p.location.clone()))
        })
        .collect();

    // Parse --key value pairs from remaining_args
    let mut i = 0;
    while i < remaining_args.len() {
        let arg = &remaining_args[i];

        if !arg.starts_with("--") {
            return Err(ShrugError::UsageError(format!(
                "Unexpected argument '{}'. Arguments must be --flag value pairs.\n\nValid parameters:\n{}",
                arg,
                format_valid_params(operation)
            )));
        }

        let flag = &arg[2..]; // strip --

        let (original_name, location) = param_lookup.get(flag).ok_or_else(|| {
            ShrugError::UsageError(format!(
                "Unknown parameter '--{}'.\n\nValid parameters:\n{}",
                flag,
                format_valid_params(operation)
            ))
        })?;

        // Get the value
        i += 1;
        if i >= remaining_args.len() {
            return Err(ShrugError::UsageError(format!(
                "Parameter '--{}' requires a value.",
                flag
            )));
        }
        let value = remaining_args[i].clone();

        match location {
            ParameterLocation::Path => {
                path_params.insert(original_name.clone(), value);
            }
            ParameterLocation::Query => {
                query_params.push((original_name.clone(), value));
            }
            ParameterLocation::Header | ParameterLocation::Cookie => {
                // Header and cookie params are deferred (audit: can safely defer)
                tracing::debug!(
                    param = original_name,
                    "Skipping header/cookie parameter (not yet supported)"
                );
            }
        }

        i += 1;
    }

    // Validate required path params are present
    for p in &operation.parameters {
        if p.required && p.location == ParameterLocation::Path && !path_params.contains_key(&p.name)
        {
            return Err(ShrugError::UsageError(format!(
                "Required parameter '--{}' is missing.\n\nValid parameters:\n{}",
                param_to_flag_name(&p.name),
                format_valid_params(operation)
            )));
        }
    }

    // Validate required request body
    let body = json_body;
    if let Some(ref rb) = operation.request_body {
        if rb.required && body.is_none() {
            return Err(ShrugError::UsageError(
                "This operation requires a request body. Use --json '{...}' to provide it."
                    .to_string(),
            ));
        }
    }

    Ok(ParsedArgs {
        path_params,
        query_params,
        body,
    })
}

/// Resolve the effective base URL for a request.
///
/// Atlassian specs use `{baseUrl}` as a server variable, which gets stripped
/// to an empty string. When this happens, we substitute the profile's site URL.
fn resolve_base_url(
    spec_server_url: Option<&str>,
    credential: Option<&ResolvedCredential>,
) -> Option<String> {
    // If spec has a usable server URL (not empty after variable stripping), use it
    if let Some(url) = spec_server_url {
        let stripped = strip_server_variables(url);
        if !stripped.is_empty() && stripped != "/" {
            return Some(stripped);
        }
    }

    // Fall back to credential's site URL
    if let Some(cred) = credential {
        let site = &cred.site;
        if site.starts_with("http://") || site.starts_with("https://") {
            return Some(site.clone());
        }
        return Some(format!("https://{}", site));
    }

    spec_server_url.map(|s| s.to_string())
}

/// Strip `{variable}` templates from server URLs.
fn strip_server_variables(url: &str) -> String {
    let mut result = String::with_capacity(url.len());
    let mut in_var = false;
    for ch in url.chars() {
        match ch {
            '{' => in_var = true,
            '}' => in_var = false,
            _ if !in_var => result.push(ch),
            _ => {}
        }
    }
    result
}

/// Result of a single HTTP send attempt.
enum SendResult {
    /// Request succeeded. Body is None for 204 No Content.
    Success(Option<String>),
    /// Retryable error (429, 5xx, transient network). Contains retry-after hint if available.
    Retryable {
        error: ShrugError,
        retry_after: Option<u64>,
    },
    /// Non-retryable error. Return immediately.
    Fatal(ShrugError),
}

/// Send a single HTTP request and categorise the result.
/// Does NOT retry — just sends and maps the response.
fn send_request(
    client: &Client,
    method: reqwest::Method,
    url: &str,
    credential: Option<&ResolvedCredential>,
    body: Option<&str>,
    is_final_attempt: bool,
    extra_headers: &[(&str, &str)],
) -> SendResult {
    // Build request
    let mut request = client.request(method, url);
    request = apply_auth(request, credential);
    request = request.header("Accept", "application/json");
    if let Some(body_str) = body {
        request = request
            .header("Content-Type", "application/json")
            .body(body_str.to_string());
    }

    // Apply quirk headers (after defaults, so they can override if needed)
    for (key, value) in extra_headers {
        request = request.header(*key, *value);
    }

    // Send
    let response = match request.send() {
        Ok(r) => r,
        Err(e) => {
            // Classify network errors
            if is_retryable_network_error(&e) {
                return SendResult::Retryable {
                    error: ShrugError::NetworkError(e),
                    retry_after: None,
                };
            }
            return SendResult::Fatal(ShrugError::NetworkError(e));
        }
    };

    let status = response.status();

    // Success
    if status.is_success() {
        if status == reqwest::StatusCode::NO_CONTENT {
            return SendResult::Success(None);
        }
        match response.text() {
            Ok(body_text) => SendResult::Success(Some(body_text)),
            Err(e) => SendResult::Fatal(ShrugError::NetworkError(e)),
        }
    }
    // Retryable HTTP errors
    else if is_retryable_status(status.as_u16()) {
        let retry_after = parse_retry_after(&response);

        // Log body at debug level for intermediate attempts
        if !is_final_attempt {
            if let Ok(body_text) = response.text() {
                if !body_text.is_empty() {
                    tracing::debug!(status = status.as_u16(), body = %body_text, "Retryable error response");
                }
            }
            let error = map_status_to_error(status.as_u16(), String::new());
            return SendResult::Retryable { error, retry_after };
        }

        // Final attempt: include body in error
        let error_body = response.text().unwrap_or_default();
        let error = map_status_to_error(status.as_u16(), error_body);
        SendResult::Retryable { error, retry_after }
    }
    // Non-retryable HTTP errors
    else {
        let error_body = response.text().unwrap_or_default();
        SendResult::Fatal(map_status_to_error(status.as_u16(), error_body))
    }
}

/// Execute a single HTTP request with retry logic. Returns the response body.
fn execute_with_retry(
    client: &Client,
    method: reqwest::Method,
    url: &str,
    credential: Option<&ResolvedCredential>,
    body: Option<&str>,
    extra_headers: &[(&str, &str)],
) -> Result<Option<String>, ShrugError> {
    let max_attempts = MAX_RETRIES + 1;
    let mut last_error = None;

    for attempt in 0..max_attempts {
        let is_final = attempt == max_attempts - 1;

        tracing::debug!(url = %url, attempt = attempt, "Sending request");

        match send_request(
            client,
            method.clone(),
            url,
            credential,
            body,
            is_final,
            extra_headers,
        ) {
            SendResult::Success(response_body) => return Ok(response_body),
            SendResult::Fatal(err) => return Err(err),
            SendResult::Retryable { error, retry_after } => {
                if is_final {
                    tracing::warn!(
                        attempts = max_attempts,
                        "Request failed after all retry attempts"
                    );
                    return Err(error);
                }

                let delay = calculate_delay(attempt, retry_after);
                tracing::info!(
                    delay_secs = delay.as_secs_f64(),
                    attempt = attempt + 1,
                    max_retries = MAX_RETRIES,
                    "Retrying request"
                );

                std::thread::sleep(delay);
                last_error = Some(error);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        ShrugError::NetworkError(
            reqwest::blocking::Client::new()
                .get("http://invalid")
                .send()
                .unwrap_err(),
        )
    }))
}

/// Build the full URL from base components and query parameters.
fn build_full_url(
    base_url: Option<&str>,
    path: &str,
    path_params: &HashMap<String, String>,
    query_params: &[(String, String)],
) -> Result<String, ShrugError> {
    let url = analysis::build_url(base_url, path, path_params)?;
    let query_string = analysis::build_query_string(query_params);
    if query_string.is_empty() {
        Ok(url)
    } else {
        Ok(format!("{}?{}", url, query_string))
    }
}

/// Execute an API call with retry logic and optional pagination.
#[allow(clippy::too_many_arguments)]
pub fn execute(
    client: &Client,
    product: &Product,
    command: &ResolvedCommand,
    args: &ParsedArgs,
    credential: Option<&ResolvedCredential>,
    dry_run: bool,
    page_all: bool,
    limit: Option<u32>,
    format: &OutputFormat,
    is_tty: bool,
    color_enabled: bool,
    fields: Option<&[String]>,
    no_pager: bool,
) -> Result<(), ShrugError> {
    let base_url = resolve_base_url(command.server_url.as_deref(), credential);
    let full_url = build_full_url(
        base_url.as_deref(),
        &command.operation.path,
        &args.path_params,
        &args.query_params,
    )?;

    if dry_run {
        print_dry_run(
            &command.operation.method,
            &full_url,
            credential,
            args.body.as_deref(),
        );
        return Ok(());
    }

    let method = to_reqwest_method(&command.operation.method);

    // Look up quirks for this operation
    let quirk = quirks::get_quirk(product, &command.operation.operation_id);
    let extra_headers: &[(&str, &str)] = match quirk {
        Some(q) => {
            tracing::debug!(
                operation = %command.operation.operation_id,
                quirk = q.description,
                "Applying quirk"
            );
            q.extra_headers
        }
        None => &[],
    };

    // Check if pagination is needed
    let pagination = if page_all {
        analysis::detect_pagination(&command.operation)
    } else {
        None
    };

    match pagination {
        Some(style) => execute_paginated(
            client,
            &method,
            base_url.as_deref(),
            &command.operation.path,
            &args.path_params,
            &args.query_params,
            credential,
            args.body.as_deref(),
            &style,
            limit,
            extra_headers,
            format,
            is_tty,
            color_enabled,
            fields,
        ),
        None => {
            // Single request (no pagination)
            let response_body = execute_with_retry(
                client,
                method,
                &full_url,
                credential,
                args.body.as_deref(),
                extra_headers,
            )?;
            if let Some(body) = response_body {
                let formatted =
                    output::format_response(&body, format, is_tty, color_enabled, fields);
                output::print_with_pager(&formatted, !no_pager, is_tty);
            }
            Ok(())
        }
    }
}

/// Execute a paginated request, fetching all pages.
#[allow(clippy::too_many_arguments)]
fn execute_paginated(
    client: &Client,
    method: &reqwest::Method,
    base_url: Option<&str>,
    path: &str,
    path_params: &HashMap<String, String>,
    initial_query_params: &[(String, String)],
    credential: Option<&ResolvedCredential>,
    body: Option<&str>,
    style: &analysis::PaginationStyle,
    limit: Option<u32>,
    extra_headers: &[(&str, &str)],
    format: &OutputFormat,
    is_tty: bool,
    color_enabled: bool,
    fields: Option<&[String]>,
) -> Result<(), ShrugError> {
    let mut total_results: u32 = 0;
    let mut page_count: u32 = 0;

    // Mutable pagination state
    let mut offset: u64 = 0; // For Offset style
    let mut page_number: u64 = 1; // For Page style
    let mut cursor: Option<String> = None; // For Cursor style

    loop {
        page_count += 1;

        // Safety limit
        if page_count > MAX_PAGES {
            tracing::warn!(
                max_pages = MAX_PAGES,
                total_results = total_results,
                "Pagination safety limit reached"
            );
            break;
        }

        // Build query params with pagination values
        let mut query_params: Vec<(String, String)> = initial_query_params.to_vec();
        add_pagination_params(
            &mut query_params,
            style,
            offset,
            page_number,
            cursor.as_deref(),
            limit,
            total_results,
        );

        let full_url = build_full_url(base_url, path, path_params, &query_params)?;

        if page_count > 1 {
            tracing::info!(
                page = page_count,
                total_results = total_results,
                "Fetching page"
            );
        }

        let response_body = execute_with_retry(
            client,
            method.clone(),
            &full_url,
            credential,
            body,
            extra_headers,
        )?;

        let body_text = match response_body {
            Some(b) => b,
            None => break, // 204 No Content — nothing to paginate
        };

        // Print this page
        let formatted = output::format_response(&body_text, format, is_tty, color_enabled, fields);
        println!("{}", formatted);

        // Parse response to determine if more pages exist
        let json: serde_json::Value = serde_json::from_str(&body_text).unwrap_or_default();
        let page_result_count = count_results(&json);
        total_results += page_result_count;

        // Check limit
        if let Some(max) = limit {
            if total_results >= max {
                break;
            }
        }

        // Determine if there are more pages
        match style {
            analysis::PaginationStyle::Offset {
                start_param: _,
                limit_param: _,
            } => {
                let has_more = has_more_offset(&json, offset, page_result_count);
                if !has_more || page_result_count == 0 {
                    break;
                }
                offset += page_result_count as u64;
            }
            analysis::PaginationStyle::Page {
                page_param: _,
                size_param: _,
            } => {
                let has_more = has_more_page(&json);
                if !has_more || page_result_count == 0 {
                    break;
                }
                page_number += 1;
            }
            analysis::PaginationStyle::Cursor { cursor_param: _ } => {
                cursor = extract_cursor(&json);
                if cursor.is_none() {
                    break;
                }
            }
        }
    }

    tracing::info!(
        pages = page_count,
        total_results = total_results,
        "Pagination complete"
    );

    Ok(())
}

/// Add pagination-specific query parameters for the current page.
fn add_pagination_params(
    query_params: &mut Vec<(String, String)>,
    style: &analysis::PaginationStyle,
    offset: u64,
    page_number: u64,
    cursor: Option<&str>,
    limit: Option<u32>,
    total_results: u32,
) {
    // Remove any existing pagination params (from initial args) to avoid duplicates
    match style {
        analysis::PaginationStyle::Offset {
            start_param,
            limit_param,
        } => {
            query_params.retain(|(k, _)| k != start_param && k != limit_param);
            query_params.push((start_param.clone(), offset.to_string()));

            // Adjust page size if limit would be exceeded
            let page_size = if let Some(max) = limit {
                let remaining = max.saturating_sub(total_results);
                remaining.min(100) // default page size cap
            } else {
                100
            };
            query_params.push((limit_param.clone(), page_size.to_string()));
        }
        analysis::PaginationStyle::Page {
            page_param,
            size_param,
        } => {
            query_params.retain(|(k, _)| k != page_param && k != size_param);
            query_params.push((page_param.clone(), page_number.to_string()));

            let page_size = if let Some(max) = limit {
                let remaining = max.saturating_sub(total_results);
                remaining.min(100)
            } else {
                100
            };
            query_params.push((size_param.clone(), page_size.to_string()));
        }
        analysis::PaginationStyle::Cursor { cursor_param } => {
            query_params.retain(|(k, _)| k != cursor_param);
            if let Some(c) = cursor {
                query_params.push((cursor_param.clone(), c.to_string()));
            }
        }
    }
}

/// Count the number of results in a paginated JSON response.
pub fn count_results(json: &serde_json::Value) -> u32 {
    // Try known array fields: issues (Jira), values (BitBucket/Jira), results (Confluence)
    for key in &["issues", "values", "results"] {
        if let Some(arr) = json.get(key).and_then(|v| v.as_array()) {
            return arr.len() as u32;
        }
    }
    // If the response is a top-level array
    if let Some(arr) = json.as_array() {
        return arr.len() as u32;
    }
    0
}

/// Check if there are more pages for offset-based pagination.
pub fn has_more_offset(json: &serde_json::Value, current_offset: u64, page_count: u32) -> bool {
    // Check if total field exists and we haven't reached it
    if let Some(total) = json.get("total").and_then(|v| v.as_u64()) {
        return current_offset + page_count as u64 > 0
            && (current_offset + page_count as u64) < total;
    }
    // No total field — rely on whether we got results
    page_count > 0
}

/// Check if there are more pages for page-based pagination (BitBucket).
pub fn has_more_page(json: &serde_json::Value) -> bool {
    // BitBucket uses a "next" field with the URL of the next page
    json.get("next").and_then(|v| v.as_str()).is_some()
}

/// Extract cursor value for cursor-based pagination.
pub fn extract_cursor(json: &serde_json::Value) -> Option<String> {
    // Try common cursor patterns
    // Direct cursor field
    if let Some(c) = json.get("cursor").and_then(|v| v.as_str()) {
        return Some(c.to_string());
    }
    // Nested _links.next or nextPageToken
    if let Some(c) = json.get("nextPageToken").and_then(|v| v.as_str()) {
        return Some(c.to_string());
    }
    // Atlassian _links.next pattern
    if let Some(c) = json.pointer("/_links/next").and_then(|v| v.as_str()) {
        return Some(c.to_string());
    }
    None
}

/// Check if an HTTP status code is retryable.
pub fn is_retryable_status(status: u16) -> bool {
    matches!(status, 429 | 500 | 502 | 503 | 504)
}

/// Check if a network error is transient and worth retrying.
pub fn is_retryable_network_error(err: &reqwest::Error) -> bool {
    err.is_timeout() || err.is_connect()
}

/// Parse the Retry-After header from a response (integer seconds only).
fn parse_retry_after(response: &reqwest::blocking::Response) -> Option<u64> {
    response
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|secs| secs.min(RETRY_AFTER_CAP_SECS))
}

/// Calculate retry delay with exponential backoff and jitter.
pub fn calculate_delay(attempt: u32, retry_after: Option<u64>) -> Duration {
    let base_secs = match retry_after {
        Some(secs) => secs as f64,
        None => BACKOFF_BASE_SECS * 2.0_f64.powi(attempt as i32),
    };

    // Add jitter: 0 to 50% of the base delay
    let jitter = rand::thread_rng().gen_range(0.0..=(base_secs * 0.5));
    Duration::from_secs_f64(base_secs + jitter)
}

/// Map an HTTP status code to the appropriate ShrugError.
fn map_status_to_error(status: u16, error_body: String) -> ShrugError {
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
            message: format!("HTTP {}{detail}", status),
        },
    }
}

/// Print dry-run output to stderr, masking credentials.
fn print_dry_run(
    method: &HttpMethod,
    url: &str,
    credential: Option<&ResolvedCredential>,
    body: Option<&str>,
) {
    eprintln!("--- DRY RUN ---");
    eprintln!("{} {}", method, url);
    eprintln!("Accept: application/json");

    // Print auth header with masked value
    match credential {
        Some(cred) => match &cred.scheme {
            AuthScheme::Basic { .. } => eprintln!("Authorization: Basic ****"),
            AuthScheme::Bearer { .. } => eprintln!("Authorization: Bearer ****"),
        },
        None => eprintln!("Authorization: (none)"),
    }

    if let Some(body_str) = body {
        eprintln!("Content-Type: application/json");
        eprintln!();
        eprintln!("{}", body_str);
    }

    eprintln!("--- END DRY RUN ---");
}

/// Apply auth header to a request builder.
fn apply_auth(
    request: reqwest::blocking::RequestBuilder,
    credential: Option<&ResolvedCredential>,
) -> reqwest::blocking::RequestBuilder {
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
        None => {
            tracing::warn!("No credentials available. Request will be unauthenticated.");
            request
        }
    }
}

/// Convert our HttpMethod to reqwest::Method.
fn to_reqwest_method(method: &HttpMethod) -> reqwest::Method {
    match method {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Delete => reqwest::Method::DELETE,
        HttpMethod::Patch => reqwest::Method::PATCH,
    }
}

/// Convert a parameter name to its CLI flag form (kebab-case).
/// "issueIdOrKey" → "issueIdOrKey" (kept as-is, flags are matched literally)
///
/// We match flags as-is since Atlassian param names are already used directly.
fn param_to_flag_name(param_name: &str) -> String {
    param_name.to_string()
}

/// Format valid parameters for an operation (for error messages).
fn format_valid_params(operation: &Operation) -> String {
    if operation.parameters.is_empty() {
        return "  (no parameters)".to_string();
    }
    operation
        .parameters
        .iter()
        .map(|p| {
            let required = if p.required { " (required)" } else { "" };
            let loc = match p.location {
                ParameterLocation::Path => "path",
                ParameterLocation::Query => "query",
                ParameterLocation::Header => "header",
                ParameterLocation::Cookie => "cookie",
            };
            format!("  --{:<30} {}{}", p.name, loc, required)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::model::*;

    fn make_op(
        id: &str,
        method: HttpMethod,
        path: &str,
        params: Vec<Parameter>,
        request_body: Option<RequestBody>,
    ) -> Operation {
        Operation {
            operation_id: id.to_string(),
            method,
            path: path.to_string(),
            summary: None,
            description: None,
            tags: vec![],
            deprecated: false,
            parameters: params,
            request_body,
        }
    }

    fn path_param(name: &str) -> Parameter {
        Parameter {
            name: name.to_string(),
            location: ParameterLocation::Path,
            required: true,
            description: None,
            schema_type: Some("string".to_string()),
        }
    }

    fn query_param(name: &str, required: bool) -> Parameter {
        Parameter {
            name: name.to_string(),
            location: ParameterLocation::Query,
            required,
            description: None,
            schema_type: Some("string".to_string()),
        }
    }

    // === parse_args tests ===

    #[test]
    fn parse_args_path_and_query_params() {
        let op = make_op(
            "getIssue",
            HttpMethod::Get,
            "/issue/{issueIdOrKey}",
            vec![
                path_param("issueIdOrKey"),
                query_param("expand", false),
                query_param("fields", false),
            ],
            None,
        );
        let args = vec![
            "--issueIdOrKey".into(),
            "TEST-1".into(),
            "--expand".into(),
            "names".into(),
        ];
        let parsed = parse_args(&op, &args, None).unwrap();

        assert_eq!(parsed.path_params.get("issueIdOrKey").unwrap(), "TEST-1");
        assert_eq!(parsed.query_params.len(), 1);
        assert_eq!(
            parsed.query_params[0],
            ("expand".to_string(), "names".to_string())
        );
        assert!(parsed.body.is_none());
    }

    #[test]
    fn parse_args_required_path_param_missing() {
        let op = make_op(
            "getIssue",
            HttpMethod::Get,
            "/issue/{issueIdOrKey}",
            vec![path_param("issueIdOrKey")],
            None,
        );
        let result = parse_args(&op, &[], None);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(
            err.contains("issueIdOrKey"),
            "Should mention missing param: {err}"
        );
    }

    #[test]
    fn parse_args_unknown_flag_error() {
        let op = make_op(
            "getIssue",
            HttpMethod::Get,
            "/issue/{issueIdOrKey}",
            vec![path_param("issueIdOrKey")],
            None,
        );
        let args = vec![
            "--issueIdOrKey".into(),
            "TEST-1".into(),
            "--unknown".into(),
            "val".into(),
        ];
        let result = parse_args(&op, &args, None);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("Unknown parameter"), "{err}");
    }

    #[test]
    fn parse_args_json_body_captured() {
        let op = make_op(
            "createIssue",
            HttpMethod::Post,
            "/issue",
            vec![],
            Some(RequestBody {
                required: true,
                description: None,
                content_types: vec!["application/json".to_string()],
            }),
        );
        let body = r#"{"fields":{"summary":"Test"}}"#.to_string();
        let parsed = parse_args(&op, &[], Some(body.clone())).unwrap();
        assert_eq!(parsed.body.unwrap(), body);
    }

    #[test]
    fn parse_args_required_body_missing() {
        let op = make_op(
            "createIssue",
            HttpMethod::Post,
            "/issue",
            vec![],
            Some(RequestBody {
                required: true,
                description: None,
                content_types: vec!["application/json".to_string()],
            }),
        );
        let result = parse_args(&op, &[], None);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("request body"), "{err}");
    }

    #[test]
    fn parse_args_empty_args_no_required_params() {
        let op = make_op(
            "listProjects",
            HttpMethod::Get,
            "/projects",
            vec![
                query_param("startAt", false),
                query_param("maxResults", false),
            ],
            None,
        );
        let parsed = parse_args(&op, &[], None).unwrap();
        assert!(parsed.path_params.is_empty());
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn parse_args_non_flag_argument_error() {
        let op = make_op("list", HttpMethod::Get, "/items", vec![], None);
        let args = vec!["bare-arg".into()];
        let result = parse_args(&op, &args, None);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("Unexpected argument"), "{err}");
    }

    #[test]
    fn parse_args_flag_without_value_error() {
        let op = make_op(
            "getIssue",
            HttpMethod::Get,
            "/issue/{id}",
            vec![path_param("id")],
            None,
        );
        let args = vec!["--id".into()];
        let result = parse_args(&op, &args, None);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("requires a value"), "{err}");
    }

    #[test]
    fn parse_args_optional_body_not_required() {
        let op = make_op(
            "updateIssue",
            HttpMethod::Put,
            "/issue/{id}",
            vec![path_param("id")],
            Some(RequestBody {
                required: false,
                description: None,
                content_types: vec!["application/json".to_string()],
            }),
        );
        let args = vec!["--id".into(), "123".into()];
        let parsed = parse_args(&op, &args, None).unwrap();
        assert!(parsed.body.is_none());
    }

    // === resolve_base_url tests ===

    #[test]
    fn resolve_base_url_uses_spec_url_when_valid() {
        let url = resolve_base_url(Some("https://example.atlassian.net"), None);
        assert_eq!(url.unwrap(), "https://example.atlassian.net");
    }

    #[test]
    fn resolve_base_url_falls_back_to_credential_site() {
        let cred = ResolvedCredential {
            site: "mysite.atlassian.net".to_string(),
            source: crate::auth::credentials::CredentialSource::Environment,
            scheme: AuthScheme::Bearer {
                access_token: "tok".to_string(),
            },
        };
        // {baseUrl} gets stripped to empty
        let url = resolve_base_url(Some("{baseUrl}"), Some(&cred));
        assert_eq!(url.unwrap(), "https://mysite.atlassian.net");
    }

    #[test]
    fn resolve_base_url_credential_with_scheme() {
        let cred = ResolvedCredential {
            site: "https://custom.example.com".to_string(),
            source: crate::auth::credentials::CredentialSource::Environment,
            scheme: AuthScheme::Bearer {
                access_token: "tok".to_string(),
            },
        };
        let url = resolve_base_url(None, Some(&cred));
        assert_eq!(url.unwrap(), "https://custom.example.com");
    }

    #[test]
    fn resolve_base_url_no_server_no_credential() {
        let url = resolve_base_url(None, None);
        assert!(url.is_none());
    }

    // === dry_run tests ===

    #[test]
    fn dry_run_masks_basic_auth() {
        // Verify the masking logic indirectly via print_dry_run
        // (it writes to stderr, so we just ensure it doesn't panic)
        let cred = ResolvedCredential {
            site: "test.atlassian.net".to_string(),
            source: crate::auth::credentials::CredentialSource::Keychain,
            scheme: AuthScheme::Basic {
                email: "user@example.com".to_string(),
                api_token: "secret-token".to_string(),
            },
        };
        // This should not panic and should mask credentials
        print_dry_run(
            &HttpMethod::Get,
            "https://test.atlassian.net/rest/api/3/issue/TEST-1",
            Some(&cred),
            None,
        );
    }

    #[test]
    fn dry_run_masks_bearer_auth() {
        let cred = ResolvedCredential {
            site: "test.atlassian.net".to_string(),
            source: crate::auth::credentials::CredentialSource::Keychain,
            scheme: AuthScheme::Bearer {
                access_token: "eyJ-secret-token".to_string(),
            },
        };
        print_dry_run(
            &HttpMethod::Post,
            "https://test.atlassian.net/rest/api/3/issue",
            Some(&cred),
            Some(r#"{"fields":{"summary":"Test"}}"#),
        );
    }

    #[test]
    fn dry_run_no_credentials() {
        print_dry_run(
            &HttpMethod::Get,
            "https://test.atlassian.net/rest/api/3/issue/TEST-1",
            None,
            None,
        );
    }

    // === auth header tests ===

    #[test]
    fn apply_auth_basic_sets_header() {
        let client = Client::new();
        let cred = ResolvedCredential {
            site: "test.atlassian.net".to_string(),
            source: crate::auth::credentials::CredentialSource::Environment,
            scheme: AuthScheme::Basic {
                email: "user@test.com".to_string(),
                api_token: "my-token".to_string(),
            },
        };
        let request = client.get("https://example.com");
        let request = apply_auth(request, Some(&cred));
        // Build the request to inspect it
        let built = request.build().unwrap();
        let auth = built
            .headers()
            .get("Authorization")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(auth.starts_with("Basic "), "Should be Basic auth: {auth}");
        // Verify the base64 encoding
        let expected = base64::engine::general_purpose::STANDARD.encode("user@test.com:my-token");
        assert_eq!(auth, format!("Basic {}", expected));
    }

    #[test]
    fn apply_auth_bearer_sets_header() {
        let client = Client::new();
        let cred = ResolvedCredential {
            site: "test.atlassian.net".to_string(),
            source: crate::auth::credentials::CredentialSource::Environment,
            scheme: AuthScheme::Bearer {
                access_token: "my-access-token".to_string(),
            },
        };
        let request = client.get("https://example.com");
        let request = apply_auth(request, Some(&cred));
        let built = request.build().unwrap();
        let auth = built
            .headers()
            .get("Authorization")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(auth, "Bearer my-access-token");
    }

    #[test]
    fn apply_auth_none_no_header() {
        let client = Client::new();
        let request = client.get("https://example.com");
        let request = apply_auth(request, None);
        let built = request.build().unwrap();
        assert!(built.headers().get("Authorization").is_none());
    }

    // === format_valid_params tests ===

    #[test]
    fn format_valid_params_shows_all_params() {
        let op = make_op(
            "test",
            HttpMethod::Get,
            "/test/{id}",
            vec![path_param("id"), query_param("expand", false)],
            None,
        );
        let formatted = format_valid_params(&op);
        assert!(formatted.contains("--id"), "{formatted}");
        assert!(formatted.contains("path"), "{formatted}");
        assert!(formatted.contains("--expand"), "{formatted}");
        assert!(formatted.contains("query"), "{formatted}");
    }

    #[test]
    fn format_valid_params_empty_operation() {
        let op = make_op("test", HttpMethod::Get, "/test", vec![], None);
        let formatted = format_valid_params(&op);
        assert!(formatted.contains("no parameters"), "{formatted}");
    }

    // === HTTP method mapping ===

    #[test]
    fn to_reqwest_method_maps_correctly() {
        assert_eq!(to_reqwest_method(&HttpMethod::Get), reqwest::Method::GET);
        assert_eq!(to_reqwest_method(&HttpMethod::Post), reqwest::Method::POST);
        assert_eq!(to_reqwest_method(&HttpMethod::Put), reqwest::Method::PUT);
        assert_eq!(
            to_reqwest_method(&HttpMethod::Delete),
            reqwest::Method::DELETE
        );
        assert_eq!(
            to_reqwest_method(&HttpMethod::Patch),
            reqwest::Method::PATCH
        );
    }

    // === Retry logic tests ===

    #[test]
    fn is_retryable_status_429() {
        assert!(is_retryable_status(429));
    }

    #[test]
    fn is_retryable_status_500() {
        assert!(is_retryable_status(500));
    }

    #[test]
    fn is_retryable_status_502() {
        assert!(is_retryable_status(502));
    }

    #[test]
    fn is_retryable_status_503() {
        assert!(is_retryable_status(503));
    }

    #[test]
    fn is_retryable_status_504() {
        assert!(is_retryable_status(504));
    }

    #[test]
    fn is_not_retryable_400() {
        assert!(!is_retryable_status(400));
    }

    #[test]
    fn is_not_retryable_401() {
        assert!(!is_retryable_status(401));
    }

    #[test]
    fn is_not_retryable_403() {
        assert!(!is_retryable_status(403));
    }

    #[test]
    fn is_not_retryable_404() {
        assert!(!is_retryable_status(404));
    }

    #[test]
    fn is_not_retryable_201() {
        assert!(!is_retryable_status(201));
    }

    // === Backoff calculation tests ===

    #[test]
    fn calculate_delay_attempt_0_around_1s() {
        let delay = calculate_delay(0, None);
        // Base is 1s, jitter adds 0-0.5s, so delay should be 1.0-1.5s
        assert!(delay.as_secs_f64() >= 1.0, "Delay too short: {:?}", delay);
        assert!(delay.as_secs_f64() <= 1.5, "Delay too long: {:?}", delay);
    }

    #[test]
    fn calculate_delay_attempt_1_around_2s() {
        let delay = calculate_delay(1, None);
        // Base is 2s, jitter adds 0-1.0s
        assert!(delay.as_secs_f64() >= 2.0, "Delay too short: {:?}", delay);
        assert!(delay.as_secs_f64() <= 3.0, "Delay too long: {:?}", delay);
    }

    #[test]
    fn calculate_delay_attempt_2_around_4s() {
        let delay = calculate_delay(2, None);
        assert!(delay.as_secs_f64() >= 4.0, "Delay too short: {:?}", delay);
        assert!(delay.as_secs_f64() <= 6.0, "Delay too long: {:?}", delay);
    }

    #[test]
    fn calculate_delay_attempt_3_around_8s() {
        let delay = calculate_delay(3, None);
        assert!(delay.as_secs_f64() >= 8.0, "Delay too short: {:?}", delay);
        assert!(delay.as_secs_f64() <= 12.0, "Delay too long: {:?}", delay);
    }

    #[test]
    fn calculate_delay_with_retry_after() {
        let delay = calculate_delay(0, Some(5));
        // Retry-After 5s + jitter 0-2.5s
        assert!(delay.as_secs_f64() >= 5.0, "Delay too short: {:?}", delay);
        assert!(delay.as_secs_f64() <= 7.5, "Delay too long: {:?}", delay);
    }

    #[test]
    fn calculate_delay_retry_after_zero() {
        let delay = calculate_delay(0, Some(0));
        // 0s base + 0 jitter = 0s
        assert!(
            delay.as_secs_f64() <= 0.01,
            "Zero retry-after should be ~0s: {:?}",
            delay
        );
    }

    // === map_status_to_error tests ===

    #[test]
    fn map_status_400_to_usage_error() {
        let err = map_status_to_error(400, "invalid field".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Bad request"), "{msg}");
        assert!(msg.contains("invalid field"), "{msg}");
    }

    #[test]
    fn map_status_401_to_auth_error() {
        let err = map_status_to_error(401, String::new());
        let msg = format!("{}", err);
        assert!(msg.contains("Authentication failed"), "{msg}");
    }

    #[test]
    fn map_status_429_to_rate_limited() {
        let err = map_status_to_error(429, String::new());
        assert!(matches!(err, ShrugError::RateLimited { .. }));
    }

    #[test]
    fn map_status_503_to_server_error() {
        let err = map_status_to_error(503, "maintenance".to_string());
        match err {
            ShrugError::ServerError { status, message } => {
                assert_eq!(status, 503);
                assert!(message.contains("maintenance"), "{message}");
            }
            _ => panic!("Expected ServerError, got {:?}", err),
        }
    }

    #[test]
    fn map_status_empty_body() {
        let err = map_status_to_error(500, String::new());
        match err {
            ShrugError::ServerError { status, message } => {
                assert_eq!(status, 500);
                // Empty body should produce "Server error" without trailing detail
                assert_eq!(message, "Server error");
            }
            _ => panic!("Expected ServerError, got {:?}", err),
        }
    }

    // === Network error retryability ===
    // Note: Creating actual reqwest network errors in unit tests is difficult
    // because reqwest::Error constructors are private. We test the classification
    // function exists and has the right signature. Integration tests would cover
    // actual retry behaviour.

    #[test]
    fn constants_are_reasonable() {
        assert_eq!(MAX_RETRIES, 4);
        assert_eq!(RETRY_AFTER_CAP_SECS, 60);
        assert!((BACKOFF_BASE_SECS - 1.0).abs() < f64::EPSILON);
        assert_eq!(MAX_PAGES, 1000);
    }

    // === Pagination tests ===

    #[test]
    fn count_results_jira_issues() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"startAt":0,"maxResults":50,"total":150,"issues":[{"id":"1"},{"id":"2"},{"id":"3"}]}"#,
        ).unwrap();
        assert_eq!(count_results(&json), 3);
    }

    #[test]
    fn count_results_bitbucket_values() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"page":1,"pagelen":2,"values":[{"slug":"repo1"},{"slug":"repo2"}]}"#,
        )
        .unwrap();
        assert_eq!(count_results(&json), 2);
    }

    #[test]
    fn count_results_confluence_results() {
        let json: serde_json::Value =
            serde_json::from_str(r#"{"start":0,"limit":25,"results":[{"id":"page1"}]}"#).unwrap();
        assert_eq!(count_results(&json), 1);
    }

    #[test]
    fn count_results_top_level_array() {
        let json: serde_json::Value = serde_json::from_str(r#"[{"id":"1"},{"id":"2"}]"#).unwrap();
        assert_eq!(count_results(&json), 2);
    }

    #[test]
    fn count_results_empty() {
        let json: serde_json::Value = serde_json::from_str(r#"{"issues":[]}"#).unwrap();
        assert_eq!(count_results(&json), 0);
    }

    #[test]
    fn count_results_no_known_fields() {
        let json: serde_json::Value = serde_json::from_str(r#"{"data":"something"}"#).unwrap();
        assert_eq!(count_results(&json), 0);
    }

    // === Offset pagination tests ===

    #[test]
    fn has_more_offset_with_total() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"startAt":0,"maxResults":50,"total":150,"issues":[{"id":"1"}]}"#,
        )
        .unwrap();
        assert!(has_more_offset(&json, 0, 50));
    }

    #[test]
    fn has_more_offset_reached_total() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"startAt":100,"maxResults":50,"total":150,"issues":[{"id":"1"}]}"#,
        )
        .unwrap();
        assert!(!has_more_offset(&json, 100, 50));
    }

    #[test]
    fn has_more_offset_empty_results() {
        let json: serde_json::Value =
            serde_json::from_str(r#"{"startAt":150,"maxResults":50,"total":150,"issues":[]}"#)
                .unwrap();
        // page_count is 0, so has_more returns false (guard)
        assert!(!has_more_offset(&json, 150, 0));
    }

    #[test]
    fn has_more_offset_no_total_with_results() {
        let json: serde_json::Value =
            serde_json::from_str(r#"{"issues":[{"id":"1"},{"id":"2"}]}"#).unwrap();
        // No total field — rely on page_count > 0
        assert!(has_more_offset(&json, 0, 2));
    }

    #[test]
    fn has_more_offset_no_total_empty() {
        let json: serde_json::Value = serde_json::from_str(r#"{"issues":[]}"#).unwrap();
        assert!(!has_more_offset(&json, 50, 0));
    }

    // === Page pagination tests ===

    #[test]
    fn has_more_page_with_next() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"page":1,"pagelen":10,"next":"https://api.bitbucket.org/2.0/repositories?page=2","values":[{"slug":"repo"}]}"#,
        ).unwrap();
        assert!(has_more_page(&json));
    }

    #[test]
    fn has_more_page_without_next() {
        let json: serde_json::Value =
            serde_json::from_str(r#"{"page":3,"pagelen":10,"values":[{"slug":"repo"}]}"#).unwrap();
        assert!(!has_more_page(&json));
    }

    #[test]
    fn has_more_page_next_is_null() {
        let json: serde_json::Value =
            serde_json::from_str(r#"{"page":1,"next":null,"values":[]}"#).unwrap();
        assert!(!has_more_page(&json));
    }

    // === Cursor pagination tests ===

    #[test]
    fn extract_cursor_direct_field() {
        let json: serde_json::Value =
            serde_json::from_str(r#"{"cursor":"abc123","values":[{"id":"1"}]}"#).unwrap();
        assert_eq!(extract_cursor(&json), Some("abc123".to_string()));
    }

    #[test]
    fn extract_cursor_next_page_token() {
        let json: serde_json::Value =
            serde_json::from_str(r#"{"nextPageToken":"tok456","values":[{"id":"1"}]}"#).unwrap();
        assert_eq!(extract_cursor(&json), Some("tok456".to_string()));
    }

    #[test]
    fn extract_cursor_links_next() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{"_links":{"next":"/api/next?after=xyz"},"values":[{"id":"1"}]}"#,
        )
        .unwrap();
        assert_eq!(
            extract_cursor(&json),
            Some("/api/next?after=xyz".to_string())
        );
    }

    #[test]
    fn extract_cursor_none_when_absent() {
        let json: serde_json::Value = serde_json::from_str(r#"{"values":[{"id":"1"}]}"#).unwrap();
        assert_eq!(extract_cursor(&json), None);
    }

    // === add_pagination_params tests ===

    #[test]
    fn add_pagination_params_offset() {
        use crate::spec::analysis::PaginationStyle;
        let mut params = vec![("jql".to_string(), "project=TEST".to_string())];
        let style = PaginationStyle::Offset {
            start_param: "startAt".to_string(),
            limit_param: "maxResults".to_string(),
        };
        add_pagination_params(&mut params, &style, 50, 1, None, None, 50);
        assert!(params.iter().any(|(k, v)| k == "startAt" && v == "50"));
        assert!(params.iter().any(|(k, v)| k == "maxResults" && v == "100"));
        assert!(params
            .iter()
            .any(|(k, v)| k == "jql" && v == "project=TEST"));
    }

    #[test]
    fn add_pagination_params_page() {
        use crate::spec::analysis::PaginationStyle;
        let mut params = vec![];
        let style = PaginationStyle::Page {
            page_param: "page".to_string(),
            size_param: "pagelen".to_string(),
        };
        add_pagination_params(&mut params, &style, 0, 3, None, None, 200);
        assert!(params.iter().any(|(k, v)| k == "page" && v == "3"));
        assert!(params.iter().any(|(k, v)| k == "pagelen" && v == "100"));
    }

    #[test]
    fn add_pagination_params_cursor() {
        use crate::spec::analysis::PaginationStyle;
        let mut params = vec![];
        let style = PaginationStyle::Cursor {
            cursor_param: "after".to_string(),
        };
        add_pagination_params(&mut params, &style, 0, 1, Some("cursor123"), None, 0);
        assert!(params.iter().any(|(k, v)| k == "after" && v == "cursor123"));
    }

    #[test]
    fn add_pagination_params_limit_caps_page_size() {
        use crate::spec::analysis::PaginationStyle;
        let mut params = vec![];
        let style = PaginationStyle::Offset {
            start_param: "startAt".to_string(),
            limit_param: "maxResults".to_string(),
        };
        // Limit 30, already fetched 20 = only need 10 more
        add_pagination_params(&mut params, &style, 20, 1, None, Some(30), 20);
        assert!(params.iter().any(|(k, v)| k == "maxResults" && v == "10"));
    }

    #[test]
    fn add_pagination_params_removes_duplicates() {
        use crate::spec::analysis::PaginationStyle;
        let mut params = vec![
            ("startAt".to_string(), "0".to_string()),
            ("maxResults".to_string(), "50".to_string()),
            ("jql".to_string(), "project=TEST".to_string()),
        ];
        let style = PaginationStyle::Offset {
            start_param: "startAt".to_string(),
            limit_param: "maxResults".to_string(),
        };
        add_pagination_params(&mut params, &style, 100, 1, None, None, 100);
        // Should have exactly one startAt and one maxResults
        let start_count = params.iter().filter(|(k, _)| k == "startAt").count();
        let max_count = params.iter().filter(|(k, _)| k == "maxResults").count();
        assert_eq!(start_count, 1);
        assert_eq!(max_count, 1);
        // jql should be preserved
        assert!(params.iter().any(|(k, _)| k == "jql"));
    }
}
