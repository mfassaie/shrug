//! Mocked E2E tests for Confluence page commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_page_list_sends_get_v2_pages() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/pages");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "results": [
                    {
                        "id": "1001",
                        "title": "Getting Started",
                        "status": "current",
                        "spaceId": "12345"
                    },
                    {
                        "id": "1002",
                        "title": "API Reference",
                        "status": "current",
                        "spaceId": "12345"
                    }
                ],
                "_links": {}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(&["--output", "json", "confluence", "page", "list"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    let arr = json.as_array().expect("Expected JSON array of pages");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["title"], "Getting Started");
}

#[test]
fn test_page_create_sends_post_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/wiki/api/v2/pages")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "2001",
                "title": "New Page",
                "status": "current",
                "spaceId": "12345",
                "version": {"number": 1}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args(&[
            "--output", "json",
            "confluence", "page", "create",
            "--title", "New Page",
            "--space-id", "12345",
            "--body", "<p>Hello World</p>",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "2001");
    assert_eq!(json["title"], "New Page");
}

#[test]
fn test_page_view_sends_get_v2_by_id() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/pages/1001");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "1001",
                "title": "Getting Started",
                "status": "current",
                "spaceId": "12345",
                "body": {
                    "storage": {"value": "<p>Welcome</p>", "representation": "storage"}
                },
                "version": {"number": 1}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(&["--output", "json", "confluence", "page", "view", "1001"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "1001");
    assert_eq!(json["title"], "Getting Started");
}

#[test]
fn test_page_delete_sends_delete_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/wiki/api/v2/pages/1001");
        then.status(204);
    });

    helpers::assert_success(
        env.cmd()
            .args(&["confluence", "page", "delete", "1001", "--yes"]),
    );

    mock.assert();
}

#[test]
fn test_page_edit_fetches_version_then_puts() {
    let env = MockEnv::new();

    // The edit command first GETs the current page to read its version number
    let get_mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/pages/1001");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "1001",
                "title": "Getting Started",
                "status": "current",
                "spaceId": "12345",
                "version": {"number": 3}
            }));
    });

    // Then PUTs the updated page with version incremented to 4
    let put_mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/wiki/api/v2/pages/1001")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "1001",
                "title": "Getting Started (Revised)",
                "status": "current",
                "spaceId": "12345",
                "version": {"number": 4}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args(&[
            "--output", "json",
            "confluence", "page", "edit", "1001",
            "--title", "Getting Started (Revised)",
            "--body", "<p>Updated content</p>",
        ]),
    );

    get_mock.assert();
    put_mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["status"], "updated");
}
