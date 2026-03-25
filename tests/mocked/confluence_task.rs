//! Mocked E2E tests for Confluence task commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_task_list_sends_get_v2() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/tasks");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "results": [
                    {
                        "id": "8001",
                        "status": "incomplete",
                        "spaceId": "12345"
                    }
                ],
                "_links": {}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "confluence", "task", "list"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    let arr = json.as_array().expect("Expected JSON array");
    assert_eq!(arr.len(), 1);
}

#[test]
fn test_task_view_sends_get_v2_by_id() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/wiki/api/v2/tasks/8001");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "8001",
                "status": "incomplete",
                "spaceId": "12345"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "confluence", "task", "view", "8001"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "8001");
}

#[test]
fn test_task_edit_sends_put_v2() {
    let env = MockEnv::new();

    // Task edit: status is positional (confluence task edit {id} {status})
    let mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/wiki/api/v2/tasks/8001")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "8001",
                "status": "complete"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "confluence", "task", "edit", "8001", "complete",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Updated") || stdout.contains("complete") || stdout.contains("8001"),
        "Expected edit confirmation: {}",
        stdout
    );
}
