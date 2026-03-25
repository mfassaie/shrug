//! Mocked E2E tests for Confluence space commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_space_list_sends_get_v2_spaces() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/spaces");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "results": [
                    {
                        "id": "12345",
                        "key": "DOCS",
                        "name": "Documentation",
                        "type": "global",
                        "status": "current"
                    },
                    {
                        "id": "12346",
                        "key": "ENG",
                        "name": "Engineering",
                        "type": "global",
                        "status": "current"
                    }
                ],
                "_links": {}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "confluence", "space", "list"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    let arr = json.as_array().expect("Expected JSON array of spaces");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["key"], "DOCS");
}

#[test]
fn test_space_create_sends_post_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/wiki/api/v2/spaces")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "55555",
                "key": "NEW",
                "name": "New Space",
                "type": "global",
                "status": "current"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "space", "create",
            "--key", "NEW",
            "--name", "New Space",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["key"], "NEW");
    assert_eq!(json["id"], "55555");
}

#[test]
fn test_space_view_sends_get_v2_by_id() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/spaces/12345");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "12345",
                "key": "DOCS",
                "name": "Documentation",
                "type": "global",
                "status": "current",
                "description": {
                    "plain": {"value": "Internal docs"},
                    "view": {"value": "<p>Internal docs</p>"}
                }
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "confluence", "space", "view", "12345"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["key"], "DOCS");
    assert_eq!(json["name"], "Documentation");
}

#[test]
fn test_space_edit_sends_put_v1() {
    let env = MockEnv::new();

    // Space edit uses v1 API with space key (not v2 ID)
    let mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/wiki/rest/api/space/DOCS")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "key": "DOCS",
                "name": "Updated Docs",
                "type": "global"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "confluence", "space", "edit", "DOCS",
            "--name", "Updated Docs",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Updated") || stdout.contains("DOCS"),
        "Expected edit confirmation: {}",
        stdout
    );
}

#[test]
fn test_space_delete_sends_delete_v1() {
    let env = MockEnv::new();

    // Space delete uses v1 API with space key
    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/wiki/rest/api/space/DOCS");
        then.status(202)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({}));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["confluence", "space", "delete", "DOCS", "--yes"]),
    );

    mock.assert();
    assert!(
        stdout.contains("Deleted") || stdout.contains("DOCS"),
        "Expected delete confirmation: {}",
        stdout
    );
}
