//! Helper commands (+create, +search, +transition) for common Jira operations.
//!
//! Helper commands are UX shortcuts prefixed with `+` that build the correct
//! API calls from simple flags. They bypass the normal dynamic routing pipeline
//! and make HTTP requests directly via the reqwest client.

use std::collections::HashMap;

use base64::Engine;
use reqwest::blocking::Client;

use crate::auth::credentials::{AuthScheme, ResolvedCredential};
use crate::cli::OutputFormat;
use crate::error::ShrugError;
use crate::jql::JqlShorthand;
use crate::markdown_to_adf;
use crate::output;
use crate::spec::analysis;
use crate::spec::model::{ApiSpec, Operation};
use crate::spec::registry::Product;

/// Available helper commands.
const AVAILABLE_HELPERS: &[&str] = &["+create", "+search", "+transition"];

/// Check if the first arg is a helper command (starts with "+").
pub fn is_helper_command(args: &[String]) -> bool {
    args.first().map(|a| a.starts_with('+')).unwrap_or(false)
}

/// Dispatch a helper command by name.
#[allow(clippy::too_many_arguments)]
pub fn dispatch_helper(
    helper_name: &str,
    product: &Product,
    remaining_args: &[String],
    spec: &ApiSpec,
    client: &Client,
    credential: Option<&ResolvedCredential>,
    jql_shorthand: &JqlShorthand,
    raw_jql: Option<&str>,
    format: &OutputFormat,
    is_tty: bool,
    color_enabled: bool,
    fields: Option<&[String]>,
    no_pager: bool,
    dry_run: bool,
) -> Result<(), ShrugError> {
    // Product validation: helpers are only available for Jira and Jira Software
    if !matches!(product, Product::Jira | Product::JiraSoftware) {
        return Err(ShrugError::UsageError(
            "Helper commands are only available for Jira and Jira Software.".to_string(),
        ));
    }

    match helper_name {
        "create" => helper_create(remaining_args, spec, client, credential, dry_run),
        "search" => helper_search(
            remaining_args,
            spec,
            client,
            credential,
            jql_shorthand,
            raw_jql,
            format,
            is_tty,
            color_enabled,
            fields,
            no_pager,
            dry_run,
        ),
        "transition" => helper_transition(remaining_args, spec, client, credential, dry_run),
        _ => Err(ShrugError::UsageError(format!(
            "Unknown helper command '+{}'.\n\nAvailable helpers:\n{}",
            helper_name,
            AVAILABLE_HELPERS
                .iter()
                .map(|h| format!("  {h}"))
                .collect::<Vec<_>>()
                .join("\n")
        ))),
    }
}

// --- Helper implementations ---

