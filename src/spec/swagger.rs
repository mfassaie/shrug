use serde_json::Value;

use crate::error::ShrugError;
use crate::spec::model::*;
use crate::spec::parser::{parse_method, parse_parameter_location};

/// Parse a Swagger 2.0 JSON spec into shrug's data model.
pub fn parse_swagger_v2(json: &str) -> Result<ApiSpec, ShrugError> {
    let doc: Value = serde_json::from_str(json)
        .map_err(|e| ShrugError::SpecError(format!("Invalid JSON: {e}")))?;

    // Validate Swagger version
    let version_str = doc
        .get("swagger")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ShrugError::SpecError("Missing 'swagger' version field".into()))?;

    if !version_str.starts_with("2.") {
        return Err(ShrugError::SpecError(format!(
            "Unsupported Swagger version '{}'. This parser supports 2.x only. For OpenAPI 3.x, use the OpenAPI parser.",
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

    // Build server_url from schemes + host + basePath
    let server_url = build_server_url(&doc);

    // Read spec-level consumes (default: ["application/json"])
    let spec_consumes = doc
        .get("consumes")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec!["application/json".to_string()]);

    // Extract tags
    let tags = parse_tags(&doc);

    // Extract operations from paths
    let operations = parse_paths(&doc, &spec_consumes)?;

    Ok(ApiSpec {
        title,
        version,
        server_url,
        tags,
        operations,
    })
}

fn build_server_url(doc: &Value) -> Option<String> {
    let host = doc.get("host").and_then(|v| v.as_str())?;
    let host = host.trim_end_matches('/');

    let base_path = doc.get("basePath").and_then(|v| v.as_str()).unwrap_or("/");
    let base_path = if base_path.starts_with('/') {
        base_path.to_string()
    } else {
        format!("/{base_path}")
    };

    // Prefer "https" from schemes array, fall back to first entry, default to "https"
    let scheme = doc
        .get("schemes")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            let schemes: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
            if schemes.contains(&"https") {
                Some("https")
            } else {
                schemes.first().copied()
            }
        })
        .unwrap_or("https");

    Some(format!("{scheme}://{host}{base_path}"))
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

fn parse_paths(doc: &Value, spec_consumes: &[String]) -> Result<Vec<Operation>, ShrugError> {
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

                // Merge path-level and operation-level parameters
                let op_level_params = op_value
                    .get("parameters")
                    .map(parse_parameters)
                    .unwrap_or_default();
                let all_params = merge_swagger_params(&path_level_params, &op_level_params);

                // Determine consumes for this operation
                let op_consumes = op_value
                    .get("consumes")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect::<Vec<_>>()
                    });

                // Separate body/formData params from regular params, build RequestBody
                let (parameters, request_body) =
                    separate_body_params(&all_params, op_consumes.as_deref(), spec_consumes);

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

/// Merge path-level and operation-level Swagger parameters.
/// Operation-level parameters override path-level by name.
fn merge_swagger_params(
    path_params: &[SwaggerParam],
    op_params: &[SwaggerParam],
) -> Vec<SwaggerParam> {
    let mut merged: Vec<SwaggerParam> = path_params.to_vec();

    for op_param in op_params {
        if let Some(pos) = merged.iter().position(|p| p.name == op_param.name) {
            merged[pos] = op_param.clone();
        } else {
            merged.push(op_param.clone());
        }
    }

    merged
}

