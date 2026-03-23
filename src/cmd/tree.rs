use std::collections::HashMap;

use crate::cmd::crud::{self, CrudMapping};
use crate::cmd::router::{available_tags, operation_to_command_name, operations_for_tag};
use crate::spec::model::{ApiSpec, Operation, Parameter};

/// Truncate a description to a single line, stripping Markdown formatting.
///
/// Strips Markdown links `[text](url)`, bold `**text**`, italic `_text_`,
/// and HTML tags. Truncates at word boundary with "..." if exceeding max_len.
fn truncate_description(desc: &str, max_len: usize) -> String {
    if desc.is_empty() {
        return String::new();
    }

    // Take only the first line/sentence
    let first_line = desc.lines().next().unwrap_or(desc);

    // Strip Markdown links: [text](url) → text
    let mut result = String::with_capacity(first_line.len());
    let mut chars = first_line.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '[' {
            // Collect link text until ]
            let mut link_text = String::new();
            for inner in chars.by_ref() {
                if inner == ']' {
                    break;
                }
                link_text.push(inner);
            }
            // Skip (url) if present
            if chars.peek() == Some(&'(') {
                chars.next(); // consume '('
                for inner in chars.by_ref() {
                    if inner == ')' {
                        break;
                    }
                }
            }
            result.push_str(&link_text);
        } else if ch == '<' {
            // Skip HTML tags
            for inner in chars.by_ref() {
                if inner == '>' {
                    break;
                }
            }
        } else {
            result.push(ch);
        }
    }

    // Strip bold **text** and italic _text_
    let result = result.replace("**", "").replace("__", "");

    // Trim whitespace
    let result = result.trim().to_string();

    if result.len() <= max_len {
        return result;
    }

    // Truncate at word boundary
    let truncated = &result[..max_len];
    match truncated.rfind(' ') {
        Some(pos) => format!("{}...", &truncated[..pos]),
        None => format!("{truncated}..."),
    }
}

/// Format the list of available tags with descriptions and operation counts.
pub fn format_tag_list(spec: &ApiSpec) -> String {
    let tags = available_tags(spec);
    if tags.is_empty() {
        return "No command groups available.".to_string();
    }

    let mut lines = vec!["Available command groups:".to_string()];
    for tag_name in &tags {
        let raw_description = spec
            .tags
            .iter()
            .find(|t| t.name.eq_ignore_ascii_case(tag_name))
            .and_then(|t| t.description.as_deref())
            .unwrap_or("");
        let description = truncate_description(raw_description, 60);
        let count = operations_for_tag(spec, tag_name).len();
        let ops_label = if count == 1 {
            "operation"
        } else {
            "operations"
        };
        lines.push(format!(
            "  {tag_name:<20} {description} ({count} {ops_label})"
        ));
    }
    lines.join("\n")
}

/// Format the list of operations for a specific tag.
pub fn format_operations(spec: &ApiSpec, tag: &str) -> String {
    let crud_mappings = crud::build_crud_mappings(spec);
    format_operations_with_crud(spec, tag, &crud_mappings)
}

/// Format operations with CRUD verbs shown at the top, followed by raw operations.
pub fn format_operations_with_crud(
    spec: &ApiSpec,
    tag: &str,
    crud_mappings: &HashMap<String, CrudMapping>,
) -> String {
    let ops = operations_for_tag(spec, tag);
    if ops.is_empty() {
        return format!("No operations available for '{tag}'.");
    }

    let mut lines = vec![format!("Operations for '{tag}':")];

    // Show CRUD verbs at the top if any are mapped
    if let Some(mapping) = crud_mappings.get(tag) {
        let mut has_crud = false;
        for verb in crud::CRUD_VERBS {
            if let Some(line) = crud::format_crud_line(verb, mapping) {
                lines.push(line);
                has_crud = true;
            }
        }
        if has_crud {
            lines.push("  ─────────────────────────────────".to_string());
        }
    }

    // Show only unmapped raw operations below (hide CRUD-mapped ones)
    let crud_op_ids: Vec<String> = if let Some(mapping) = crud_mappings.get(tag) {
        let mut ids = Vec::new();
        if let Some(op) = &mapping.list { ids.push(op.operation_id.clone()); }
        if let Some(op) = &mapping.get { ids.push(op.operation_id.clone()); }
        if let Some(op) = &mapping.delete { ids.push(op.operation_id.clone()); }
        ids
    } else {
        Vec::new()
    };

    for op in &ops {
        if crud_op_ids.contains(&op.operation_id) {
            continue;
        }
        // Hide deprecated operations from listings (still accessible by raw operation ID)
        if op.deprecated {
            continue;
        }
        let name = operation_to_command_name(&op.operation_id);
        let method = format!("{}", op.method);
        let summary = op.summary.as_deref().unwrap_or("");
        lines.push(format!("  {name:<34} {method:<7} {summary}"));
    }
    lines.join("\n")
}

