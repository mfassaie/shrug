use serde_json::Value;

use crate::error::ShrugError;
use crate::spec::model::*;

/// Parse an OpenAPI 3.0.1 JSON spec into shrug's data model.
pub fn parse_openapi_v3(json: &str) -> Result<ApiSpec, ShrugError> {
    let doc: Value = serde_json::from_str(json)
        .map_err(|e| ShrugError::SpecError(format!("Invalid JSON: {e}")))?;

    // Validate OpenAPI version
    let version_str = doc
        .get("openapi")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ShrugError::SpecError("Missing 'openapi' version field".into()))?;

    if !version_str.starts_with("3.") {
        return Err(ShrugError::SpecError(format!(
            "Unsupported OpenAPI version '{}'. This parser supports 3.x only. For Swagger 2.0, use the swagger parser.",
            version_str
        )));
    }

    // Extract info
    let info = doc.get("info").unwrap_or(&Value::Null);
    let title = info
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled")
        .to_string();
    let version = info
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0")
        .to_string();

    // Extract server URL
    let server_url = doc
        .get("servers")
        .and_then(|s| s.as_array())
        .and_then(|arr| arr.first())
        .and_then(|s| s.get("url"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Extract tags
    let tags = parse_tags(&doc);

    // Extract operations from paths
    let operations = parse_paths(&doc)?;

    Ok(ApiSpec {
        title,
        version,
        server_url,
        tags,
        operations,
    })
}

fn parse_tags(doc: &Value) -> Vec<Tag> {
    doc.get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| {
                    let name = t.get("name")?.as_str()?.to_string();
                    let description = t
                        .get("description")
                        .and_then(|d| d.as_str())
                        .map(String::from);
                    Some(Tag { name, description })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_paths(doc: &Value) -> Result<Vec<Operation>, ShrugError> {
    let paths = match doc.get("paths").and_then(|v| v.as_object()) {
        Some(p) => p,
        None => return Ok(Vec::new()),
    };

    let methods = ["get", "post", "put", "delete", "patch"];
    let mut operations = Vec::new();

    for (path, path_item) in paths {
        let path_level_params = path_item
            .get("parameters")
            .map(parse_parameters)
            .unwrap_or_default();

        for method_str in &methods {
            if let Some(op_value) = path_item.get(*method_str) {
                let method = match parse_method(method_str) {
                    Some(m) => m,
                    None => continue,
                };

                let operation_id = match op_value.get("operationId").and_then(|v| v.as_str()) {
                    Some(id) => id.to_string(),
                    None => {
                        tracing::warn!(
                            path = %path,
                            method = %method_str,
                            "Skipping operation without operationId"
                        );
                        continue;
                    }
                };

                let summary = op_value
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let description = op_value
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                let op_tags = op_value
                    .get("tags")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|t| t.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                let deprecated = op_value
                    .get("deprecated")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                // Merge parameters: path-level first, then operation-level overrides by name
                let op_level_params = op_value
                    .get("parameters")
                    .map(parse_parameters)
                    .unwrap_or_default();
                let parameters = merge_parameters(&path_level_params, &op_level_params);

                let request_body = op_value.get("requestBody").map(|rb| {
                    let required = rb
                        .get("required")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let rb_description = rb
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    let content_types = rb
                        .get("content")
                        .and_then(|v| v.as_object())
                        .map(|obj| obj.keys().cloned().collect())
                        .unwrap_or_default();

                    // Extract top-level schema properties from application/json content
                    let properties = extract_body_properties(rb);

                    RequestBody {
                        required,
                        description: rb_description,
                        content_types,
                        properties,
                    }
                });

                operations.push(Operation {
                    operation_id,
                    method,
                    path: path.clone(),
                    summary,
                    description,
                    tags: op_tags,
                    deprecated,
                    parameters,
                    request_body,
                });
            }
        }
    }

    Ok(operations)
}

/// Extract top-level schema properties from a request body's JSON content.
fn extract_body_properties(rb: &Value) -> Vec<BodyProperty> {
    let schema = rb.pointer("/content/application~1json/schema").or_else(|| {
        // Some specs use other JSON content types
        rb.get("content")
            .and_then(|c| c.as_object())
            .and_then(|obj| obj.iter().find(|(k, _)| k.contains("json")).map(|(_, v)| v))
            .and_then(|v| v.get("schema"))
    });

    let schema = match schema {
        Some(s) => s,
        None => return Vec::new(),
    };

    let required_fields: Vec<String> = schema
        .get("required")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let props = match schema.get("properties").and_then(|p| p.as_object()) {
        Some(p) => p,
        None => return Vec::new(),
    };

    props
        .iter()
        .map(|(name, prop_value)| {
            let schema_type = prop_value
                .get("type")
                .and_then(|t| t.as_str())
                .map(String::from);
            let description = prop_value
                .get("description")
                .and_then(|d| d.as_str())
                .map(String::from);
            BodyProperty {
                name: name.clone(),
                schema_type,
                required: required_fields.contains(name),
                description,
            }
        })
        .collect()
}

fn parse_parameters(params: &Value) -> Vec<Parameter> {
    params
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|p| {
                    let name = p.get("name")?.as_str()?.to_string();
                    let location = p
                        .get("in")
                        .and_then(|v| v.as_str())
                        .and_then(parse_parameter_location)?;
                    let required = p.get("required").and_then(|v| v.as_bool()).unwrap_or(false);
                    let description = p
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    let schema_type = p
                        .get("schema")
                        .and_then(|s| s.get("type"))
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    Some(Parameter {
                        name,
                        location,
                        required,
                        description,
                        schema_type,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Merge path-level and operation-level parameters.
/// Operation-level parameters override path-level by name.
pub(crate) fn merge_parameters(
    path_params: &[Parameter],
    op_params: &[Parameter],
) -> Vec<Parameter> {
    let mut merged: Vec<Parameter> = path_params.to_vec();

    for op_param in op_params {
        if let Some(pos) = merged.iter().position(|p| p.name == op_param.name) {
            merged[pos] = op_param.clone();
        } else {
            merged.push(op_param.clone());
        }
    }

    merged
}

pub(crate) fn parse_method(s: &str) -> Option<HttpMethod> {
    match s {
        "get" => Some(HttpMethod::Get),
        "post" => Some(HttpMethod::Post),
        "put" => Some(HttpMethod::Put),
        "delete" => Some(HttpMethod::Delete),
        "patch" => Some(HttpMethod::Patch),
        _ => None,
    }
}

pub(crate) fn parse_parameter_location(s: &str) -> Option<ParameterLocation> {
    match s {
        "path" => Some(ParameterLocation::Path),
        "query" => Some(ParameterLocation::Query),
        "header" => Some(ParameterLocation::Header),
        "cookie" => Some(ParameterLocation::Cookie),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_spec() -> &'static str {
        r#"{
            "openapi": "3.0.1",
            "info": { "title": "Test API", "version": "1.0.0" },
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
                        "requestBody": {
                            "required": true,
                            "content": { "application/json": {} }
                        },
                        "parameters": []
                    }
                },
                "/rest/api/3/issue/{issueIdOrKey}": {
                    "parameters": [
                        {
                            "name": "issueIdOrKey",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string" },
                            "description": "The ID or key of the issue"
                        }
                    ],
                    "get": {
                        "operationId": "getIssue",
                        "summary": "Get an issue",
                        "tags": ["issues"],
                        "parameters": [
                            {
                                "name": "expand",
                                "in": "query",
                                "required": false,
                                "schema": { "type": "string" },
                                "description": "Fields to expand"
                            }
                        ]
                    },
                    "put": {
                        "operationId": "editIssue",
                        "summary": "Edit an issue",
                        "tags": ["issues"],
                        "deprecated": true,
                        "requestBody": {
                            "required": true,
                            "content": { "application/json": {} }
                        }
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
                            {
                                "name": "jql",
                                "in": "query",
                                "required": false,
                                "schema": { "type": "string" }
                            },
                            {
                                "name": "startAt",
                                "in": "query",
                                "required": false,
                                "schema": { "type": "integer" }
                            },
                            {
                                "name": "maxResults",
                                "in": "query",
                                "required": false,
                                "schema": { "type": "integer" }
                            }
                        ]
                    }
                }
            }
        }"#
    }

    #[test]
    fn parses_minimal_spec() {
        let spec = parse_openapi_v3(minimal_spec()).unwrap();
        assert_eq!(spec.title, "Test API");
        assert_eq!(spec.version, "1.0.0");
        assert_eq!(
            spec.server_url,
            Some("https://example.atlassian.net".into())
        );
    }

    #[test]
    fn extracts_correct_operation_count() {
        let spec = parse_openapi_v3(minimal_spec()).unwrap();
        assert_eq!(spec.operations.len(), 5);
    }

    #[test]
    fn extracts_operation_ids_and_methods() {
        let spec = parse_openapi_v3(minimal_spec()).unwrap();
        let ops: Vec<(&str, &HttpMethod)> = spec
            .operations
            .iter()
            .map(|o| (o.operation_id.as_str(), &o.method))
            .collect();
        assert!(ops.contains(&("createIssue", &HttpMethod::Post)));
        assert!(ops.contains(&("getIssue", &HttpMethod::Get)));
        assert!(ops.contains(&("editIssue", &HttpMethod::Put)));
        assert!(ops.contains(&("deleteIssue", &HttpMethod::Delete)));
        assert!(ops.contains(&("searchForIssuesUsingJql", &HttpMethod::Get)));
    }

    #[test]
    fn preserves_path_templates() {
        let spec = parse_openapi_v3(minimal_spec()).unwrap();
        let get_issue = spec
            .operations
            .iter()
            .find(|o| o.operation_id == "getIssue")
            .unwrap();
        assert_eq!(get_issue.path, "/rest/api/3/issue/{issueIdOrKey}");
    }

    #[test]
    fn extracts_tags() {
        let spec = parse_openapi_v3(minimal_spec()).unwrap();
        assert_eq!(spec.tags.len(), 2);
        assert_eq!(spec.tags[0].name, "issues");
        assert_eq!(spec.tags[0].description, Some("Issue operations".into()));
    }

    #[test]
    fn extracts_parameters_with_location() {
        let spec = parse_openapi_v3(minimal_spec()).unwrap();
        let get_issue = spec
            .operations
            .iter()
            .find(|o| o.operation_id == "getIssue")
            .unwrap();

        // Should have path param (from path-level) + query param (from operation-level)
        assert_eq!(get_issue.parameters.len(), 2);

        let path_param = get_issue
            .parameters
            .iter()
            .find(|p| p.name == "issueIdOrKey")
            .unwrap();
        assert_eq!(path_param.location, ParameterLocation::Path);
        assert!(path_param.required);
        assert_eq!(path_param.schema_type, Some("string".into()));

        let query_param = get_issue
            .parameters
            .iter()
            .find(|p| p.name == "expand")
            .unwrap();
        assert_eq!(query_param.location, ParameterLocation::Query);
        assert!(!query_param.required);
    }

    #[test]
    fn extracts_pagination_parameters() {
        let spec = parse_openapi_v3(minimal_spec()).unwrap();
        let search = spec
            .operations
            .iter()
            .find(|o| o.operation_id == "searchForIssuesUsingJql")
            .unwrap();
        let param_names: Vec<&str> = search.parameters.iter().map(|p| p.name.as_str()).collect();
        assert!(param_names.contains(&"startAt"));
        assert!(param_names.contains(&"maxResults"));
    }

    #[test]
    fn extracts_request_body() {
        let spec = parse_openapi_v3(minimal_spec()).unwrap();
        let create = spec
            .operations
            .iter()
            .find(|o| o.operation_id == "createIssue")
            .unwrap();
        let body = create.request_body.as_ref().unwrap();
        assert!(body.required);
        assert!(body.content_types.contains(&"application/json".to_string()));
    }

    #[test]
    fn detects_deprecated_operations() {
        let spec = parse_openapi_v3(minimal_spec()).unwrap();
        let edit = spec
            .operations
            .iter()
            .find(|o| o.operation_id == "editIssue")
            .unwrap();
        assert!(edit.deprecated);

        let get = spec
            .operations
            .iter()
            .find(|o| o.operation_id == "getIssue")
            .unwrap();
        assert!(!get.deprecated);
    }

    #[test]
    fn rejects_swagger_2() {
        let json = r#"{"swagger": "2.0", "info": {"title": "Test", "version": "1.0"}}"#;
        let result = parse_openapi_v3(json);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(
            err.contains("Missing 'openapi'"),
            "Should reject missing openapi field: {err}"
        );
    }

    #[test]
    fn rejects_openapi_2x_version() {
        let json =
            r#"{"openapi": "2.0", "info": {"title": "Test", "version": "1.0"}, "paths": {}}"#;
        let result = parse_openapi_v3(json);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("3.x only"), "Should reject 2.x: {err}");
    }

    #[test]
    fn skips_operations_without_operation_id() {
        let json = r#"{
            "openapi": "3.0.1",
            "info": { "title": "Test", "version": "1.0" },
            "paths": {
                "/test": {
                    "get": { "operationId": "validOp", "summary": "Valid" },
                    "options": { "summary": "CORS preflight" }
                }
            }
        }"#;
        let spec = parse_openapi_v3(json).unwrap();
        // options is not in our method list, so only get is parsed
        assert_eq!(spec.operations.len(), 1);
        assert_eq!(spec.operations[0].operation_id, "validOp");
    }

    #[test]
    fn skips_get_without_operation_id() {
        let json = r#"{
            "openapi": "3.0.1",
            "info": { "title": "Test", "version": "1.0" },
            "paths": {
                "/test": {
                    "get": { "summary": "No operationId" },
                    "post": { "operationId": "hasId", "summary": "Has ID" }
                }
            }
        }"#;
        let spec = parse_openapi_v3(json).unwrap();
        assert_eq!(spec.operations.len(), 1);
        assert_eq!(spec.operations[0].operation_id, "hasId");
    }

    #[test]
    fn path_level_params_merge_with_operation_level() {
        let json = r#"{
            "openapi": "3.0.1",
            "info": { "title": "Test", "version": "1.0" },
            "paths": {
                "/resource/{id}": {
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string" }, "description": "Path level desc" },
                        { "name": "shared", "in": "query", "required": false, "schema": { "type": "string" } }
                    ],
                    "get": {
                        "operationId": "getResource",
                        "parameters": [
                            { "name": "id", "in": "path", "required": true, "schema": { "type": "integer" }, "description": "Op level desc" },
                            { "name": "extra", "in": "query", "required": false, "schema": { "type": "boolean" } }
                        ]
                    }
                }
            }
        }"#;
        let spec = parse_openapi_v3(json).unwrap();
        let op = &spec.operations[0];

        // Should have 3 params: id (op-level wins), shared (path-level), extra (op-level)
        assert_eq!(op.parameters.len(), 3);

        // id should have op-level description and type
        let id_param = op.parameters.iter().find(|p| p.name == "id").unwrap();
        assert_eq!(id_param.description, Some("Op level desc".into()));
        assert_eq!(id_param.schema_type, Some("integer".into()));

        // shared should be from path-level
        assert!(op.parameters.iter().any(|p| p.name == "shared"));
        // extra should be from op-level
        assert!(op.parameters.iter().any(|p| p.name == "extra"));
    }

    #[test]
    fn http_method_display() {
        assert_eq!(format!("{}", HttpMethod::Get), "GET");
        assert_eq!(format!("{}", HttpMethod::Post), "POST");
        assert_eq!(format!("{}", HttpMethod::Put), "PUT");
        assert_eq!(format!("{}", HttpMethod::Delete), "DELETE");
        assert_eq!(format!("{}", HttpMethod::Patch), "PATCH");
    }
}
