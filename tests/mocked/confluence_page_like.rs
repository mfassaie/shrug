//! Mocked E2E tests for Confluence page like commands (read-only).

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_page_like_view_sends_get_count() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/pages/1001/likes/count");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "count": 5
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "page", "like", "view", "1001",
        ]),
    );

    mock.assert();
    assert!(stdout.contains("5"), "Expected like count: {}", stdout);
}

#[test]
fn test_page_like_list_sends_get_users() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/pages/1001/likes/users");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "results": [
                    {"accountId": "user1", "publicName": "Alice"}
                ],
                "_links": {}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "page", "like", "list", "1001",
        ]),
    );

    mock.assert();
    assert!(stdout.contains("Alice") || stdout.contains("user1"), "Expected user: {}", stdout);
}
