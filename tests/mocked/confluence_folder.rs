//! Mocked E2E tests for Confluence folder commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_folder_create_sends_post_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/wiki/api/v2/folders")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "5001",
                "title": "Archive",
                "spaceId": "12345"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "folder", "create",
            "--title", "Archive",
            "--space-id", "12345",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "5001");
}

#[test]
fn test_folder_view_sends_get_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/folders/5001");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "5001",
                "title": "Archive",
                "spaceId": "12345"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "confluence", "folder", "view", "5001"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "5001");
}

#[test]
fn test_folder_delete_sends_delete_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/wiki/api/v2/folders/5001");
        then.status(204);
    });

    helpers::assert_success(
        env.cmd()
            .args(["confluence", "folder", "delete", "5001", "--yes"]),
    );

    mock.assert();
}
