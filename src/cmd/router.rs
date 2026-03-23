use crate::cmd::tree;
use crate::error::ShrugError;
use crate::spec::model::{ApiSpec, Operation};
use crate::spec::registry::Product;

/// A fully resolved command ready for execution.
pub struct ResolvedCommand {
    pub product: Product,
    pub operation: Operation,
    pub server_url: Option<String>,
    pub remaining_args: Vec<String>,
}

/// Convert a camelCase operationId to a kebab-case CLI command name.
///
/// "createIssue" → "create-issue"
/// "searchForIssuesUsingJql" → "search-for-issues-using-jql"
/// "getIssue" → "get-issue"
pub fn operation_to_command_name(operation_id: &str) -> String {
    let mut result = String::with_capacity(operation_id.len() + 4);
    for (i, ch) in operation_id.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('-');
        }
        result.push(ch.to_ascii_lowercase());
    }
    result
}

/// Get unique tag names from a spec, sorted alphabetically, lowercased.
pub fn available_tags(spec: &ApiSpec) -> Vec<String> {
    if !spec.tags.is_empty() {
        let mut tags: Vec<String> = spec
            .tags
            .iter()
            .map(|t| t.name.to_lowercase())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    } else {
        // Fallback: collect unique tags from operations
        let mut tags: Vec<String> = spec
            .operations
            .iter()
            .flat_map(|op| op.tags.iter().map(|t| t.to_lowercase()))
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }
}

/// Get all operations that have the given tag.
pub fn operations_for_tag<'a>(spec: &'a ApiSpec, tag: &str) -> Vec<&'a Operation> {
    spec.operations
        .iter()
        .filter(|op| op.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)))
        .collect()
}