/// Format full operation details including parameters.
pub fn format_operation_detail(operation: &Operation, server_url: Option<&str>) -> String {
    let name = operation_to_command_name(&operation.operation_id);
    let summary = operation.summary.as_deref().unwrap_or("No description");
    let method = format!("{}", operation.method);
    let deprecated_marker = if operation.deprecated {
        " [DEPRECATED]"
    } else {
        ""
    };

    let mut lines = vec![
        format!("{name} — {summary}{deprecated_marker}"),
        format!("  {method} {}", operation.path),
    ];

    if let Some(url) = server_url {
        lines.push(format!("  Server: {url}"));
    }

    if operation.request_body.is_some() {
        lines.push("  Request body: required (--json)".to_string());
    }

    if !operation.parameters.is_empty() {
        lines.push(String::new());
        lines.push("  Parameters:".to_string());
        lines.push(format_params(&operation.parameters));
    }

    lines.join("\n")
}

/// Format parameters as a table with required params first.
pub fn format_params(params: &[Parameter]) -> String {
    let mut sorted: Vec<&Parameter> = params.iter().collect();
    // Required first, then alphabetical
    sorted.sort_by(|a, b| b.required.cmp(&a.required).then(a.name.cmp(&b.name)));

    sorted
        .iter()
        .map(|p| {
            let req = if p.required { "required" } else { "optional" };
            let loc = match p.location {
                crate::spec::model::ParameterLocation::Path => "path",
                crate::spec::model::ParameterLocation::Query => "query",
                crate::spec::model::ParameterLocation::Header => "header",
                crate::spec::model::ParameterLocation::Cookie => "cookie",
            };
            let typ = p.schema_type.as_deref().unwrap_or("-");
            let desc = p.description.as_deref().unwrap_or("");
            format!("    {:<20} {:<7} {:<9} {:<8} {desc}", p.name, loc, req, typ)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::model::*;

    fn test_spec() -> ApiSpec {
        ApiSpec {
            title: "Test API".to_string(),
            version: "1.0.0".to_string(),
            server_url: Some("https://example.com".to_string()),
            tags: vec![
                Tag {
                    name: "issues".to_string(),
                    description: Some("Issue operations".to_string()),
                },
                Tag {
                    name: "projects".to_string(),
                    description: Some("Project operations".to_string()),
                },
            ],
            operations: vec![
                Operation {
                    operation_id: "createIssue".to_string(),
                    method: HttpMethod::Post,
                    path: "/issue".to_string(),
                    summary: Some("Create an issue".to_string()),
                    description: None,
                    tags: vec!["issues".to_string()],
                    deprecated: false,
                    parameters: vec![],
                    request_body: Some(RequestBody {
                        required: true,
                        description: None,
                        content_types: vec!["application/json".to_string()],
                    }),
                },
                Operation {
                    operation_id: "getIssue".to_string(),
                    method: HttpMethod::Get,
                    path: "/issue/{issueIdOrKey}".to_string(),
                    summary: Some("Get an issue".to_string()),
                    description: None,
                    tags: vec!["issues".to_string()],
                    deprecated: false,
                    parameters: vec![
                        Parameter {
                            name: "issueIdOrKey".to_string(),
                            location: ParameterLocation::Path,
                            required: true,
                            description: Some("The issue key".to_string()),
                            schema_type: Some("string".to_string()),
                        },
                        Parameter {
                            name: "expand".to_string(),
                            location: ParameterLocation::Query,
                            required: false,
                            description: Some("Fields to expand".to_string()),
                            schema_type: Some("string".to_string()),
                        },
                    ],
                    request_body: None,
                },
                Operation {
                    operation_id: "editIssue".to_string(),
                    method: HttpMethod::Put,
                    path: "/issue/{issueIdOrKey}".to_string(),
                    summary: Some("Edit an issue".to_string()),
                    description: None,
                    tags: vec!["issues".to_string()],
                    deprecated: true,
                    parameters: vec![],
                    request_body: None,
                },
                Operation {
                    operation_id: "getProject".to_string(),
                    method: HttpMethod::Get,
                    path: "/project/{id}".to_string(),
                    summary: Some("Get a project".to_string()),
                    description: None,
                    tags: vec!["projects".to_string()],
                    deprecated: false,
                    parameters: vec![],
                    request_body: None,
                },
            ],
        }
    }

    #[test]
    fn format_tag_list_shows_tags_with_descriptions() {
        let spec = test_spec();
        let output = format_tag_list(&spec);
        assert!(output.contains("issues"));
        assert!(output.contains("Issue operations"));
        assert!(output.contains("3 operations"));
        assert!(output.contains("projects"));
        assert!(output.contains("1 operation"));
    }

    #[test]
    fn format_operations_shows_crud_and_raw() {
        let spec = test_spec();
        let output = format_operations(&spec, "issues");
        // CRUD-mapped ops appear as verbs, not raw IDs
        assert!(output.contains("get <"), "Should show CRUD get: {output}");
        // Non-CRUD ops still show as raw
        assert!(output.contains("create-issue"), "Should show unmapped raw op: {output}");
        assert!(output.contains("POST"));
    }

    #[test]
    fn format_operations_hides_deprecated() {
        let spec = test_spec();
        let output = format_operations(&spec, "issues");
        assert!(
            !output.contains("[deprecated]"),
            "Deprecated ops should be hidden: {output}"
        );
        assert!(
            !output.contains("edit-issue"),
            "Deprecated edit-issue should not appear: {output}"
        );
    }

    #[test]
    fn truncate_description_strips_markdown_links() {
        let desc = "Use [Jira REST API](https://developer.atlassian.com) for operations";
        let result = truncate_description(desc, 80);
        assert_eq!(result, "Use Jira REST API for operations");
        assert!(!result.contains("http"));
    }

    #[test]
    fn truncate_description_truncates_long_text() {
        let desc = "This is a very long description that should be truncated because it exceeds the maximum allowed length for display";
        let result = truncate_description(desc, 40);
        assert!(result.ends_with("..."));
        assert!(result.len() <= 44); // 40 + "..."
    }

    #[test]
    fn truncate_description_passes_short_text() {
        let desc = "Short description";
        let result = truncate_description(desc, 80);
        assert_eq!(result, "Short description");
    }

    #[test]
    fn truncate_description_strips_bold() {
        let desc = "This is **bold** text";
        let result = truncate_description(desc, 80);
        assert_eq!(result, "This is bold text");
    }

    #[test]
    fn truncate_description_handles_empty() {
        assert_eq!(truncate_description("", 80), "");
    }

    #[test]
    fn format_operation_detail_shows_parameters() {
        let spec = test_spec();
        let op = spec
            .operations
            .iter()
            .find(|o| o.operation_id == "getIssue")
            .unwrap();
        let output = format_operation_detail(op, Some("https://example.com"));
        assert!(output.contains("issueIdOrKey"));
        assert!(output.contains("required"));
        assert!(output.contains("expand"));
        assert!(output.contains("optional"));
        assert!(output.contains("Server: https://example.com"));
    }

    #[test]
    fn format_params_sorts_required_first() {
        let params = vec![
            Parameter {
                name: "optional_param".to_string(),
                location: ParameterLocation::Query,
                required: false,
                description: None,
                schema_type: Some("string".to_string()),
            },
            Parameter {
                name: "required_param".to_string(),
                location: ParameterLocation::Path,
                required: true,
                description: None,
                schema_type: Some("string".to_string()),
            },
        ];
        let output = format_params(&params);
        let req_pos = output.find("required_param").unwrap();
        let opt_pos = output.find("optional_param").unwrap();
        assert!(req_pos < opt_pos, "Required should come first: {output}");
    }

    #[test]
    fn format_operation_detail_shows_request_body() {
        let spec = test_spec();
        let op = spec
            .operations
            .iter()
            .find(|o| o.operation_id == "createIssue")
            .unwrap();
        let output = format_operation_detail(op, None);
        assert!(
            output.contains("Request body"),
            "Should note request body: {output}"
        );
    }

    #[test]
    fn format_tag_list_empty_spec() {
        let spec = ApiSpec {
            title: "Empty".to_string(),
            version: "1.0".to_string(),
            server_url: None,
            tags: vec![],
            operations: vec![],
        };
        let output = format_tag_list(&spec);
        assert!(output.is_empty() || !output.contains("operations)"));
    }

    #[test]
    fn format_operations_empty_tag() {
        let spec = test_spec();
        let output = format_operations(&spec, "nonexistent-tag");
        // Should handle missing tag gracefully
        assert!(output.is_empty() || output.contains("No operations"));
    }

    #[test]
    fn format_operation_detail_no_params() {
        let op = Operation {
            operation_id: "simple".to_string(),
            method: HttpMethod::Get,
            path: "/simple".to_string(),
            summary: Some("Simple op".to_string()),
            description: None,
            tags: vec![],
            deprecated: false,
            parameters: vec![],
            request_body: None,
        };
        let output = format_operation_detail(&op, Some("https://example.com"));
        assert!(output.contains("simple"));
        assert!(output.contains("https://example.com"));
    }

    #[test]
    fn format_operation_detail_no_summary() {
        let op = Operation {
            operation_id: "nosummary".to_string(),
            method: HttpMethod::Post,
            path: "/test".to_string(),
            summary: None,
            description: None,
            tags: vec![],
            deprecated: false,
            parameters: vec![],
            request_body: None,
        };
        let output = format_operation_detail(&op, None);
        assert!(output.contains("nosummary"));
    }
}
