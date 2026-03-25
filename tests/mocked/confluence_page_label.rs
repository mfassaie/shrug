//! Mocked E2E tests for Confluence page label commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_page_label_list_sends_get_v2() {
    let env = MockEnv::new();

    // Label list uses v2 API
    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/pages/1001/labels");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "results": [
                    {"id": "lab1", "name": "important", "prefix": "global"}
                ],
                "_links": {}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "page", "label", "list", "1001",
        ]),
    );

    mock.assert();
    assert!(stdout.contains("important"), "Expected label name: {}", stdout);
}

#[test]
fn test_page_label_create_sends_post_v1() {
    let env = MockEnv::new();

    // Label create uses v1 API (different from list!)
    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/wiki/rest/api/content/1001/label")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!([
                {"prefix": "global", "name": "new-label"}
            ]));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "page", "label", "create", "1001", "new-label",
        ]),
    );

    mock.assert();
    assert!(stdout.contains("new-label"), "Expected label in output: {}", stdout);
}

#[test]
fn test_page_label_delete_sends_delete_v1() {
    let env = MockEnv::new();

    // Label delete uses v1 API
    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/wiki/rest/api/content/1001/label/old-label");
        then.status(204);
    });

    helpers::assert_success(
        env.cmd().args([
            "confluence", "page", "label", "delete", "1001", "old-label",
        ]),
    );

    mock.assert();
}
