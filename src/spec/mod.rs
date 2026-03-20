pub mod analysis;
pub mod cache;
pub mod model;
pub mod parser;
pub mod registry;
pub mod swagger;

pub use analysis::{build_url, detect_pagination, PaginationStyle};
pub use cache::SpecCache;
pub use parser::parse_openapi_v3;
pub use registry::{Product, SpecLoader};
pub use swagger::parse_swagger_v2;

use crate::error::ShrugError;
use model::ApiSpec;

/// Auto-detect spec format and parse accordingly.
/// Supports OpenAPI 3.x ("openapi" field) and Swagger 2.0 ("swagger" field).
pub fn parse_spec(json: &str) -> Result<ApiSpec, ShrugError> {
    let doc: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| ShrugError::SpecError(format!("Invalid JSON: {e}")))?;

    if let Some(v) = doc.get("openapi").and_then(|v| v.as_str()) {
        if v.starts_with("3.") {
            return parse_openapi_v3(json);
        }
    }

    if let Some(v) = doc.get("swagger").and_then(|v| v.as_str()) {
        if v.starts_with("2.") {
            return parse_swagger_v2(json);
        }
    }

    Err(ShrugError::SpecError(
        "Unrecognized spec format: expected OpenAPI 3.x or Swagger 2.0".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_spec_routes_openapi_v3() {
        let json = r#"{
            "openapi": "3.0.1",
            "info": { "title": "V3 API", "version": "1.0" },
            "paths": {}
        }"#;
        let spec = parse_spec(json).unwrap();
        assert_eq!(spec.title, "V3 API");
    }

    #[test]
    fn parse_spec_routes_swagger_v2() {
        let json = r#"{
            "swagger": "2.0",
            "info": { "title": "V2 API", "version": "1.0" },
            "host": "example.com",
            "paths": {}
        }"#;
        let spec = parse_spec(json).unwrap();
        assert_eq!(spec.title, "V2 API");
    }

    #[test]
    fn parse_spec_rejects_unknown_format() {
        let json = r#"{"info": {"title": "Test", "version": "1.0"}}"#;
        let result = parse_spec(json);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(
            err.contains("Unrecognized spec format"),
            "Should reject unknown format: {err}"
        );
    }

    #[test]
    fn parse_spec_rejects_unsupported_versions() {
        // OpenAPI 2.x (not valid for v3 parser, no swagger field for v2 parser)
        let json = r#"{"openapi": "2.0", "info": {"title": "Test", "version": "1.0"}}"#;
        let result = parse_spec(json);
        assert!(result.is_err());
    }
}