/// In Swagger 2.0, "in: body" and "in: formData" params are converted to RequestBody.
/// Regular params (path, query, header) stay in the parameters vec.
/// Multiple formData params produce a single RequestBody.
/// Body and formData are mutually exclusive per Swagger 2.0 spec.
fn separate_body_params(
    all_params: &[SwaggerParam],
    op_consumes: Option<&[String]>,
    spec_consumes: &[String],
) -> (Vec<Parameter>, Option<RequestBody>) {
    let mut regular_params = Vec::new();
    let mut body_param: Option<&SwaggerParam> = None;
    let mut form_data_params: Vec<&SwaggerParam> = Vec::new();

    for param in all_params {
        match param.location.as_str() {
            "body" => body_param = Some(param),
            "formData" => form_data_params.push(param),
            _ => {
                if let Some(loc) = parse_parameter_location(&param.location) {
                    regular_params.push(Parameter {
                        name: param.name.clone(),
                        location: loc,
                        required: param.required,
                        description: param.description.clone(),
                        schema_type: param.schema_type.clone(),
                    });
                }
            }
        }
    }

    let request_body = if let Some(bp) = body_param {
        // "in: body" → RequestBody
        let content_types = op_consumes.unwrap_or(spec_consumes).to_vec();
        Some(RequestBody {
            required: bp.required,
            description: bp.description.clone(),
            content_types,
        })
    } else if !form_data_params.is_empty() {
        // Multiple formData params → single RequestBody
        let any_required = form_data_params.iter().any(|p| p.required);
        let content_types = op_consumes.map(|c| c.to_vec()).unwrap_or_else(|| {
            // For formData, if spec consumes doesn't include form types, default to multipart
            if spec_consumes
                .iter()
                .any(|c| c == "multipart/form-data" || c == "application/x-www-form-urlencoded")
            {
                spec_consumes.to_vec()
            } else {
                vec!["multipart/form-data".to_string()]
            }
        });
        Some(RequestBody {
            required: any_required,
            description: None,
            content_types,
        })
    } else {
        None
    };

    (regular_params, request_body)
}

/// Intermediate representation for Swagger 2.0 parameters before separation.
/// Keeps the raw "in" string so we can distinguish body/formData from regular params.
#[derive(Debug, Clone)]
struct SwaggerParam {
    name: String,
    location: String, // raw "in" value: "path", "query", "header", "body", "formData"
    required: bool,
    description: Option<String>,
    schema_type: Option<String>,
}

