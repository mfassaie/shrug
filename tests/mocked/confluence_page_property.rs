//! Mocked E2E tests for Confluence page property commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_page_property_list_sends_get_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/pages/1001/properties");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "results": [
                    {"id": "pp001", "key": "editor", "value": {"version": "v2"}}
                ],
                "_links": {}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "page", "property", "list", "1001",
        ]),
    );

    mock.assert();
    assert!(stdout.contains("editor"), "Expected property key: {}", stdout);
}

#[test]
fn test_page_property_create_sends_post_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/wiki/api/v2/pages/1001/properties")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "pp002",
                "key": "status",
                "version": {"number": 1}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "page", "property", "create", "1001",
            "--key", "status",
            "--value", r#"{"draft":true}"#,
        ]),
    );

    mock.assert();
    assert!(stdout.contains("pp002"), "Expected property ID: {}", stdout);
}

#[test]
fn test_page_property_view_sends_get_v2_by_id() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/pages/1001/properties/pp001");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "pp001",
                "key": "editor",
                "value": {"version": "v2"},
                "version": {"number": 1}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "page", "property", "view", "1001", "pp001",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "pp001");
}

#[test]
fn test_page_property_edit_fetches_version_then_puts() {
    let env = MockEnv::new();

    let get_mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/pages/1001/properties/pp001");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "pp001",
                "key": "editor",
                "value": {"version": "v2"},
                "version": {"number": 1}
            }));
    });

    let put_mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/wiki/api/v2/pages/1001/properties/pp001")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "pp001",
                "version": {"number": 2}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "confluence", "page", "property", "edit", "1001", "pp001",
            "--value", r#"{"version":"v3"}"#,
        ]),
    );

    get_mock.assert();
    put_mock.assert();
    assert!(
        stdout.contains("Updated") || stdout.contains("pp001"),
        "Expected edit confirmation: {}", stdout
    );
}

#[test]
fn test_page_property_delete_sends_delete_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/wiki/api/v2/pages/1001/properties/pp001");
        then.status(204);
    });

    helpers::assert_success(
        env.cmd().args([
            "confluence", "page", "property", "delete", "1001", "pp001", "--yes",
        ]),
    );

    mock.assert();
}
