use std::collections::HashMap;

use base64::Engine;
use reqwest::blocking::Client;

use crate::auth::credentials::{AuthScheme, ResolvedCredential};
use crate::cmd::router::ResolvedCommand;
use crate::error::ShrugError;
use crate::spec::analysis;
use crate::spec::model::{HttpMethod, Operation, ParameterLocation};

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

/// Execute an API call (or dry-run it).
pub fn execute(
    client: &Client,
    command: &ResolvedCommand,
    args: &ParsedArgs,
    credential: Option<&ResolvedCredential>,
    dry_run: bool,
) -> Result<(), ShrugError> {
    let base_url = resolve_base_url(command.server_url.as_deref(), credential);

    // Build URL
    let url = analysis::build_url(base_url.as_deref(), &command.operation.path, &args.path_params)?;

    let query_string = analysis::build_query_string(&args.query_params);
    let full_url = if query_string.is_empty() {
        url
    } else {
        format!("{}?{}", url, query_string)
    };

    if dry_run {
        print_dry_run(
            &command.operation.method,
            &full_url,
            credential,
            args.body.as_deref(),
        );
        return Ok(());
    }

    // Build request
    let method = to_reqwest_method(&command.operation.method);
    let mut request = client.request(method, &full_url);

    // Auth header
    request = apply_auth(request, credential);

    // Headers
    request = request.header("Accept", "application/json");

    // Body
    if let Some(ref body) = args.body {
        request = request
            .header("Content-Type", "application/json")
            .body(body.clone());
    }

    // Send
    tracing::debug!(url = %full_url, method = %command.operation.method, "Sending request");
    let response = request.send()?;
    let status = response.status();

    // Handle response
    if status.is_success() {
        if status == reqwest::StatusCode::NO_CONTENT {
            // 204: succeed silently
            return Ok(());
        }
        let body = response.text()?;
        println!("{}", body);
        return Ok(());
    }

    // Error responses: read body for diagnostics
    let error_body = response.text().unwrap_or_default();
    let detail = if error_body.is_empty() {
        String::new()
    } else {
        format!(": {}", error_body)
    };

    match status.as_u16() {
        400 => Err(ShrugError::UsageError(format!(
            "Bad request (HTTP 400){detail}"
        ))),
        401 => Err(ShrugError::AuthError(format!(
            "Authentication failed (HTTP 401){detail}"
        ))),
        403 => Err(ShrugError::PermissionDenied(format!(
            "Access denied (HTTP 403){detail}"
        ))),
        404 => Err(ShrugError::NotFound(format!(
            "Not found (HTTP 404){detail}"
        ))),
        429 => {
            // Try to parse Retry-After header from the error body context
            // Note: we already consumed the response, so we rely on what Atlassian returns
            Err(ShrugError::RateLimited { retry_after: None })
        }
        500..=599 => Err(ShrugError::ServerError {
            status: status.as_u16(),
            message: format!("Server error{detail}"),
        }),
        _ => Err(ShrugError::ServerError {
            status: status.as_u16(),
            message: format!("HTTP {}{detail}", status.as_u16()),
        }),
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
        assert_eq!(parsed.query_params[0], ("expand".to_string(), "names".to_string()));
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
        assert!(err.contains("issueIdOrKey"), "Should mention missing param: {err}");
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
        let args = vec!["--issueIdOrKey".into(), "TEST-1".into(), "--unknown".into(), "val".into()];
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
            vec![query_param("startAt", false), query_param("maxResults", false)],
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
        let auth = built.headers().get("Authorization").unwrap().to_str().unwrap();
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
        let auth = built.headers().get("Authorization").unwrap().to_str().unwrap();
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
            vec![
                path_param("id"),
                query_param("expand", false),
            ],
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
        assert_eq!(to_reqwest_method(&HttpMethod::Delete), reqwest::Method::DELETE);
        assert_eq!(to_reqwest_method(&HttpMethod::Patch), reqwest::Method::PATCH);
    }
}
