use std::fmt;

use serde::{Deserialize, Serialize};

/// The parsed API specification — everything shrug needs for CLI generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSpec {
    pub title: String,
    pub version: String,
    pub server_url: Option<String>,
    pub tags: Vec<Tag>,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub operation_id: String,
    pub method: HttpMethod,
    pub path: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub parameters: Vec<Parameter>,
    pub request_body: Option<RequestBody>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Patch => write!(f, "PATCH"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub location: ParameterLocation,
    pub required: bool,
    pub description: Option<String>,
    pub schema_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterLocation {
    Path,
    Query,
    Header,
    Cookie,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    pub required: bool,
    pub description: Option<String>,
    pub content_types: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_method_display_get() {
        assert_eq!(HttpMethod::Get.to_string(), "GET");
    }

    #[test]
    fn http_method_display_post() {
        assert_eq!(HttpMethod::Post.to_string(), "POST");
    }

    #[test]
    fn http_method_display_put() {
        assert_eq!(HttpMethod::Put.to_string(), "PUT");
    }

    #[test]
    fn http_method_display_delete() {
        assert_eq!(HttpMethod::Delete.to_string(), "DELETE");
    }

    #[test]
    fn http_method_display_patch() {
        assert_eq!(HttpMethod::Patch.to_string(), "PATCH");
    }

    #[test]
    fn parameter_location_variants() {
        let locations = [
            ParameterLocation::Path,
            ParameterLocation::Query,
            ParameterLocation::Header,
            ParameterLocation::Cookie,
        ];
        assert_eq!(locations.len(), 4);
        assert_ne!(ParameterLocation::Path, ParameterLocation::Query);
    }

    #[test]
    fn api_spec_serialization_roundtrip() {
        let spec = ApiSpec {
            title: "Test".to_string(),
            version: "1.0".to_string(),
            server_url: Some("https://example.com".to_string()),
            tags: vec![Tag {
                name: "test".to_string(),
                description: Some("desc".to_string()),
            }],
            operations: vec![],
        };
        let json = serde_json::to_string(&spec).unwrap();
        let parsed: ApiSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.title, "Test");
        assert_eq!(parsed.version, "1.0");
        assert_eq!(parsed.tags.len(), 1);
    }

    #[test]
    fn operation_with_all_fields() {
        let op = Operation {
            operation_id: "getUser".to_string(),
            method: HttpMethod::Get,
            path: "/users/{id}".to_string(),
            summary: Some("Get user".to_string()),
            description: Some("Fetches a user by ID".to_string()),
            tags: vec!["users".to_string()],
            deprecated: false,
            parameters: vec![Parameter {
                name: "id".to_string(),
                location: ParameterLocation::Path,
                required: true,
                description: Some("User ID".to_string()),
                schema_type: Some("string".to_string()),
            }],
            request_body: None,
        };
        assert_eq!(op.operation_id, "getUser");
        assert_eq!(op.parameters.len(), 1);
        assert!(op.parameters[0].required);
    }
}
