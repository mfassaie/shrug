//! Mocked E2E tests for Jira label commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_label_list_sends_get() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/label");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "values": ["bug", "feature", "urgent"],
                "total": 3,
                "isLast": true
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira", "label", "list"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    let arr = json.as_array().expect("Expected JSON array");
    assert!(!arr.is_empty(), "Expected non-empty label list");
}
