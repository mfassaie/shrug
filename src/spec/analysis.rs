use std::collections::HashMap;

use crate::error::ShrugError;
use crate::spec::model::{Operation, Parameter, ParameterLocation};

/// Pagination style detected from operation parameters.
#[derive(Debug, Clone, PartialEq)]
pub enum PaginationStyle {
    /// Offset-based: startAt/maxResults (Jira) or start/limit (Confluence)
    Offset {
        start_param: String,
        limit_param: String,
    },
    /// Page-based: page/pagelen
    Page {
        page_param: String,
        size_param: String,
    },
    /// Cursor-based: cursor or after parameter
    Cursor { cursor_param: String },
}

/// Build a full URL from server URL, path template, and path parameters.
///
/// Substitutes `{param}` placeholders with provided values.
/// Percent-encodes values for path segments.
/// Errors if a placeholder has no matching value.
pub fn build_url(
    server_url: Option<&str>,
    path: &str,
    path_params: &HashMap<String, String>,
) -> Result<String, ShrugError> {
    // Substitute path template placeholders
    let mut result = path.to_string();
    for placeholder in extract_placeholders(path) {
        let value = path_params.get(&placeholder).ok_or_else(|| {
            ShrugError::SpecError(format!(
                "Missing path parameter '{{{}}}' for path '{path}'",
                placeholder
            ))
        })?;
        let encoded = percent_encode_path_segment(value);
        result = result.replace(&format!("{{{placeholder}}}"), &encoded);
    }

    // Prepend server URL if present
    match server_url {
        Some(base) => {
            // Strip {variable} templates from server URL (e.g., "{baseUrl}")
            let clean_base = strip_server_variables(base);
            let base = clean_base.trim_end_matches('/');
            let path_part = if result.starts_with('/') {
                result
            } else {
                format!("/{result}")
            };
            Ok(format!("{base}{path_part}"))
        }
        None => Ok(result),
    }
}

/// Build a query string from key-value pairs.
/// Returns empty string if no params.
pub fn build_query_string(params: &[(String, String)]) -> String {
    if params.is_empty() {
        return String::new();
    }
    params
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode_query(k), percent_encode_query(v)))
        .collect::<Vec<_>>()
        .join("&")
}

/// Detect the pagination style of an operation from its query parameters.
pub fn detect_pagination(operation: &Operation) -> Option<PaginationStyle> {
    let query_names: Vec<&str> = operation
        .parameters
        .iter()
        .filter(|p| p.location == ParameterLocation::Query)
        .map(|p| p.name.as_str())
        .collect();

    // Jira offset pattern: startAt + maxResults
    if query_names.contains(&"startAt") && query_names.contains(&"maxResults") {
        return Some(PaginationStyle::Offset {
            start_param: "startAt".to_string(),
            limit_param: "maxResults".to_string(),
        });
    }

    // Confluence offset pattern: start + limit
    if query_names.contains(&"start") && query_names.contains(&"limit") {
        return Some(PaginationStyle::Offset {
            start_param: "start".to_string(),
            limit_param: "limit".to_string(),
        });
    }

    // Page pattern: page + pagelen
    if query_names.contains(&"page") && query_names.contains(&"pagelen") {
        return Some(PaginationStyle::Page {
            page_param: "page".to_string(),
            size_param: "pagelen".to_string(),
        });
    }

    // Cursor pattern: cursor or after
    if let Some(cursor) = query_names.iter().find(|&&n| n == "cursor" || n == "after") {
        return Some(PaginationStyle::Cursor {
            cursor_param: cursor.to_string(),
        });
    }

    None
}

/// Validate that all path template placeholders have matching path parameters.
/// Returns list of missing parameter names (empty = valid).
pub fn validate_path_params(operation: &Operation) -> Vec<String> {
    let placeholders = extract_placeholders(&operation.path);
    let param_names: Vec<&str> = operation
        .parameters
        .iter()
        .filter(|p| p.location == ParameterLocation::Path)
        .map(|p| p.name.as_str())
        .collect();

    placeholders
        .into_iter()
        .filter(|p| !param_names.contains(&p.as_str()))
        .collect()
}

