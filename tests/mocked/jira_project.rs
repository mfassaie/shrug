//! Mocked E2E tests for Jira project commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_project_list_sends_get_search() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/project/search");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "values": [
                    {
                        "id": "10000",
                        "key": "TEAM",
                        "name": "Team Project",
                        "projectTypeKey": "software"
                    },
                    {
                        "id": "10001",
                        "key": "OPS",
                        "name": "Operations",
                        "projectTypeKey": "business"
                    }
                ],
                "total": 2,
                "isLast": true
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira", "project", "list"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    let arr = json.as_array().expect("Expected JSON array of projects");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["key"], "TEAM");
}

#[test]
fn test_project_create_sends_post() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/project")
            .header("content-type", "application/json");
        then.status(201)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": 10100,
                "key": "NEW",
                "self": "https://example.atlassian.net/rest/api/3/project/10100"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira", "project", "create",
            "--key", "NEW",
            "--name", "New Project",
            "--type", "software",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["key"], "NEW");
}

#[test]
fn test_project_view_sends_get_by_key() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/project/TEAM");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "10000",
                "key": "TEAM",
                "name": "Team Project",
                "projectTypeKey": "software",
                "lead": {
                    "displayName": "Test User",
                    "accountId": "abc123"
                }
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira", "project", "view", "TEAM"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["key"], "TEAM");
    assert_eq!(json["name"], "Team Project");
}

#[test]
fn test_project_edit_sends_put() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/rest/api/3/project/TEAM")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "10000",
                "key": "TEAM",
                "name": "Renamed Project"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "project", "edit", "TEAM",
            "--name", "Renamed Project",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Updated") || stdout.contains("TEAM"),
        "Expected edit confirmation in stdout: {}",
        stdout
    );
}

#[test]
fn test_project_delete_sends_delete() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/rest/api/3/project/TEAM");
        then.status(204);
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["jira", "project", "delete", "TEAM", "--yes"]),
    );

    mock.assert();
    assert!(
        stdout.contains("Deleted") || stdout.contains("TEAM"),
        "Expected delete confirmation in stdout: {}",
        stdout
    );
}
