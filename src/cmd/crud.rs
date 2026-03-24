//! CRUD verb mapping layer.
//!
//! Scans an ApiSpec and identifies which operations map to standard CRUD verbs
//! (list, create, get, update, delete) per entity/tag.

use std::collections::HashMap;

use crate::spec::model::{ApiSpec, HttpMethod, Operation, ParameterLocation};

/// CRUD verb names recognised by the router.
pub const CRUD_VERBS: &[&str] = &["list", "create", "get", "update", "delete"];

/// CRUD mappings for a single entity (tag/command group).
#[derive(Debug, Clone)]
pub struct CrudMapping {
    pub list: Option<Operation>,
    pub create: Option<Operation>,
    pub get: Option<Operation>,
    pub update: Option<Operation>,
    pub delete: Option<Operation>,
    /// The path parameter name used for get/update/delete (e.g., "issueIdOrKey", "id").
    pub id_param: Option<String>,
}

impl CrudMapping {
    fn new() -> Self {
        Self {
            list: None,
            create: None,
            get: None,
            update: None,
            delete: None,
            id_param: None,
        }
    }

    /// Look up an operation by CRUD verb name.
    pub fn resolve(&self, verb: &str) -> Option<&Operation> {
        match verb {
            "list" => self.list.as_ref(),
            "create" => self.create.as_ref(),
            "get" => self.get.as_ref(),
            "update" => self.update.as_ref(),
            "delete" => self.delete.as_ref(),
            _ => None,
        }
    }

    /// Whether this verb requires a positional ID argument.
    pub fn verb_needs_id(verb: &str) -> bool {
        matches!(verb, "get" | "update" | "delete")
    }
}

/// Check if a string is a recognised CRUD verb.
pub fn is_crud_verb(s: &str) -> bool {
    CRUD_VERBS.contains(&s)
}

/// Build CRUD mappings for all tags in a spec.
///
/// For each tag, scans operations and maps them to CRUD verbs by:
/// - HTTP method (GET=list/get, DELETE=delete)
/// - Path pattern (collection path=list, item path with `{param}`=get/delete)
/// - Operation ID patterns (getX, deleteX, searchX, listX)
pub fn build_crud_mappings(spec: &ApiSpec) -> HashMap<String, CrudMapping> {
    let mut mappings: HashMap<String, CrudMapping> = HashMap::new();

    // Group operations by tag
    let mut ops_by_tag: HashMap<String, Vec<&Operation>> = HashMap::new();
    for op in &spec.operations {
        for tag in &op.tags {
            let key = tag.to_lowercase();
            ops_by_tag.entry(key).or_default().push(op);
        }
    }

    for (tag, ops) in &ops_by_tag {
        let mut mapping = CrudMapping::new();

        // Find get: GET with a path parameter, prefer non-deprecated, shorter path
        let mut get_candidates: Vec<&&Operation> = ops
            .iter()
            .filter(|op| op.method == HttpMethod::Get && has_path_param(op))
            .collect();
        sort_candidates(&mut get_candidates);
        if let Some(op) = get_candidates.first() {
            mapping.id_param = first_path_param(op).map(|p| p.name.clone());
            mapping.get = Some((**op).clone());
        }

        // Find delete: DELETE with a path parameter
        let mut delete_candidates: Vec<&&Operation> = ops
            .iter()
            .filter(|op| op.method == HttpMethod::Delete && has_path_param(op))
            .collect();
        sort_candidates(&mut delete_candidates);
        if let Some(op) = delete_candidates.first() {
            if mapping.id_param.is_none() {
                mapping.id_param = first_path_param(op).map(|p| p.name.clone());
            }
            mapping.delete = Some((**op).clone());
        }

        // Find list: GET without path params (collection endpoint),
        // or a search/list operation ID pattern
        let list_candidates: Vec<&&Operation> = ops
            .iter()
            .filter(|op| {
                if op.method != HttpMethod::Get {
                    return false;
                }
                if op.deprecated {
                    return false;
                }
                // Collection endpoint (no path params) or search/list pattern
                !has_path_param(op) || is_list_pattern(&op.operation_id)
            })
            .collect();

        // Prefer: no path params first, then list/search patterns, then shortest path
        let best_list = list_candidates
            .iter()
            .min_by_key(|op| {
                let has_path = if has_path_param(op) { 1 } else { 0 };
                let is_pattern = if is_list_pattern(&op.operation_id) {
                    0
                } else {
                    1
                };
                let path_len = op.path.len();
                (has_path, is_pattern, path_len)
            })
            .copied();

        if let Some(op) = best_list {
            mapping.list = Some((*op).clone());
        }

        // Find create: POST without path parameters (collection endpoint)
        let mut create_candidates: Vec<&&Operation> = ops
            .iter()
            .filter(|op| op.method == HttpMethod::Post && !has_path_param(op) && !op.deprecated)
            .collect();
        sort_candidates(&mut create_candidates);
        if let Some(op) = create_candidates.first() {
            mapping.create = Some((**op).clone());
        }

        // Find update: PUT or PATCH with path parameter
        let mut update_candidates: Vec<&&Operation> = ops
            .iter()
            .filter(|op| {
                (op.method == HttpMethod::Put || op.method == HttpMethod::Patch)
                    && has_path_param(op)
                    && !op.deprecated
            })
            .collect();
        sort_candidates(&mut update_candidates);
        if let Some(op) = update_candidates.first() {
            if mapping.id_param.is_none() {
                mapping.id_param = first_path_param(op).map(|p| p.name.clone());
            }
            mapping.update = Some((**op).clone());
        }

        mappings.insert(tag.clone(), mapping);
    }

    mappings
}