fn helper_create(
    args: &[String],
    spec: &ApiSpec,
    client: &Client,
    credential: Option<&ResolvedCredential>,
    dry_run: bool,
) -> Result<(), ShrugError> {
    let parsed = parse_helper_args(args);

    let project = require_arg(&parsed, "project", "+create")?;
    let summary = require_arg(&parsed, "summary", "+create")?;
    let issue_type = parsed
        .get("type")
        .cloned()
        .unwrap_or_else(|| "Task".to_string());

    // Build fields object
    let mut fields = serde_json::json!({
        "project": {"key": project},
        "summary": summary,
        "issuetype": {"name": issue_type}
    });

    // Optional: description (auto-convert Markdown to ADF)
    if let Some(description) = parsed.get("description") {
        let adf = markdown_to_adf::markdown_to_adf(description);
        fields["description"] = adf;
    }

    if let Some(assignee) = parsed.get("assignee") {
        fields["assignee"] = serde_json::json!({"id": assignee});
    }

    if let Some(priority) = parsed.get("priority") {
        fields["priority"] = serde_json::json!({"name": priority});
    }

    if let Some(labels) = parsed.get("labels") {
        let label_list: Vec<&str> = labels.split(',').map(|s| s.trim()).collect();
        fields["labels"] = serde_json::json!(label_list);
    }

    let body = serde_json::json!({"fields": fields});
    let body_str = serde_json::to_string(&body)
        .map_err(|e| ShrugError::UsageError(format!("Failed to build request body: {}", e)))?;

    // Find the createIssue operation
    let operation = find_operation(spec, "createIssue", "+create")?;

    // Build URL
    let base_url = resolve_helper_base_url(spec.server_url.as_deref(), credential);
    let url = analysis::build_url(base_url.as_deref(), &operation.path, &HashMap::new())?;

    if dry_run {
        eprintln!("POST {}", url);
        eprintln!("Body: {}", body_str);
        return Ok(());
    }

    // Make HTTP request directly
    let response = send_json_request(
        client,
        reqwest::Method::POST,
        &url,
        credential,
        Some(&body_str),
    )?;

    // Extract issue key from response
    if let Some(key) = response.get("key").and_then(|k| k.as_str()) {
        println!("Created: {}", key);
    } else {
        // Fallback: print the full response
        println!(
            "{}",
            serde_json::to_string_pretty(&response).unwrap_or_else(|_| response.to_string())
        );
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn helper_search(
    args: &[String],
    spec: &ApiSpec,
    client: &Client,
    credential: Option<&ResolvedCredential>,
    jql_shorthand: &JqlShorthand,
    raw_jql: Option<&str>,
    format: &OutputFormat,
    is_tty: bool,
    color_enabled: bool,
    fields: Option<&[String]>,
    no_pager: bool,
    dry_run: bool,
) -> Result<(), ShrugError> {
    // Build JQL: use shorthand, raw JQL, or default
    let jql = jql_shorthand
        .build_jql(raw_jql)
        .unwrap_or_else(|| "assignee = currentUser() AND resolution = Unresolved".to_string());

    // Parse any extra args (currently unused, but keeps interface consistent)
    let _parsed = parse_helper_args(args);

    // Find the search operation
    let operation = find_operation(spec, "searchForIssuesUsingJql", "+search")?;

    // Build URL with jql query param
    let base_url = resolve_helper_base_url(spec.server_url.as_deref(), credential);
    let query_params = vec![("jql".to_string(), jql.clone())];
    let base = analysis::build_url(base_url.as_deref(), &operation.path, &HashMap::new())?;
    let query_string = analysis::build_query_string(&query_params);
    let url = if query_string.is_empty() {
        base
    } else {
        format!("{}?{}", base, query_string)
    };

    if dry_run {
        eprintln!("GET {}", url);
        return Ok(());
    }

    // Make HTTP request
    let response = send_json_request(client, reqwest::Method::GET, &url, credential, None)?;

    // Format and display results
    let body_str = serde_json::to_string(&response).unwrap_or_else(|_| response.to_string());
    let formatted = output::format_response(&body_str, format, is_tty, color_enabled, fields);
    output::print_with_pager(&formatted, !no_pager, is_tty);

    Ok(())
}

fn helper_transition(
    args: &[String],
    spec: &ApiSpec,
    client: &Client,
    credential: Option<&ResolvedCredential>,
    dry_run: bool,
) -> Result<(), ShrugError> {
    let parsed = parse_helper_args(args);

    let issue_key = require_arg(&parsed, "issue", "+transition")?;
    let target_name = require_arg(&parsed, "to", "+transition")?;

    // Find operations
    let get_op = find_operation(spec, "getTransitions", "+transition (get)")?;
    let do_op = find_operation(spec, "doTransition", "+transition (post)")?;

    let base_url = resolve_helper_base_url(spec.server_url.as_deref(), credential);

    // Build GET transitions URL
    let mut path_params = HashMap::new();
    path_params.insert("issueIdOrKey".to_string(), issue_key.clone());
    let get_url = analysis::build_url(base_url.as_deref(), &get_op.path, &path_params)?;

    // Build POST transition URL
    let post_url = analysis::build_url(base_url.as_deref(), &do_op.path, &path_params)?;

    if dry_run {
        eprintln!("GET  {}", get_url);
        eprintln!("POST {}", post_url);
        eprintln!("(transition name: \"{}\")", target_name);
        return Ok(());
    }

    // Step 1: GET available transitions
    let transitions_response =
        send_json_request(client, reqwest::Method::GET, &get_url, credential, None)?;

    // Parse transitions array
    let transitions = transitions_response
        .get("transitions")
        .and_then(|t| t.as_array())
        .ok_or_else(|| {
            ShrugError::UsageError(format!("Could not retrieve transitions for {}.", issue_key))
        })?;

    // Step 2: Match target name (case-insensitive)
    let matched = transitions.iter().find(|t| {
        t.get("name")
            .and_then(|n| n.as_str())
            .map(|n| n.eq_ignore_ascii_case(&target_name))
            .unwrap_or(false)
    });

    let transition = match matched {
        Some(t) => t,
        None => {
            let available: Vec<String> = transitions
                .iter()
                .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(String::from))
                .collect();
            return Err(ShrugError::UsageError(format!(
                "Transition '{}' not found for {}.\n\nAvailable transitions:\n{}",
                target_name,
                issue_key,
                available
                    .iter()
                    .map(|n| format!("  {n}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )));
        }
    };

    let transition_id = transition
        .get("id")
        .and_then(|id| id.as_str())
        .ok_or_else(|| ShrugError::UsageError("Transition has no ID.".to_string()))?;

    let transition_name = transition
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or(&target_name);

    // Step 3: POST transition
    let body = serde_json::json!({"transition": {"id": transition_id}});
    let body_str = serde_json::to_string(&body)
        .map_err(|e| ShrugError::UsageError(format!("Failed to build request body: {}", e)))?;

    send_json_request(
        client,
        reqwest::Method::POST,
        &post_url,
        credential,
        Some(&body_str),
    )?;

    println!("{} \u{2192} {}", issue_key, transition_name);

    Ok(())
}

// --- Shared utilities ---

/// Parse --key value pairs from args.
pub fn parse_helper_args(args: &[String]) -> HashMap<String, String> {
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

/// Require a parsed arg, returning a clear error if missing.
fn require_arg(
    args: &HashMap<String, String>,
    name: &str,
    helper: &str,
) -> Result<String, ShrugError> {
    args.get(name).cloned().ok_or_else(|| {
        ShrugError::UsageError(format!(
            "Missing required flag '--{}' for {}.",
            name, helper
        ))
    })
}

/// Find an operation by operationId in the spec, with a clear not-found error.
fn find_operation(
    spec: &ApiSpec,
    operation_id: &str,
    helper_context: &str,
) -> Result<Operation, ShrugError> {
    spec.operations
        .iter()
        .find(|op| op.operation_id == operation_id)
        .cloned()
        .ok_or_else(|| {
            ShrugError::SpecError(format!(
                "Helper {} requires the '{}' operation but it was not found in the loaded spec.",
                helper_context, operation_id
            ))
        })
}

/// Resolve the base URL for helper requests.
fn resolve_helper_base_url(
    spec_server_url: Option<&str>,
    credential: Option<&ResolvedCredential>,
) -> Option<String> {
    // Try spec server URL first (strip variable templates)
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

/// Apply auth headers to a request builder.
fn apply_helper_auth(
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
        None => request,
    }
}

/// Send an HTTP request and return the parsed JSON response.
///
/// For 204 No Content (common for POST transitions), returns an empty JSON object.
/// For non-2xx responses, returns an error with the response body.
fn send_json_request(
    client: &Client,
    method: reqwest::Method,
    url: &str,
    credential: Option<&ResolvedCredential>,
    body: Option<&str>,
) -> Result<serde_json::Value, ShrugError> {
    let mut request = client.request(method, url);
    request = apply_helper_auth(request, credential);
    request = request.header("Accept", "application/json");

    if let Some(body_str) = body {
        request = request
            .header("Content-Type", "application/json")
            .body(body_str.to_string());
    }

    let response = request.send().map_err(ShrugError::NetworkError)?;

    let status = response.status();

    if status == reqwest::StatusCode::NO_CONTENT {
        return Ok(serde_json::json!({}));
    }

    let body_text = response.text().unwrap_or_default();

    if !status.is_success() {
        return Err(ShrugError::ServerError {
            status: status.as_u16(),
            message: body_text,
        });
    }

    serde_json::from_str(&body_text).map_err(|_| ShrugError::ServerError {
        status: status.as_u16(),
        message: format!("Unexpected response: {}", body_text),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_helper_command_detects_plus_prefix() {
        assert!(is_helper_command(&["+create".to_string()]));
        assert!(is_helper_command(&[
            "+search".to_string(),
            "--project".to_string(),
            "FOO".to_string()
        ]));
        assert!(is_helper_command(&["+transition".to_string()]));
    }

    #[test]
    fn is_helper_command_returns_false_for_normal() {
        assert!(!is_helper_command(&["issues".to_string()]));
        assert!(!is_helper_command(&[]));
        assert!(!is_helper_command(&["create".to_string()]));
    }

    #[test]
    fn parse_helper_args_parses_key_value_pairs() {
        let args = vec![
            "--project".to_string(),
            "FOO".to_string(),
            "--summary".to_string(),
            "Fix bug".to_string(),
        ];
        let parsed = parse_helper_args(&args);
        assert_eq!(parsed.get("project").unwrap(), "FOO");
        assert_eq!(parsed.get("summary").unwrap(), "Fix bug");
    }

    #[test]
    fn parse_helper_args_empty() {
        let parsed = parse_helper_args(&[]);
        assert!(parsed.is_empty());
    }

    #[test]
    fn require_arg_returns_value_when_present() {
        let mut args = HashMap::new();
        args.insert("project".to_string(), "FOO".to_string());
        assert_eq!(require_arg(&args, "project", "+create").unwrap(), "FOO");
    }

    #[test]
    fn require_arg_errors_when_missing() {
        let args = HashMap::new();
        let err = require_arg(&args, "project", "+create").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("--project"),
            "Error should name the flag: {}",
            msg
        );
        assert!(
            msg.contains("+create"),
            "Error should name the helper: {}",
            msg
        );
    }

    #[test]
    fn build_create_body_minimal() {
        // Simulate what helper_create builds with just project + summary
        let fields = serde_json::json!({
            "project": {"key": "FOO"},
            "summary": "Test issue",
            "issuetype": {"name": "Task"}
        });
        let body = serde_json::json!({"fields": fields});
        assert_eq!(body["fields"]["project"]["key"], "FOO");
        assert_eq!(body["fields"]["summary"], "Test issue");
        assert_eq!(body["fields"]["issuetype"]["name"], "Task");
    }

    #[test]
    fn build_create_body_all_fields() {
        let mut fields = serde_json::json!({
            "project": {"key": "BAR"},
            "summary": "Full issue",
            "issuetype": {"name": "Bug"}
        });

        // Description auto-converted to ADF
        let adf = markdown_to_adf::markdown_to_adf("# Steps\n\n1. Do thing");
        fields["description"] = adf;

        fields["assignee"] = serde_json::json!({"id": "abc123"});
        fields["priority"] = serde_json::json!({"name": "High"});
        fields["labels"] = serde_json::json!(["bug", "urgent"]);

        let body = serde_json::json!({"fields": fields});
        assert_eq!(body["fields"]["description"]["type"], "doc");
        assert_eq!(body["fields"]["assignee"]["id"], "abc123");
        assert_eq!(body["fields"]["priority"]["name"], "High");
        assert_eq!(body["fields"]["labels"][0], "bug");
    }

    #[test]
    fn create_description_auto_converts_to_adf() {
        let adf = markdown_to_adf::markdown_to_adf("**bold** text");
        assert_eq!(adf["type"], "doc");
        assert_eq!(adf["version"], 1);
        let content = adf["content"].as_array().unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn search_builds_jql_from_shorthand() {
        let shorthand = JqlShorthand {
            project: Some("FOO".to_string()),
            status: Some("Open".to_string()),
            ..Default::default()
        };
        let jql = shorthand.build_jql(None).unwrap();
        assert!(jql.contains("project = \"FOO\""));
        assert!(jql.contains("status = \"Open\""));
    }

    #[test]
    fn search_defaults_to_current_user() {
        let shorthand = JqlShorthand::default();
        let jql = shorthand
            .build_jql(None)
            .unwrap_or_else(|| "assignee = currentUser() AND resolution = Unresolved".to_string());
        assert!(jql.contains("currentUser()"));
        assert!(jql.contains("Unresolved"));
    }

    #[test]
    fn transition_finds_matching_name_case_insensitive() {
        let transitions = serde_json::json!([
            {"id": "11", "name": "To Do"},
            {"id": "21", "name": "In Progress"},
            {"id": "31", "name": "Done"}
        ]);
        let arr = transitions.as_array().unwrap();
        let target = "in progress";
        let matched = arr.iter().find(|t| {
            t.get("name")
                .and_then(|n| n.as_str())
                .map(|n| n.eq_ignore_ascii_case(target))
                .unwrap_or(false)
        });
        assert!(matched.is_some());
        assert_eq!(matched.unwrap()["id"], "21");
    }

    #[test]
    fn transition_error_lists_available_names() {
        let transitions = serde_json::json!([
            {"id": "11", "name": "To Do"},
            {"id": "21", "name": "In Progress"},
            {"id": "31", "name": "Done"}
        ]);
        let arr = transitions.as_array().unwrap();
        let target = "NonExistent";
        let matched = arr.iter().find(|t| {
            t.get("name")
                .and_then(|n| n.as_str())
                .map(|n| n.eq_ignore_ascii_case(target))
                .unwrap_or(false)
        });
        assert!(matched.is_none());

        // Build error message
        let available: Vec<String> = arr
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(String::from))
            .collect();
        assert_eq!(available.len(), 3);
        assert!(available.contains(&"To Do".to_string()));
        assert!(available.contains(&"In Progress".to_string()));
        assert!(available.contains(&"Done".to_string()));
    }

    #[test]
    fn unknown_helper_lists_available() {
        let result = dispatch_helper(
            "unknown",
            &Product::Jira,
            &[],
            &make_empty_spec(),
            &Client::new(),
            None,
            &JqlShorthand::default(),
            None,
            &OutputFormat::Json,
            false,
            false,
            None,
            false,
            false,
        );
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("+create"), "Should list +create: {}", msg);
        assert!(msg.contains("+search"), "Should list +search: {}", msg);
        assert!(
            msg.contains("+transition"),
            "Should list +transition: {}",
            msg
        );
    }

    #[test]
    fn dispatch_rejects_non_jira_product() {
        let result = dispatch_helper(
            "create",
            &Product::Confluence,
            &[],
            &make_empty_spec(),
            &Client::new(),
            None,
            &JqlShorthand::default(),
            None,
            &OutputFormat::Json,
            false,
            false,
            None,
            false,
            false,
        );
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("only available for Jira"),
            "Should mention Jira-only: {}",
            msg
        );
    }

    #[test]
    fn find_operation_returns_error_when_not_found() {
        let spec = make_empty_spec();
        let result = find_operation(&spec, "createIssue", "+create");
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("createIssue"),
            "Should name the operation: {}",
            msg
        );
        assert!(msg.contains("+create"), "Should name the helper: {}", msg);
    }

    fn make_empty_spec() -> ApiSpec {
        ApiSpec {
            title: "Test".to_string(),
            version: "1.0".to_string(),
            server_url: None,
            tags: Vec::new(),
            operations: Vec::new(),
        }
    }
}
