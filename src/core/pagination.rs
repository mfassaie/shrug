//! Unified pagination helpers for offset-based and cursor-based APIs.
//!
//! Jira uses offset-based pagination (startAt/maxResults).
//! Confluence v2 uses cursor-based pagination (_links.next).
//! These helpers work with serde_json::Value responses.

use serde_json::Value;

/// Count the number of results in a paginated response.
///
/// Tries known array field names: issues (Jira search), values (Jira PageBean),
/// results (Confluence), records (Jira audit). Falls back to top-level array.
pub fn count_results(json: &Value) -> u32 {
    for key in &["issues", "values", "results", "records"] {
        if let Some(arr) = json.get(key).and_then(|v| v.as_array()) {
            return arr.len() as u32;
        }
    }
    if let Some(arr) = json.as_array() {
        return arr.len() as u32;
    }
    0
}

/// Extract the results array from a paginated response.
///
/// Returns the array under the first matching key, or the top-level array.
pub fn extract_results(json: &Value) -> Option<&Vec<Value>> {
    for key in &["issues", "values", "results", "records"] {
        if let Some(arr) = json.get(key).and_then(|v| v.as_array()) {
            return Some(arr);
        }
    }
    json.as_array()
}

/// Check if there are more pages for offset-based pagination.
///
/// Uses the `total` field if present. Otherwise assumes more pages if results were returned.
pub fn has_more_offset(json: &Value, current_offset: u64, page_count: u32) -> bool {
    if let Some(total) = json.get("total").and_then(|v| v.as_u64()) {
        return (current_offset + page_count as u64) < total;
    }
    // Jira's isLast field
    if let Some(is_last) = json.get("isLast").and_then(|v| v.as_bool()) {
        return !is_last;
    }
    page_count > 0
}

/// Check if there are more pages for link-based pagination.
///
/// Looks for a "next" field (Confluence, Bitbucket).
pub fn has_more_link(json: &Value) -> bool {
    json.get("next").and_then(|v| v.as_str()).is_some()
        || json.pointer("/_links/next").and_then(|v| v.as_str()).is_some()
}

/// Extract cursor/token for cursor-based pagination.
///
/// Tries: nextPageToken (Jira v2), cursor, _links.next (Confluence).
pub fn extract_cursor(json: &Value) -> Option<String> {
    if let Some(c) = json.get("nextPageToken").and_then(|v| v.as_str()) {
        return Some(c.to_string());
    }
    if let Some(c) = json.get("cursor").and_then(|v| v.as_str()) {
        return Some(c.to_string());
    }
    if let Some(c) = json.pointer("/_links/next").and_then(|v| v.as_str()) {
        return Some(c.to_string());
    }
    None
}

/// Extract the total count from a paginated response, if available.
pub fn extract_total(json: &Value) -> Option<u64> {
    json.get("total")
        .or_else(|| json.get("totalSize"))
        .and_then(|v| v.as_u64())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_count_results_issues() {
        let json = json!({"issues": [{"key": "A"}, {"key": "B"}], "total": 5});
        assert_eq!(count_results(&json), 2);
    }

    #[test]
    fn test_count_results_values() {
        let json = json!({"values": [1, 2, 3], "total": 10});
        assert_eq!(count_results(&json), 3);
    }

    #[test]
    fn test_count_results_array() {
        let json = json!([{"id": 1}, {"id": 2}]);
        assert_eq!(count_results(&json), 2);
    }

    #[test]
    fn test_count_results_empty() {
        let json = json!({"something": "else"});
        assert_eq!(count_results(&json), 0);
    }

    #[test]
    fn test_extract_results_issues() {
        let json = json!({"issues": [{"key": "A"}]});
        let results = extract_results(&json).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_has_more_offset_with_total() {
        let json = json!({"issues": [{"key": "A"}], "total": 100, "startAt": 0});
        assert!(has_more_offset(&json, 0, 50));
        assert!(!has_more_offset(&json, 50, 50));
    }

    #[test]
    fn test_has_more_offset_with_is_last() {
        let json = json!({"values": [1], "isLast": false});
        assert!(has_more_offset(&json, 0, 1));

        let json = json!({"values": [1], "isLast": true});
        assert!(!has_more_offset(&json, 0, 1));
    }

    #[test]
    fn test_has_more_link() {
        let json = json!({"results": [], "next": "/wiki/rest/api/search?start=25"});
        assert!(has_more_link(&json));

        let json = json!({"results": []});
        assert!(!has_more_link(&json));
    }

    #[test]
    fn test_has_more_link_atlassian_links() {
        let json = json!({"results": [], "_links": {"next": "/wiki/api/v2/pages?cursor=abc"}});
        assert!(has_more_link(&json));
    }

    #[test]
    fn test_extract_cursor() {
        let json = json!({"nextPageToken": "abc123"});
        assert_eq!(extract_cursor(&json), Some("abc123".to_string()));
    }

    #[test]
    fn test_extract_cursor_links() {
        let json = json!({"_links": {"next": "/api/v2?cursor=xyz"}});
        assert_eq!(
            extract_cursor(&json),
            Some("/api/v2?cursor=xyz".to_string())
        );
    }

    #[test]
    fn test_extract_cursor_none() {
        let json = json!({"data": []});
        assert_eq!(extract_cursor(&json), None);
    }

    #[test]
    fn test_extract_total() {
        assert_eq!(extract_total(&json!({"total": 42})), Some(42));
        assert_eq!(extract_total(&json!({"totalSize": 10})), Some(10));
        assert_eq!(extract_total(&json!({"data": []})), None);
    }
}