/// Check if an operation has any path parameters.
fn has_path_param(op: &Operation) -> bool {
    op.parameters
        .iter()
        .any(|p| p.location == ParameterLocation::Path)
}

/// Get the first path parameter of an operation.
fn first_path_param(op: &Operation) -> Option<&crate::spec::model::Parameter> {
    op.parameters
        .iter()
        .find(|p| p.location == ParameterLocation::Path)
}

/// Check if an operation ID matches a list/search pattern.
fn is_list_pattern(operation_id: &str) -> bool {
    let lower = operation_id.to_lowercase();
    lower.starts_with("search")
        || lower.starts_with("list")
        || lower.starts_with("getall")
        || lower.starts_with("get_all")
}

/// Sort operation candidates: prefer non-deprecated, then shorter paths.
fn sort_candidates(candidates: &mut [&&Operation]) {
    candidates.sort_by_key(|op| {
        let deprecated = if op.deprecated { 1 } else { 0 };
        let path_len = op.path.len();
        (deprecated, path_len)
    });
}

/// Format a summary line for a CRUD verb in help output.
pub fn format_crud_line(verb: &str, mapping: &CrudMapping) -> Option<String> {
    let op = mapping.resolve(verb)?;
    let summary = op.summary.as_deref().unwrap_or("");
    let body_hint = match op.request_body.as_ref() {
        Some(rb) if rb.required && !rb.properties.is_empty() => {
            let props: Vec<String> = rb
                .properties
                .iter()
                .take(5)
                .map(|p| {
                    if p.required {
                        format!("{}*", p.name)
                    } else {
                        p.name.clone()
                    }
                })
                .collect();
            format!(" [body: {}]", props.join(", "))
        }
        Some(rb) if rb.required => " [body required]".to_string(),
        _ => String::new(),
    };
    if CrudMapping::verb_needs_id(verb) {
        let id_name = mapping.id_param.as_deref().unwrap_or("ID").to_uppercase();
        Some(format!(
            "  {verb} <{id_name}>{:<width$} {summary}{body_hint}",
            "",
            width = 30 - verb.len() - id_name.len() - 3
        ))
    } else {
        Some(format!(
            "  {verb}{:<width$} {summary}{body_hint}",
            "",
            width = 34 - verb.len()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::model::*;

    fn make_op(id: &str, method: HttpMethod, path: &str, tag: &str) -> Operation {
        let params = if path.contains('{') {
            vec![Parameter {
                name: path
                    .split('{')
                    .nth(1)
                    .and_then(|s| s.split('}').next())
                    .unwrap_or("id")
                    .to_string(),
                location: ParameterLocation::Path,
                required: true,
                description: None,
                schema_type: Some("string".to_string()),
            }]
        } else {
            vec![]
        };

        Operation {
            operation_id: id.to_string(),
            method,
            path: path.to_string(),
            summary: Some(format!("{} operation", id)),
            description: None,
            tags: vec![tag.to_string()],
            deprecated: false,
            parameters: params,
            request_body: None,
        }
    }

    fn make_spec(ops: Vec<Operation>) -> ApiSpec {
        ApiSpec {
            title: "Test".to_string(),
            version: "1.0".to_string(),
            server_url: None,
            tags: vec![],
            operations: ops,
        }
    }

    #[test]
    fn maps_get_by_method_and_path_param() {
        let spec = make_spec(vec![make_op(
            "getIssue",
            HttpMethod::Get,
            "/issues/{issueId}",
            "issues",
        )]);
        let mappings = build_crud_mappings(&spec);
        let m = mappings.get("issues").unwrap();
        assert!(m.get.is_some());
        assert_eq!(m.get.as_ref().unwrap().operation_id, "getIssue");
        assert_eq!(m.id_param.as_deref(), Some("issueId"));
    }

    #[test]
    fn maps_delete_by_method() {
        let spec = make_spec(vec![make_op(
            "deleteIssue",
            HttpMethod::Delete,
            "/issues/{issueId}",
            "issues",
        )]);
        let mappings = build_crud_mappings(&spec);
        let m = mappings.get("issues").unwrap();
        assert!(m.delete.is_some());
        assert_eq!(m.delete.as_ref().unwrap().operation_id, "deleteIssue");
    }

    #[test]
    fn maps_list_by_collection_endpoint() {
        let spec = make_spec(vec![
            make_op("listIssues", HttpMethod::Get, "/issues", "issues"),
            make_op("getIssue", HttpMethod::Get, "/issues/{id}", "issues"),
        ]);
        let mappings = build_crud_mappings(&spec);
        let m = mappings.get("issues").unwrap();
        assert!(m.list.is_some());
        assert_eq!(m.list.as_ref().unwrap().operation_id, "listIssues");
    }

    #[test]
    fn prefers_non_deprecated() {
        let mut deprecated_op = make_op("getIssueOld", HttpMethod::Get, "/issues/{id}", "issues");
        deprecated_op.deprecated = true;
        let spec = make_spec(vec![
            deprecated_op,
            make_op("getIssue", HttpMethod::Get, "/issues/{id}", "issues"),
        ]);
        let mappings = build_crud_mappings(&spec);
        let m = mappings.get("issues").unwrap();
        assert_eq!(m.get.as_ref().unwrap().operation_id, "getIssue");
    }

    #[test]
    fn raw_fallback_unaffected() {
        // CRUD verbs are separate from raw operation ID matching
        assert!(is_crud_verb("list"));
        assert!(is_crud_verb("create"));
        assert!(is_crud_verb("get"));
        assert!(is_crud_verb("update"));
        assert!(is_crud_verb("delete"));
        assert!(!is_crud_verb("get-issue"));
        assert!(!is_crud_verb("unknown"));
    }

    #[test]
    fn no_mapping_for_missing_verbs() {
        let spec = make_spec(vec![make_op(
            "getIssue",
            HttpMethod::Get,
            "/issues/{id}",
            "issues",
        )]);
        let mappings = build_crud_mappings(&spec);
        let m = mappings.get("issues").unwrap();
        assert!(m.get.is_some());
        assert!(m.list.is_none());
        assert!(m.delete.is_none());
    }

    #[test]
    fn resolve_returns_correct_operation() {
        let spec = make_spec(vec![
            make_op("getIssue", HttpMethod::Get, "/issues/{id}", "issues"),
            make_op("deleteIssue", HttpMethod::Delete, "/issues/{id}", "issues"),
        ]);
        let mappings = build_crud_mappings(&spec);
        let m = mappings.get("issues").unwrap();
        assert_eq!(m.resolve("get").unwrap().operation_id, "getIssue");
        assert_eq!(m.resolve("delete").unwrap().operation_id, "deleteIssue");
        assert!(m.resolve("list").is_none());
        assert!(m.resolve("create").is_none());
    }

    #[test]
    fn verb_needs_id_correct() {
        assert!(CrudMapping::verb_needs_id("get"));
        assert!(CrudMapping::verb_needs_id("update"));
        assert!(CrudMapping::verb_needs_id("delete"));
        assert!(!CrudMapping::verb_needs_id("list"));
        assert!(!CrudMapping::verb_needs_id("create"));
    }

    #[test]
    fn maps_create_by_post_without_path_param() {
        let spec = make_spec(vec![
            make_op("createIssue", HttpMethod::Post, "/issues", "issues"),
            make_op("getIssue", HttpMethod::Get, "/issues/{id}", "issues"),
        ]);
        let mappings = build_crud_mappings(&spec);
        let m = mappings.get("issues").unwrap();
        assert!(m.create.is_some());
        assert_eq!(m.create.as_ref().unwrap().operation_id, "createIssue");
    }

    #[test]
    fn maps_update_by_put_with_path_param() {
        let spec = make_spec(vec![
            make_op("updateIssue", HttpMethod::Put, "/issues/{id}", "issues"),
            make_op("getIssue", HttpMethod::Get, "/issues/{id}", "issues"),
        ]);
        let mappings = build_crud_mappings(&spec);
        let m = mappings.get("issues").unwrap();
        assert!(m.update.is_some());
        assert_eq!(m.update.as_ref().unwrap().operation_id, "updateIssue");
    }

    #[test]
    fn is_list_pattern_matches_search_prefix() {
        assert!(is_list_pattern("searchIssuesUsingJql"));
        assert!(is_list_pattern("searchAndReconsileIssuesUsingJql"));
        assert!(is_list_pattern("listProjects"));
        assert!(is_list_pattern("getAllIssueTypes"));
        assert!(is_list_pattern("get_all_users"));
    }

    #[test]
    fn is_list_pattern_rejects_search_substring() {
        // Fixed: broad .contains("search") was matching non-list operations
        assert!(!is_list_pattern("getSearchRequestsByEventType"));
        assert!(!is_list_pattern("deleteSearchFilter"));
        assert!(!is_list_pattern("updateSearchRequest"));
    }
}