/// Get path parameters from an operation.
pub fn path_params(operation: &Operation) -> Vec<&Parameter> {
    operation
        .parameters
        .iter()
        .filter(|p| p.location == ParameterLocation::Path)
        .collect()
}

/// Get query parameters from an operation.
pub fn query_params(operation: &Operation) -> Vec<&Parameter> {
    operation
        .parameters
        .iter()
        .filter(|p| p.location == ParameterLocation::Query)
        .collect()
}

/// Get required parameters from an operation.
pub fn required_params(operation: &Operation) -> Vec<&Parameter> {
    operation.parameters.iter().filter(|p| p.required).collect()
}

/// Extract `{placeholder}` names from a path template.
fn extract_placeholders(path: &str) -> Vec<String> {
    let mut placeholders = Vec::new();
    let mut chars = path.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            let name: String = chars.by_ref().take_while(|&c| c != '}').collect();
            if !name.is_empty() {
                placeholders.push(name);
            }
        }
    }
    placeholders
}

/// Percent-encode a value for use in a URL path segment.
/// Encodes characters unsafe in path segments while preserving safe ones.
fn percent_encode_path_segment(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            // Unreserved characters (RFC 3986 Section 2.3)
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            // Sub-delimiters safe in path segments
            b'!' | b'$' | b'&' | b'\'' | b'(' | b')' | b'*' | b'+' | b',' | b';' | b'=' | b':'
            | b'@' => {
                encoded.push(byte as char);
            }
            // Everything else gets percent-encoded
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

/// Percent-encode a value for use in a query string parameter.
fn percent_encode_query(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

/// Strip `{variable}` templates from server URLs (e.g., Atlassian's `{baseUrl}`).
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

    // === URL Building Tests ===

    #[test]
    fn build_url_basic_substitution() {
        let params = HashMap::from([("issueIdOrKey".to_string(), "TEST-123".to_string())]);
        let url = build_url(
            Some("https://example.atlassian.net"),
            "/rest/api/3/issue/{issueIdOrKey}",
            &params,
        )
        .unwrap();
        assert_eq!(
            url,
            "https://example.atlassian.net/rest/api/3/issue/TEST-123"
        );
    }

    #[test]
    fn build_url_multiple_params() {
        let params = HashMap::from([
            ("workspace".to_string(), "myteam".to_string()),
            ("repo_slug".to_string(), "myrepo".to_string()),
        ]);
        let url = build_url(
            Some("https://api.example.com/2.0"),
            "/repositories/{workspace}/{repo_slug}",
            &params,
        )
        .unwrap();
        assert_eq!(
            url,
            "https://api.example.com/2.0/repositories/myteam/myrepo"
        );
    }

    #[test]
    fn build_url_missing_param_error() {
        let params = HashMap::new();
        let result = build_url(
            Some("https://example.com"),
            "/issue/{issueIdOrKey}",
            &params,
        );
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(
            err.contains("issueIdOrKey"),
            "Should mention missing param: {err}"
        );
    }

    #[test]
    fn build_url_extra_params_ignored() {
        let params = HashMap::from([
            ("id".to_string(), "123".to_string()),
            ("extra".to_string(), "ignored".to_string()),
        ]);
        let url = build_url(Some("https://example.com"), "/item/{id}", &params).unwrap();
        assert_eq!(url, "https://example.com/item/123");
    }

    #[test]
    fn build_url_percent_encodes_path_segments() {
        let params = HashMap::from([("name".to_string(), "hello world/test".to_string())]);
        let url = build_url(Some("https://example.com"), "/item/{name}", &params).unwrap();
        assert_eq!(url, "https://example.com/item/hello%20world%2Ftest");
    }

    #[test]
    fn build_url_no_server_url() {
        let params = HashMap::from([("id".to_string(), "42".to_string())]);
        let url = build_url(None, "/item/{id}", &params).unwrap();
        assert_eq!(url, "/item/42");
    }

    #[test]
    fn build_url_trailing_slash_dedup() {
        let params = HashMap::new();
        let url = build_url(Some("https://example.com/"), "/api/test", &params).unwrap();
        assert_eq!(url, "https://example.com/api/test");
    }

    #[test]
    fn build_url_no_placeholders() {
        let params = HashMap::new();
        let url = build_url(Some("https://example.com"), "/api/projects", &params).unwrap();
        assert_eq!(url, "https://example.com/api/projects");
    }

    #[test]
    fn build_url_strips_server_variable_templates() {
        let params = HashMap::new();
        let url = build_url(Some("{baseUrl}"), "/api/test", &params).unwrap();
        assert_eq!(url, "/api/test");

        let url2 = build_url(Some("https://{site}.atlassian.net"), "/api/test", &params).unwrap();
        assert_eq!(url2, "https://.atlassian.net/api/test");
    }

    // === Query String Tests ===

    #[test]
    fn build_query_string_basic() {
        let params = vec![
            ("jql".to_string(), "project = TEST".to_string()),
            ("maxResults".to_string(), "50".to_string()),
        ];
        let qs = build_query_string(&params);
        assert_eq!(qs, "jql=project+%3D+TEST&maxResults=50");
    }

    #[test]
    fn build_query_string_empty() {
        let qs = build_query_string(&[]);
        assert_eq!(qs, "");
    }

    #[test]
    fn build_query_string_encodes_special_chars() {
        let params = vec![("q".to_string(), "a&b=c".to_string())];
        let qs = build_query_string(&params);
        assert_eq!(qs, "q=a%26b%3Dc");
    }

    // === Pagination Detection Tests ===

    #[test]
    fn detect_pagination_offset_jira() {
        let op = make_op(
            "search",
            HttpMethod::Get,
            "/search",
            vec![
                query_param("startAt", false),
                query_param("maxResults", false),
                query_param("jql", false),
            ],
            None,
        );
        let style = detect_pagination(&op).unwrap();
        assert_eq!(
            style,
            PaginationStyle::Offset {
                start_param: "startAt".to_string(),
                limit_param: "maxResults".to_string(),
            }
        );
    }

    #[test]
    fn detect_pagination_offset_confluence() {
        let op = make_op(
            "getContent",
            HttpMethod::Get,
            "/content",
            vec![query_param("start", false), query_param("limit", false)],
            None,
        );
        let style = detect_pagination(&op).unwrap();
        assert_eq!(
            style,
            PaginationStyle::Offset {
                start_param: "start".to_string(),
                limit_param: "limit".to_string(),
            }
        );
    }

    #[test]
    fn detect_pagination_page_based() {
        let op = make_op(
            "listRepos",
            HttpMethod::Get,
            "/repositories",
            vec![query_param("page", false), query_param("pagelen", false)],
            None,
        );
        let style = detect_pagination(&op).unwrap();
        assert_eq!(
            style,
            PaginationStyle::Page {
                page_param: "page".to_string(),
                size_param: "pagelen".to_string(),
            }
        );
    }

    #[test]
    fn detect_pagination_cursor() {
        let op = make_op(
            "listItems",
            HttpMethod::Get,
            "/items",
            vec![query_param("after", false), query_param("limit", false)],
            None,
        );
        let style = detect_pagination(&op).unwrap();
        assert_eq!(
            style,
            PaginationStyle::Cursor {
                cursor_param: "after".to_string(),
            }
        );
    }

    #[test]
    fn detect_pagination_none() {
        let op = make_op(
            "getIssue",
            HttpMethod::Get,
            "/issue/{id}",
            vec![path_param("id")],
            None,
        );
        assert!(detect_pagination(&op).is_none());
    }

    // === Path Validation Tests ===

    #[test]
    fn validate_path_params_all_present() {
        let op = make_op(
            "getIssue",
            HttpMethod::Get,
            "/issue/{issueIdOrKey}",
            vec![path_param("issueIdOrKey")],
            None,
        );
        assert!(validate_path_params(&op).is_empty());
    }

    #[test]
    fn validate_path_params_missing_detected() {
        let op = make_op(
            "getIssue",
            HttpMethod::Get,
            "/issue/{issueIdOrKey}",
            vec![], // no params
            None,
        );
        let missing = validate_path_params(&op);
        assert_eq!(missing, vec!["issueIdOrKey"]);
    }

    // === Parameter Helper Tests ===

    #[test]
    fn path_params_filters_correctly() {
        let op = make_op(
            "test",
            HttpMethod::Get,
            "/item/{id}",
            vec![
                path_param("id"),
                query_param("expand", false),
                query_param("fields", false),
            ],
            None,
        );
        assert_eq!(path_params(&op).len(), 1);
        assert_eq!(path_params(&op)[0].name, "id");
    }

    #[test]
    fn query_params_filters_correctly() {
        let op = make_op(
            "test",
            HttpMethod::Get,
            "/item/{id}",
            vec![
                path_param("id"),
                query_param("expand", false),
                query_param("fields", false),
            ],
            None,
        );
        assert_eq!(query_params(&op).len(), 2);
    }

    #[test]
    fn required_params_filters_correctly() {
        let op = make_op(
            "test",
            HttpMethod::Get,
            "/item/{id}",
            vec![
                path_param("id"),             // required=true
                query_param("expand", false), // required=false
                query_param("fields", true),  // required=true
            ],
            None,
        );
        assert_eq!(required_params(&op).len(), 2);
    }
}

#[cfg(test)]
mod conformance {
    use super::*;
    use crate::spec::parse_spec;

    fn jira_fixture() -> &'static str {
        r#"{
            "openapi": "3.0.1",
            "info": { "title": "Jira Platform", "version": "1001.0.0" },
            "servers": [{ "url": "https://example.atlassian.net" }],
            "tags": [
                { "name": "issues", "description": "Issue operations" },
                { "name": "projects", "description": "Project operations" }
            ],
            "paths": {
                "/rest/api/3/issue": {
                    "post": {
                        "operationId": "createIssue",
                        "summary": "Create an issue",
                        "tags": ["issues"],
                        "requestBody": { "required": true, "content": { "application/json": {} } }
                    }
                },
                "/rest/api/3/issue/{issueIdOrKey}": {
                    "parameters": [
                        { "name": "issueIdOrKey", "in": "path", "required": true, "schema": { "type": "string" } }
                    ],
                    "get": {
                        "operationId": "getIssue",
                        "summary": "Get an issue",
                        "tags": ["issues"],
                        "parameters": [
                            { "name": "expand", "in": "query", "required": false, "schema": { "type": "string" } }
                        ]
                    },
                    "put": {
                        "operationId": "editIssue",
                        "summary": "Edit an issue",
                        "tags": ["issues"],
                        "requestBody": { "required": true, "content": { "application/json": {} } }
                    },
                    "delete": {
                        "operationId": "deleteIssue",
                        "summary": "Delete an issue",
                        "tags": ["issues"]
                    }
                },
                "/rest/api/3/search": {
                    "get": {
                        "operationId": "searchForIssuesUsingJql",
                        "summary": "Search for issues using JQL",
                        "tags": ["issues"],
                        "parameters": [
                            { "name": "jql", "in": "query", "required": false, "schema": { "type": "string" } },
                            { "name": "startAt", "in": "query", "required": false, "schema": { "type": "integer" } },
                            { "name": "maxResults", "in": "query", "required": false, "schema": { "type": "integer" } }
                        ]
                    }
                },
                "/rest/api/3/project/{projectIdOrKey}": {
                    "get": {
                        "operationId": "getProject",
                        "summary": "Get a project",
                        "tags": ["projects"],
                        "parameters": [
                            { "name": "projectIdOrKey", "in": "path", "required": true, "schema": { "type": "string" } }
                        ]
                    }
                },
                "/rest/api/3/project": {
                    "get": {
                        "operationId": "listProjects",
                        "summary": "List projects",
                        "tags": ["projects"],
                        "parameters": [
                            { "name": "startAt", "in": "query", "required": false, "schema": { "type": "integer" } },
                            { "name": "maxResults", "in": "query", "required": false, "schema": { "type": "integer" } }
                        ]
                    }
                },
                "/rest/api/3/issue/{issueIdOrKey}/comment": {
                    "post": {
                        "operationId": "addComment",
                        "summary": "Add a comment",
                        "tags": ["issues"],
                        "parameters": [
                            { "name": "issueIdOrKey", "in": "path", "required": true, "schema": { "type": "string" } }
                        ],
                        "requestBody": { "required": true, "content": { "application/json": {} } }
                    }
                },
                "/rest/api/3/issue/{issueIdOrKey}/transitions": {
                    "parameters": [
                        { "name": "issueIdOrKey", "in": "path", "required": true, "schema": { "type": "string" } }
                    ],
                    "get": {
                        "operationId": "getTransitions",
                        "summary": "Get transitions",
                        "tags": ["issues"]
                    },
                    "post": {
                        "operationId": "doTransition",
                        "summary": "Do a transition",
                        "tags": ["issues"],
                        "requestBody": { "required": true, "content": { "application/json": {} } }
                    }
                }
            }
        }"#
    }

    #[test]
    fn conformance_jira_v3_parse_and_analyze() {
        let spec = parse_spec(jira_fixture()).unwrap();

        // Verify operation count
        assert_eq!(
            spec.operations.len(),
            10,
            "Jira fixture should have 10 operations"
        );

        // Verify all operations have valid IDs
        for op in &spec.operations {
            assert!(
                !op.operation_id.is_empty(),
                "All operations should have IDs"
            );
        }

        // Validate path params for all operations
        for op in &spec.operations {
            let missing = validate_path_params(op);
            assert!(
                missing.is_empty(),
                "Operation '{}' has missing path params: {:?}",
                op.operation_id,
                missing
            );
        }

        // Build URLs for all operations with sample path params
        let server = spec.server_url.as_deref();
        for op in &spec.operations {
            let mut params = HashMap::new();
            for p in path_params(op) {
                params.insert(p.name.clone(), "SAMPLE-VALUE".to_string());
            }
            let result = build_url(server, &op.path, &params);
            assert!(
                result.is_ok(),
                "URL building failed for '{}': {:?}",
                op.operation_id,
                result.err()
            );
        }

        // Verify pagination detection
        let search = spec
            .operations
            .iter()
            .find(|o| o.operation_id == "searchForIssuesUsingJql")
            .unwrap();
        assert_eq!(
            detect_pagination(search),
            Some(PaginationStyle::Offset {
                start_param: "startAt".to_string(),
                limit_param: "maxResults".to_string(),
            })
        );

        let list_projects = spec
            .operations
            .iter()
            .find(|o| o.operation_id == "listProjects")
            .unwrap();
        assert!(detect_pagination(list_projects).is_some());

        // Non-paginated operations
        let get_issue = spec
            .operations
            .iter()
            .find(|o| o.operation_id == "getIssue")
            .unwrap();
        assert!(detect_pagination(get_issue).is_none());

        let create_issue = spec
            .operations
            .iter()
            .find(|o| o.operation_id == "createIssue")
            .unwrap();
        assert!(detect_pagination(create_issue).is_none());
    }

}
