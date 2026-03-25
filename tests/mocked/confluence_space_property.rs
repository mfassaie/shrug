//! Mocked E2E tests for Confluence space property commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_space_property_list_sends_get_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/spaces/12345/properties");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "results": [
                    {"id": "90001", "key": "my.prop", "value": {"enabled": true}}
                ],
                "_links": {}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "confluence", "space", "property", "list", "12345"]),
    );

    mock.assert();
    assert!(stdout.contains("my.prop"), "Expected property key in output: {}", stdout);
}

#[test]
fn test_space_property_create_sends_post_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/wiki/api/v2/spaces/12345/properties")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "90002",
                "key": "new.prop",
                "version": {"number": 1}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "space", "property", "create", "12345",
            "--key", "new.prop",
            "--value", r#"{"enabled":true}"#,
        ]),
    );

    mock.assert();
    assert!(stdout.contains("90002"), "Expected property ID: {}", stdout);
}

#[test]
fn test_space_property_view_sends_get_v2_by_id() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/spaces/12345/properties/90001");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "90001",
                "key": "my.prop",
                "value": {"enabled": true},
                "version": {"number": 1}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "space", "property", "view", "12345", "90001",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "90001");
}

#[test]
fn test_space_property_edit_fetches_version_then_puts() {
    let env = MockEnv::new();

    let get_mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/spaces/12345/properties/90001");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "90001",
                "key": "my.prop",
                "value": {"enabled": true},
                "version": {"number": 2}
            }));
    });

    let put_mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/wiki/api/v2/spaces/12345/properties/90001")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "90001",
                "version": {"number": 3}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "confluence", "space", "property", "edit", "12345", "90001",
            "--value", r#"{"enabled":false}"#,
        ]),
    );

    get_mock.assert();
    put_mock.assert();
    assert!(
        stdout.contains("Updated") || stdout.contains("90001"),
        "Expected edit confirmation: {}", stdout
    );
}

#[test]
fn test_space_property_delete_sends_delete_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/wiki/api/v2/spaces/12345/properties/90001");
        then.status(204);
    });

    helpers::assert_success(
        env.cmd().args([
            "confluence", "space", "property", "delete", "12345", "90001", "--yes",
        ]),
    );

    mock.assert();
}
