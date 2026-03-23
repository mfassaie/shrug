use crate::spec::registry::Product;

/// Endpoint-specific behaviour not captured in OpenAPI specs.
///
/// Some Atlassian API endpoints require headers, content types, or other
/// request modifications that the spec does not declare. The quirks registry
/// provides a static lookup keyed by (Product, operationId).
pub struct Quirk {
    /// Extra headers to add to the request.
    pub extra_headers: &'static [(&'static str, &'static str)],
    /// Human-readable description (used in debug logging).
    pub description: &'static str,
}

// --- Static quirk definitions ---

static CSRF_BYPASS: Quirk = Quirk {
    extra_headers: &[("X-Atlassian-Token", "no-check")],
    description: "CSRF bypass for file attachment",
};

/// Look up a quirk for a given product and operationId.
///
/// Returns `None` if no quirk is registered for the operation.
pub fn get_quirk(product: &Product, operation_id: &str) -> Option<&'static Quirk> {
    match (product, operation_id) {
        // Jira Platform
        (Product::Jira, "addAttachment") => Some(&CSRF_BYPASS),

        // Confluence
        (Product::Confluence, "createAttachment") => Some(&CSRF_BYPASS),
        (Product::Confluence, "updateAttachment") => Some(&CSRF_BYPASS),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jira_add_attachment_has_csrf_quirk() {
        let quirk = get_quirk(&Product::Jira, "addAttachment");
        assert!(quirk.is_some(), "Jira addAttachment should have a quirk");
        let q = quirk.unwrap();
        assert!(
            q.extra_headers
                .iter()
                .any(|(k, v)| *k == "X-Atlassian-Token" && *v == "no-check"),
            "Should include X-Atlassian-Token: no-check"
        );
    }

    #[test]
    fn confluence_create_attachment_has_csrf_quirk() {
        let quirk = get_quirk(&Product::Confluence, "createAttachment");
        assert!(quirk.is_some());
        assert_eq!(quirk.unwrap().extra_headers.len(), 1);
    }

    #[test]
    fn confluence_update_attachment_has_csrf_quirk() {
        let quirk = get_quirk(&Product::Confluence, "updateAttachment");
        assert!(quirk.is_some());
    }

    #[test]
    fn unknown_operation_returns_none() {
        assert!(get_quirk(&Product::Jira, "getIssue").is_none());
        assert!(get_quirk(&Product::Jira, "createIssue").is_none());
    }

    #[test]
    fn correct_operation_wrong_product_returns_none() {
        // createAttachment is Confluence, not Jira
        assert!(get_quirk(&Product::Jira, "createAttachment").is_none());
    }

    #[test]
    fn quirk_has_description() {
        let quirk = get_quirk(&Product::Jira, "addAttachment").unwrap();
        assert!(
            !quirk.description.is_empty(),
            "Quirk should have a non-empty description"
        );
    }

    #[test]
    fn registered_operation_ids_exist_in_bundled_specs() {
        use crate::spec::parse_spec;
        use crate::spec::registry::bundled_spec;

        let registered: &[(Product, &str)] = &[
            (Product::Jira, "addAttachment"),
            (Product::Confluence, "createAttachment"),
            (Product::Confluence, "updateAttachment"),
        ];

        for (product, op_id) in registered {
            let json = bundled_spec(product);
            let spec = parse_spec(json).unwrap_or_else(|e| {
                panic!("Failed to parse bundled spec for {:?}: {}", product, e);
            });

            if spec.operations.is_empty() {
                eprintln!(
                    "SKIP: Bundled spec for {:?} has 0 operations (minimal fixture), \
                     cannot verify operationId '{}'",
                    product, op_id
                );
                continue;
            }

            let found = spec.operations.iter().any(|op| op.operation_id == *op_id);

            assert!(
                found,
                "Registered quirk operationId '{}' not found in bundled spec for {:?}.",
                op_id, product
            );
        }
    }
}
