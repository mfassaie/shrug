//! Mocked E2E tests for Confluence custom content commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_custom_content_list_sends_get_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/custom-content");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "results": [
                    {
                        "id": "6001",
                        "type": "ac:my-app:content",
                        "title": "Custom Widget",
                        "status": "current"
                    }
                ],
                "_links": {}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "custom-content", "list",
            "--type", "ac:my-app:content",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    let arr = json.as_array().expect("Expected JSON array");
    assert_eq!(arr.len(), 1);
}

#[test]
fn test_custom_content_create_sends_post_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/wiki/api/v2/custom-content")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "6002",
                "type": "ac:my-app:content",
                "title": "New Widget",
                "status": "current"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "custom-content", "create",
            "--type", "ac:my-app:content",
            "--title", "New Widget",
            "--space-id", "12345",
            "--body", "<p>Widget content</p>",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "6002");
}

#[test]
fn test_custom_content_view_sends_get_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/custom-content/6001");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "6001",
                "type": "ac:my-app:content",
                "title": "Custom Widget",
                "status": "current",
                "version": {"number": 1}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "confluence", "custom-content", "view", "6001"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "6001");
}

#[test]
fn test_custom_content_edit_fetches_version_then_puts() {
    let env = MockEnv::new();

    // GET to fetch current version
    let get_mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/custom-content/6001");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "6001",
                "type": "ac:my-app:content",
                "title": "Custom Widget",
                "status": "current",
                "version": {"number": 1}
            }));
    });

    // PUT with incremented version
    let put_mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/wiki/api/v2/custom-content/6001")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "6001",
                "title": "Updated Widget",
                "status": "current",
                "version": {"number": 2}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "custom-content", "edit", "6001",
            "--title", "Updated Widget",
        ]),
    );

    get_mock.assert();
    put_mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["status"], "updated");
}

#[test]
fn test_custom_content_delete_sends_delete_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/wiki/api/v2/custom-content/6001");
        then.status(204);
    });

    helpers::assert_success(
        env.cmd()
            .args(["confluence", "custom-content", "delete", "6001", "--yes"]),
    );

    mock.assert();
}
