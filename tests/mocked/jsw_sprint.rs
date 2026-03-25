//! Mocked E2E tests for Jira Software sprint commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_sprint_list_sends_get_by_board() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/agile/1.0/board/42/sprint");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "values": [
                    {
                        "id": 1,
                        "name": "Sprint 1",
                        "state": "active",
                        "originBoardId": 42
                    },
                    {
                        "id": 2,
                        "name": "Sprint 2",
                        "state": "future",
                        "originBoardId": 42
                    }
                ],
                "maxResults": 50,
                "startAt": 0,
                "isLast": true
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira-software", "sprint", "list",
            "--board", "42",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    let arr = json.as_array().expect("Expected JSON array of sprints");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["name"], "Sprint 1");
    assert_eq!(arr[0]["state"], "active");
}

#[test]
fn test_sprint_create_sends_post() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/rest/agile/1.0/sprint")
            .header("content-type", "application/json");
        then.status(201)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": 99,
                "name": "New Sprint",
                "state": "future",
                "originBoardId": 42,
                "self": "https://example.atlassian.net/rest/agile/1.0/sprint/99"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira-software", "sprint", "create",
            "--name", "New Sprint",
            "--board", "42",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], 99);
    assert_eq!(json["name"], "New Sprint");
}

#[test]
fn test_sprint_edit_sends_put() {
    let env = MockEnv::new();

    // Sprint edit now fetches current sprint first (to fill in required name/state)
    let get_mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/agile/1.0/sprint/99");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": 99,
                "name": "Current Sprint",
                "state": "future",
                "originBoardId": 42
            }));
    });

    let put_mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/rest/agile/1.0/sprint/99")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": 99,
                "name": "Renamed Sprint",
                "state": "future",
                "goal": "Ship v2",
                "originBoardId": 42
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira-software", "sprint", "edit", "99",
            "--name", "Renamed Sprint",
            "--goal", "Ship v2",
        ]),
    );

    get_mock.assert();
    put_mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["status"], "updated");
}

#[test]
fn test_sprint_delete_sends_delete() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/rest/agile/1.0/sprint/99");
        then.status(204);
    });

    helpers::assert_success(
        env.cmd()
            .args(["jira-software", "sprint", "delete", "99", "--yes"]),
    );

    mock.assert();
}