fn parse_parameters(params: &Value) -> Vec<SwaggerParam> {
    params
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|p| {
                    let name = p.get("name")?.as_str()?.to_string();
                    let location = p.get("in")?.as_str()?.to_string();
                    let required = p.get("required").and_then(|v| v.as_bool()).unwrap_or(false);
                    let description = p
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    // Swagger 2.0: non-body params have "type" directly,
                    // body params have "schema.type"
                    let schema_type = if location == "body" {
                        p.get("schema")
                            .and_then(|s| s.get("type"))
                            .and_then(|v| v.as_str())
                            .map(String::from)
                    } else {
                        p.get("type").and_then(|v| v.as_str()).map(String::from)
                    };

                    Some(SwaggerParam {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_spec() -> &'static str {
        r#"{
            "swagger": "2.0",
            "info": { "title": "BitBucket API", "version": "2.0" },
            "host": "api.bitbucket.org",
            "basePath": "/2.0",
            "schemes": ["https"],
            "consumes": ["application/json"],
            "tags": [
                { "name": "repositories", "description": "Repository operations" },
                { "name": "pullrequests", "description": "Pull request operations" }
            ],
            "paths": {
                "/repositories/{workspace}/{repo_slug}": {
                    "parameters": [
                        {
                            "name": "workspace",
                            "in": "path",
                            "required": true,
                            "type": "string",
                            "description": "The workspace slug"
                        },
                        {
                            "name": "repo_slug",
                            "in": "path",
                            "required": true,
                            "type": "string",
                            "description": "The repository slug"
                        }
                    ],
                    "get": {
                        "operationId": "getRepository",
                        "summary": "Get a repository",
                        "tags": ["repositories"],
                        "parameters": [
                            {
                                "name": "fields",
                                "in": "query",
                                "required": false,
                                "type": "string",
                                "description": "Fields to include"
                            }
                        ]
                    },
                    "delete": {
                        "operationId": "deleteRepository",
                        "summary": "Delete a repository",
                        "tags": ["repositories"]
                    }
                },
                "/repositories/{workspace}": {
                    "get": {
                        "operationId": "listRepositories",
                        "summary": "List repositories in a workspace",
                        "tags": ["repositories"],
                        "parameters": [
                            {
                                "name": "workspace",
                                "in": "path",
                                "required": true,
                                "type": "string"
                            },
                            {
                                "name": "page",
                                "in": "query",
                                "required": false,
                                "type": "integer"
                            },
                            {
                                "name": "pagelen",
                                "in": "query",
                                "required": false,
                                "type": "integer"
                            }
                        ]
                    },
                    "post": {
                        "operationId": "createRepository",
                        "summary": "Create a repository",
                        "tags": ["repositories"],
                        "parameters": [
                            {
                                "name": "workspace",
                                "in": "path",
                                "required": true,
                                "type": "string"
                            },
                            {
                                "name": "body",
                                "in": "body",
                                "required": true,
                                "schema": { "type": "object" }
                            }
                        ]
                    }
                },
                "/repositories/{workspace}/{repo_slug}/pullrequests": {
                    "get": {
                        "operationId": "listPullRequests",
                        "summary": "List pull requests",
                        "tags": ["pullrequests"],
                        "deprecated": true,
                        "parameters": [
                            {
                                "name": "workspace",
                                "in": "path",
                                "required": true,
                                "type": "string"
                            },
                            {
                                "name": "repo_slug",
                                "in": "path",
                                "required": true,
                                "type": "string"
                            },
                            {
                                "name": "state",
                                "in": "query",
                                "required": false,
                                "type": "string"
                            }
                        ]
                    }
                }
            }
        }"#
    }

    #[test]
    fn parses_minimal_spec() {
        let spec = parse_swagger_v2(minimal_spec()).unwrap();
        assert_eq!(spec.title, "BitBucket API");
        assert_eq!(spec.version, "2.0");
        assert_eq!(
            spec.server_url,
            Some("https://api.bitbucket.org/2.0".into())
        );
    }

    #[test]
    fn extracts_correct_operation_count() {
        let spec = parse_swagger_v2(minimal_spec()).unwrap();
        assert_eq!(spec.operations.len(), 5);
    }

    #[test]
    fn extracts_operation_ids_and_methods() {
        let spec = parse_swagger_v2(minimal_spec()).unwrap();
        let ops: Vec<(&str, &HttpMethod)> = spec
            .operations
            .iter()
            .map(|o| (o.operation_id.as_str(), &o.method))
            .collect();
        assert!(ops.contains(&("getRepository", &HttpMethod::Get)));
        assert!(ops.contains(&("deleteRepository", &HttpMethod::Delete)));
        assert!(ops.contains(&("listRepositories", &HttpMethod::Get)));
        assert!(ops.contains(&("createRepository", &HttpMethod::Post)));
        assert!(ops.contains(&("listPullRequests", &HttpMethod::Get)));
    }

    #[test]
    fn preserves_path_templates() {
        let spec = parse_swagger_v2(minimal_spec()).unwrap();
        let get_repo = spec
            .operations
            .iter()
            .find(|o| o.operation_id == "getRepository")
            .unwrap();
        assert_eq!(get_repo.path, "/repositories/{workspace}/{repo_slug}");
    }

    #[test]
    fn extracts_tags() {
        let spec = parse_swagger_v2(minimal_spec()).unwrap();
        assert_eq!(spec.tags.len(), 2);
        assert_eq!(spec.tags[0].name, "repositories");
        assert_eq!(
            spec.tags[0].description,
            Some("Repository operations".into())
        );
    }

    #[test]
    fn extracts_parameters_with_location() {
        let spec = parse_swagger_v2(minimal_spec()).unwrap();
        let get_repo = spec
            .operations
            .iter()
            .find(|o| o.operation_id == "getRepository")
            .unwrap();

        // Should have path params (from path-level) + query param (from operation-level)
        assert_eq!(get_repo.parameters.len(), 3);

        let path_param = get_repo
            .parameters
            .iter()
            .find(|p| p.name == "workspace")
            .unwrap();
        assert_eq!(path_param.location, ParameterLocation::Path);
        assert!(path_param.required);
        assert_eq!(path_param.schema_type, Some("string".into()));

        let query_param = get_repo
            .parameters
            .iter()
            .find(|p| p.name == "fields")
            .unwrap();
        assert_eq!(query_param.location, ParameterLocation::Query);
        assert!(!query_param.required);
    }

    #[test]
    fn extracts_pagination_parameters() {
        let spec = parse_swagger_v2(minimal_spec()).unwrap();
        let list = spec
            .operations
            .iter()
            .find(|o| o.operation_id == "listRepositories")
            .unwrap();
        let param_names: Vec<&str> = list.parameters.iter().map(|p| p.name.as_str()).collect();
        assert!(param_names.contains(&"page"));
        assert!(param_names.contains(&"pagelen"));
    }

    #[test]
    fn body_param_converts_to_request_body() {
        let spec = parse_swagger_v2(minimal_spec()).unwrap();
        let create = spec
            .operations
            .iter()
            .find(|o| o.operation_id == "createRepository")
            .unwrap();
        let body = create.request_body.as_ref().unwrap();
        assert!(body.required);
        assert!(body.content_types.contains(&"application/json".to_string()));

        // Body param should NOT appear in parameters vec
        assert!(
            !create.parameters.iter().any(|p| p.name == "body"),
            "body param should be excluded from parameters vec"
        );
    }

    #[test]
    fn detects_deprecated_operations() {
        let spec = parse_swagger_v2(minimal_spec()).unwrap();
        let list_prs = spec
            .operations
            .iter()
            .find(|o| o.operation_id == "listPullRequests")
            .unwrap();
        assert!(list_prs.deprecated);

        let get_repo = spec
            .operations
            .iter()
            .find(|o| o.operation_id == "getRepository")
            .unwrap();
        assert!(!get_repo.deprecated);
    }

    #[test]
    fn rejects_non_swagger_2() {
        let json = r#"{"swagger": "3.0", "info": {"title": "Test", "version": "1.0"}}"#;
        let result = parse_swagger_v2(json);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("2.x only"), "Should reject non-2.x: {err}");
    }

    #[test]
    fn rejects_missing_swagger_field() {
        let json = r#"{"openapi": "3.0.1", "info": {"title": "Test", "version": "1.0"}}"#;
        let result = parse_swagger_v2(json);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(
            err.contains("Missing 'swagger'"),
            "Should reject missing swagger field: {err}"
        );
    }

    #[test]
    fn skips_operations_without_operation_id() {
        let json = r#"{
            "swagger": "2.0",
            "info": { "title": "Test", "version": "1.0" },
            "host": "example.com",
            "paths": {
                "/test": {
                    "get": { "operationId": "validOp", "summary": "Valid" },
                    "post": { "summary": "No operationId" }
                }
            }
        }"#;
        let spec = parse_swagger_v2(json).unwrap();
        assert_eq!(spec.operations.len(), 1);
        assert_eq!(spec.operations[0].operation_id, "validOp");
    }

    #[test]
    fn path_level_params_merge_with_operation_level() {
        let json = r#"{
            "swagger": "2.0",
            "info": { "title": "Test", "version": "1.0" },
            "host": "example.com",
            "paths": {
                "/resource/{id}": {
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "type": "string", "description": "Path level desc" },
                        { "name": "shared", "in": "query", "required": false, "type": "string" }
                    ],
                    "get": {
                        "operationId": "getResource",
                        "parameters": [
                            { "name": "id", "in": "path", "required": true, "type": "integer", "description": "Op level desc" },
                            { "name": "extra", "in": "query", "required": false, "type": "boolean" }
                        ]
                    }
                }
            }
        }"#;
        let spec = parse_swagger_v2(json).unwrap();
        let op = &spec.operations[0];

        // Should have 3 params: id (op-level wins), shared (path-level), extra (op-level)
        assert_eq!(op.parameters.len(), 3);

        // id should have op-level description and type
        let id_param = op.parameters.iter().find(|p| p.name == "id").unwrap();
        assert_eq!(id_param.description, Some("Op level desc".into()));
        assert_eq!(id_param.schema_type, Some("integer".into()));

        assert!(op.parameters.iter().any(|p| p.name == "shared"));
        assert!(op.parameters.iter().any(|p| p.name == "extra"));
    }

    #[test]
    fn host_basepath_to_server_url() {
        let json = r#"{
            "swagger": "2.0",
            "info": { "title": "Test", "version": "1.0" },
            "host": "api.example.com",
            "basePath": "/v2",
            "paths": {}
        }"#;
        let spec = parse_swagger_v2(json).unwrap();
        assert_eq!(spec.server_url, Some("https://api.example.com/v2".into()));
    }

    #[test]
    fn missing_host_produces_none_server_url() {
        let json = r#"{
            "swagger": "2.0",
            "info": { "title": "Test", "version": "1.0" },
            "paths": {}
        }"#;
        let spec = parse_swagger_v2(json).unwrap();
        assert_eq!(spec.server_url, None);
    }

    #[test]
    fn schemes_array_prefers_https() {
        let json = r#"{
            "swagger": "2.0",
            "info": { "title": "Test", "version": "1.0" },
            "host": "api.example.com",
            "basePath": "/v1",
            "schemes": ["http", "https"],
            "paths": {}
        }"#;
        let spec = parse_swagger_v2(json).unwrap();
        assert_eq!(spec.server_url, Some("https://api.example.com/v1".into()));
    }

    #[test]
    fn schemes_array_falls_back_to_first() {
        let json = r#"{
            "swagger": "2.0",
            "info": { "title": "Test", "version": "1.0" },
            "host": "api.example.com",
            "basePath": "/v1",
            "schemes": ["http"],
            "paths": {}
        }"#;
        let spec = parse_swagger_v2(json).unwrap();
        assert_eq!(spec.server_url, Some("http://api.example.com/v1".into()));
    }

    #[test]
    fn schemes_missing_defaults_to_https() {
        let json = r#"{
            "swagger": "2.0",
            "info": { "title": "Test", "version": "1.0" },
            "host": "api.example.com",
            "paths": {}
        }"#;
        let spec = parse_swagger_v2(json).unwrap();
        assert_eq!(spec.server_url, Some("https://api.example.com/".into()));
    }

    #[test]
    fn basepath_normalization_no_double_slash() {
        let json = r#"{
            "swagger": "2.0",
            "info": { "title": "Test", "version": "1.0" },
            "host": "api.example.com/",
            "basePath": "/v2",
            "paths": {}
        }"#;
        let spec = parse_swagger_v2(json).unwrap();
        assert_eq!(spec.server_url, Some("https://api.example.com/v2".into()));
    }

    #[test]
    fn basepath_missing_leading_slash_normalized() {
        let json = r#"{
            "swagger": "2.0",
            "info": { "title": "Test", "version": "1.0" },
            "host": "api.example.com",
            "basePath": "v2",
            "paths": {}
        }"#;
        let spec = parse_swagger_v2(json).unwrap();
        assert_eq!(spec.server_url, Some("https://api.example.com/v2".into()));
    }

    #[test]
    fn formdata_params_convert_to_request_body() {
        let json = r#"{
            "swagger": "2.0",
            "info": { "title": "Test", "version": "1.0" },
            "host": "example.com",
            "paths": {
                "/upload": {
                    "post": {
                        "operationId": "uploadFile",
                        "consumes": ["multipart/form-data"],
                        "parameters": [
                            { "name": "file", "in": "formData", "required": true, "type": "file" },
                            { "name": "message", "in": "formData", "required": false, "type": "string" }
                        ]
                    }
                }
            }
        }"#;
        let spec = parse_swagger_v2(json).unwrap();
        let upload = &spec.operations[0];

        // formData params should NOT be in parameters vec
        assert!(
            upload.parameters.is_empty(),
            "formData params should be excluded from parameters: {:?}",
            upload.parameters
        );

        // Should have a single RequestBody
        let body = upload.request_body.as_ref().unwrap();
        assert!(
            body.required,
            "required should be true because 'file' param is required"
        );
        assert!(body
            .content_types
            .contains(&"multipart/form-data".to_string()));
    }

    #[test]
    fn multiple_formdata_params_produce_single_request_body() {
        let json = r#"{
            "swagger": "2.0",
            "info": { "title": "Test", "version": "1.0" },
            "host": "example.com",
            "paths": {
                "/upload": {
                    "post": {
                        "operationId": "uploadWithMetadata",
                        "consumes": ["multipart/form-data"],
                        "parameters": [
                            { "name": "file", "in": "formData", "required": true, "type": "file" },
                            { "name": "message", "in": "formData", "required": false, "type": "string" },
                            { "name": "branch", "in": "formData", "required": false, "type": "string" },
                            { "name": "repo_slug", "in": "path", "required": true, "type": "string" }
                        ]
                    }
                }
            }
        }"#;
        let spec = parse_swagger_v2(json).unwrap();
        let upload = &spec.operations[0];

        // Only path param should remain in parameters
        assert_eq!(upload.parameters.len(), 1);
        assert_eq!(upload.parameters[0].name, "repo_slug");

        // Single RequestBody from 3 formData params
        let body = upload.request_body.as_ref().unwrap();
        assert!(
            body.required,
            "ANY required formData param makes body required"
        );
        assert!(body
            .content_types
            .contains(&"multipart/form-data".to_string()));
    }

    #[test]
    fn spec_level_consumes_inherited_by_body_param() {
        let json = r#"{
            "swagger": "2.0",
            "info": { "title": "Test", "version": "1.0" },
            "host": "example.com",
            "consumes": ["application/xml", "application/json"],
            "paths": {
                "/resource": {
                    "post": {
                        "operationId": "createResource",
                        "parameters": [
                            { "name": "body", "in": "body", "required": true, "schema": { "type": "object" } }
                        ]
                    }
                }
            }
        }"#;
        let spec = parse_swagger_v2(json).unwrap();
        let create = &spec.operations[0];
        let body = create.request_body.as_ref().unwrap();

        // Should inherit spec-level consumes, not hardcoded default
        assert_eq!(body.content_types.len(), 2);
        assert!(body.content_types.contains(&"application/xml".to_string()));
        assert!(body.content_types.contains(&"application/json".to_string()));
    }

    #[test]
    fn operation_consumes_overrides_spec_consumes() {
        let json = r#"{
            "swagger": "2.0",
            "info": { "title": "Test", "version": "1.0" },
            "host": "example.com",
            "consumes": ["application/json"],
            "paths": {
                "/resource": {
                    "post": {
                        "operationId": "createResource",
                        "consumes": ["application/xml"],
                        "parameters": [
                            { "name": "body", "in": "body", "required": true, "schema": { "type": "object" } }
                        ]
                    }
                }
            }
        }"#;
        let spec = parse_swagger_v2(json).unwrap();
        let create = &spec.operations[0];
        let body = create.request_body.as_ref().unwrap();

        // Operation-level consumes should override spec-level
        assert_eq!(body.content_types, vec!["application/xml".to_string()]);
    }

    #[test]
    fn formdata_inherits_spec_consumes_when_form_type() {
        let json = r#"{
            "swagger": "2.0",
            "info": { "title": "Test", "version": "1.0" },
            "host": "example.com",
            "consumes": ["multipart/form-data"],
            "paths": {
                "/upload": {
                    "post": {
                        "operationId": "upload",
                        "parameters": [
                            { "name": "file", "in": "formData", "required": true, "type": "file" }
                        ]
                    }
                }
            }
        }"#;
        let spec = parse_swagger_v2(json).unwrap();
        let upload = &spec.operations[0];
        let body = upload.request_body.as_ref().unwrap();

        // Spec-level consumes includes form type, so use it
        assert_eq!(body.content_types, vec!["multipart/form-data".to_string()]);
    }

    #[test]
    fn swagger_type_directly_on_param() {
        let json = r#"{
            "swagger": "2.0",
            "info": { "title": "Test", "version": "1.0" },
            "host": "example.com",
            "paths": {
                "/test": {
                    "get": {
                        "operationId": "testOp",
                        "parameters": [
                            { "name": "count", "in": "query", "required": false, "type": "integer" }
                        ]
                    }
                }
            }
        }"#;
        let spec = parse_swagger_v2(json).unwrap();
        let op = &spec.operations[0];
        let param = &op.parameters[0];
        assert_eq!(param.schema_type, Some("integer".into()));
    }
}
