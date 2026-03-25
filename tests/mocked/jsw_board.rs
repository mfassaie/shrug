//! Mocked E2E tests for Jira Software board commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_board_list_sends_get_all_boards() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/agile/1.0/board");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "values": [
                    {
                        "id": 1,
                        "name": "TEAM board",
                        "type": "scrum"
                    },
                    {
                        "id": 2,
                        "name": "OPS board",
                        "type": "kanban"
                    }
                ],
                "maxResults": 50,
                "startAt": 0,
                "isLast": true
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira-software", "board", "list"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    let arr = json.as_array().expect("Expected JSON array of boards");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["name"], "TEAM board");
}

#[test]
fn test_board_create_sends_post() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/rest/agile/1.0/board")
            .header("content-type", "application/json");
        then.status(201)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": 42,
                "name": "Test Board",
                "type": "scrum",
                "self": "https://example.atlassian.net/rest/agile/1.0/board/42"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira-software", "board", "create",
            "--name", "Test Board",
            "--type", "scrum",
            "--filter-id", "100",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], 42);
    assert_eq!(json["name"], "Test Board");
}

#[test]
fn test_board_view_sends_get_by_id() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/agile/1.0/board/42");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": 42,
                "name": "Test Board",
                "type": "scrum",
                "self": "https://example.atlassian.net/rest/agile/1.0/board/42"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira-software", "board", "view", "42"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], 42);
    assert_eq!(json["name"], "Test Board");
}

#[test]
fn test_board_delete_sends_delete() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/rest/agile/1.0/board/42");
        then.status(204);
    });

    helpers::assert_success(
        env.cmd()
            .args(["jira-software", "board", "delete", "42", "--yes"]),
    );

    mock.assert();
}

#[test]
fn test_board_config_sends_get_configuration() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/agile/1.0/board/42/configuration");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": 42,
                "name": "Test Board",
                "type": "scrum",
                "filter": {"id": "100", "self": "https://example.atlassian.net/rest/api/3/filter/100"}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira-software", "board", "config", "42"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], 42);
}
