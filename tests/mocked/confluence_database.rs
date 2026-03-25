//! Mocked E2E tests for Confluence database commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_database_create_sends_post_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/wiki/api/v2/databases")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "4001",
                "title": "Bug Tracker",
                "spaceId": "12345"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "confluence", "database", "create",
            "--title", "Bug Tracker",
            "--space-id", "12345",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "4001");
}

#[test]
fn test_database_view_sends_get_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/databases/4001");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "4001",
                "title": "Bug Tracker",
                "spaceId": "12345"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "confluence", "database", "view", "4001"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "4001");
}

#[test]
fn test_database_delete_sends_delete_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/wiki/api/v2/databases/4001");
        then.status(204);
    });

    helpers::assert_success(
        env.cmd()
            .args(["confluence", "database", "delete", "4001", "--yes"]),
    );

    mock.assert();
}