/// Resolve a command from args: [tag, operation, ...remaining].
///
/// Returns the matched Operation and any remaining args for parameter extraction.
pub fn resolve_command(
    spec: &ApiSpec,
    args: &[String],
) -> Result<(Operation, Vec<String>), ShrugError> {
    let tags = available_tags(spec);

    if args.is_empty() {
        return Err(ShrugError::UsageError(format!(
            "No command specified.\n\n{}",
            tree::format_tag_list(spec)
        )));
    }

    let tag_input = &args[0];

    // Find matching tag (case-insensitive, normalize hyphens to match spaces)
    let matched_tag = find_tag_match(tag_input, &tags);

    let tag = match matched_tag {
        Some(t) => t,
        None => {
            // Try to suggest close matches
            let suggestions = find_close_matches(tag_input, &tags);
            let suggestion_msg = if suggestions.is_empty() {
                String::new()
            } else {
                format!(
                    "\n\nDid you mean?\n{}",
                    suggestions
                        .iter()
                        .map(|s| format!("  {s}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            };
            return Err(ShrugError::UsageError(format!(
                "Unknown command group '{tag_input}'.{suggestion_msg}\n\n{}",
                tree::format_tag_list(spec)
            )));
        }
    };

    let ops = operations_for_tag(spec, &tag);

    if args.len() < 2 {
        return Err(ShrugError::UsageError(format!(
            "No operation specified for '{tag}'.\n\n{}",
            tree::format_operations(spec, &tag)
        )));
    }

    let op_input = &args[1];

    // Find matching operation by command name
    let matched_op = ops.iter().find(|op| {
        let cmd_name = operation_to_command_name(&op.operation_id);
        cmd_name == *op_input
    });

    match matched_op {
        Some(op) => Ok(((*op).clone(), args[2..].to_vec())),
        None => {
            // Suggest close matches
            let op_names: Vec<String> = ops
                .iter()
                .map(|op| operation_to_command_name(&op.operation_id))
                .collect();
            let suggestions = find_close_matches(op_input, &op_names);
            let suggestion_msg = if suggestions.is_empty() {
                String::new()
            } else {
                format!(
                    "\n\nDid you mean?\n{}",
                    suggestions
                        .iter()
                        .map(|s| format!("  {s}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            };
            Err(ShrugError::UsageError(format!(
                "Unknown operation '{op_input}' in '{tag}'.{suggestion_msg}\n\n{}",
                tree::format_operations(spec, &tag)
            )))
        }
    }
}

/// Route a product command: resolve product + spec + args into a ResolvedCommand.
pub fn route_product(
    product: &Product,
    spec: &ApiSpec,
    args: &[String],
) -> Result<ResolvedCommand, ShrugError> {
    let (operation, remaining_args) = resolve_command(spec, args)?;
    Ok(ResolvedCommand {
        product: *product,
        operation,
        server_url: spec.server_url.clone(),
        remaining_args,
    })
}

/// Find a tag match with case-insensitive + hyphen/space normalization.
fn find_tag_match(input: &str, tags: &[String]) -> Option<String> {
    let normalized_input = normalize_name(input);
    tags.iter()
        .find(|t| normalize_name(t) == normalized_input)
        .cloned()
}

/// Normalize a name for matching: lowercase, replace hyphens/underscores with spaces.
fn normalize_name(name: &str) -> String {
    name.to_lowercase().replace(['-', '_'], " ")
}

/// Find close matches using prefix and contains matching.
fn find_close_matches(input: &str, candidates: &[String]) -> Vec<String> {
    let input_lower = input.to_lowercase();
    let mut matches: Vec<String> = candidates
        .iter()
        .filter(|c| {
            let c_lower = c.to_lowercase();
            c_lower.starts_with(&input_lower)
                || input_lower.starts_with(&c_lower)
                || c_lower.contains(&input_lower)
        })
        .cloned()
        .collect();
    matches.sort();
    matches.dedup();
    matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::model::*;

    fn test_spec() -> ApiSpec {
        ApiSpec {
            title: "Test API".to_string(),
            version: "1.0.0".to_string(),
            server_url: Some("https://example.atlassian.net".to_string()),
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
                    path: "/rest/api/3/issue".to_string(),
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
                    path: "/rest/api/3/issue/{issueIdOrKey}".to_string(),
                    summary: Some("Get an issue".to_string()),
                    description: None,
                    tags: vec!["issues".to_string()],
                    deprecated: false,
                    parameters: vec![Parameter {
                        name: "issueIdOrKey".to_string(),
                        location: ParameterLocation::Path,
                        required: true,
                        description: None,
                        schema_type: Some("string".to_string()),
                    }],
                    request_body: None,
                },
                Operation {
                    operation_id: "searchForIssuesUsingJql".to_string(),
                    method: HttpMethod::Get,
                    path: "/rest/api/3/search".to_string(),
                    summary: Some("Search for issues using JQL".to_string()),
                    description: None,
                    tags: vec!["issues".to_string()],
                    deprecated: false,
                    parameters: vec![],
                    request_body: None,
                },
                Operation {
                    operation_id: "getProject".to_string(),
                    method: HttpMethod::Get,
                    path: "/rest/api/3/project/{projectIdOrKey}".to_string(),
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

    fn args(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn operation_to_command_name_camel_to_kebab() {
        assert_eq!(operation_to_command_name("createIssue"), "create-issue");
        assert_eq!(operation_to_command_name("getIssue"), "get-issue");
        assert_eq!(
            operation_to_command_name("searchForIssuesUsingJql"),
            "search-for-issues-using-jql"
        );
        assert_eq!(
            operation_to_command_name("deleteRepository"),
            "delete-repository"
        );
        assert_eq!(operation_to_command_name("list"), "list");
    }

    #[test]
    fn available_tags_returns_sorted() {
        let spec = test_spec();
        let tags = available_tags(&spec);
        assert_eq!(tags, vec!["issues", "projects"]);
    }

    #[test]
    fn operations_for_tag_filters_correctly() {
        let spec = test_spec();
        let ops = operations_for_tag(&spec, "issues");
        assert_eq!(ops.len(), 3);
        assert!(ops.iter().any(|o| o.operation_id == "createIssue"));
        assert!(ops.iter().any(|o| o.operation_id == "getIssue"));
        assert!(ops
            .iter()
            .any(|o| o.operation_id == "searchForIssuesUsingJql"));
    }

    #[test]
    fn resolve_command_matches_tag_and_operation() {
        let spec = test_spec();
        let (op, remaining) = resolve_command(&spec, &args(&["issues", "get-issue"])).unwrap();
        assert_eq!(op.operation_id, "getIssue");
        assert!(remaining.is_empty());
    }

    #[test]
    fn resolve_command_returns_remaining_args() {
        let spec = test_spec();
        let (op, remaining) =
            resolve_command(&spec, &args(&["issues", "get-issue", "--expand", "names"])).unwrap();
        assert_eq!(op.operation_id, "getIssue");
        assert_eq!(remaining, vec!["--expand", "names"]);
    }

    #[test]
    fn resolve_command_error_on_empty_args_lists_tags() {
        let spec = test_spec();
        let err = resolve_command(&spec, &args(&[])).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("issues"), "Should list tags: {msg}");
        assert!(msg.contains("projects"), "Should list tags: {msg}");
    }

    #[test]
    fn resolve_command_error_on_unknown_tag() {
        let spec = test_spec();
        let err = resolve_command(&spec, &args(&["nonexistent"])).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("Unknown command group"),
            "Should report unknown: {msg}"
        );
        assert!(msg.contains("issues"), "Should list available tags: {msg}");
    }

    #[test]
    fn resolve_command_error_on_unknown_operation() {
        let spec = test_spec();
        let err = resolve_command(&spec, &args(&["issues", "nonexistent"])).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("Unknown operation"),
            "Should report unknown op: {msg}"
        );
        assert!(
            msg.contains("create-issue"),
            "Should list available ops: {msg}"
        );
    }

    #[test]
    fn resolve_command_tag_only_lists_operations() {
        let spec = test_spec();
        let err = resolve_command(&spec, &args(&["issues"])).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("No operation specified"), "{msg}");
        assert!(msg.contains("create-issue"), "Should list ops: {msg}");
        assert!(msg.contains("get-issue"), "Should list ops: {msg}");
    }

    #[test]
    fn resolve_command_case_insensitive_tag() {
        let spec = test_spec();
        let result = resolve_command(&spec, &args(&["Issues", "get-issue"]));
        assert!(result.is_ok(), "Tag matching should be case-insensitive");
    }

    #[test]
    fn resolve_command_suggests_close_matches() {
        let spec = test_spec();
        let err = resolve_command(&spec, &args(&["issue"])).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("Did you mean"),
            "Should suggest close match: {msg}"
        );
        assert!(msg.contains("issues"), "Should suggest 'issues': {msg}");
    }

    #[test]
    fn route_product_produces_resolved_command() {
        let spec = test_spec();
        let resolved =
            route_product(&Product::Jira, &spec, &args(&["issues", "create-issue"])).unwrap();
        assert_eq!(resolved.product, Product::Jira);
        assert_eq!(resolved.operation.operation_id, "createIssue");
        assert_eq!(
            resolved.server_url,
            Some("https://example.atlassian.net".to_string())
        );
        assert!(resolved.remaining_args.is_empty());
    }
}
