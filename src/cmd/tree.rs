use crate::cmd::router::{available_tags, operation_to_command_name, operations_for_tag};
use crate::spec::model::{ApiSpec, Operation, Parameter};

/// Format the list of available tags with descriptions and operation counts.
pub fn format_tag_list(spec: &ApiSpec) -> String {
    let tags = available_tags(spec);
    if tags.is_empty() {
        return "No command groups available.".to_string();
    }

    let mut lines = vec!["Available command groups:".to_string()];
    for tag_name in &tags {
        let description = spec
            .tags
            .iter()
            .find(|t| t.name == *tag_name)
            .and_then(|t| t.description.as_deref())
            .unwrap_or("");
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
    let ops = operations_for_tag(spec, tag);
    if ops.is_empty() {
        return format!("No operations available for '{tag}'.");
    }

    let mut lines = vec![format!("Operations for '{tag}':")];
    for op in &ops {
        let name = operation_to_command_name(&op.operation_id);
        let method = format!("{}", op.method);
        let summary = op.summary.as_deref().unwrap_or("");
        let deprecated = if op.deprecated { " [deprecated]" } else { "" };
        lines.push(format!("  {name}{deprecated:<14} {method:<7} {summary}"));
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
    fn format_operations_shows_command_names_and_methods() {
        let spec = test_spec();
        let output = format_operations(&spec, "issues");
        assert!(output.contains("create-issue"));
        assert!(output.contains("POST"));
        assert!(output.contains("get-issue"));
        assert!(output.contains("GET"));
    }

    #[test]
    fn format_operations_marks_deprecated() {
        let spec = test_spec();
        let output = format_operations(&spec, "issues");
        assert!(
            output.contains("[deprecated]"),
            "Should mark deprecated: {output}"
        );
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
}
