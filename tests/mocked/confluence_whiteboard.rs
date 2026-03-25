//! Mocked E2E tests for Confluence whiteboard commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_whiteboard_create_sends_post_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/wiki/api/v2/whiteboards")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "3001",
                "title": "Sprint Planning",
                "spaceId": "12345"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "whiteboard", "create",
            "--title", "Sprint Planning",
            "--space-id", "12345",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "3001");
}

#[test]
fn test_whiteboard_view_sends_get_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/whiteboards/3001");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "3001",
                "title": "Sprint Planning",
                "spaceId": "12345"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "confluence", "whiteboard", "view", "3001"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "3001");
}

#[test]
fn test_whiteboard_delete_sends_delete_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/wiki/api/v2/whiteboards/3001");
        then.status(204);
    });

    helpers::assert_success(
        env.cmd()
            .args(["confluence", "whiteboard", "delete", "3001", "--yes"]),
    );

    mock.assert();
}
