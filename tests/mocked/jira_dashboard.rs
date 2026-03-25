//! Mocked E2E tests for Jira dashboard commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_dashboard_list_sends_get_search() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/dashboard/search");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "values": [
                    {
                        "id": "10300",
                        "name": "Sprint Board",
                        "owner": {"displayName": "Test User"}
                    }
                ],
                "total": 1,
                "isLast": true
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira", "dashboard", "list"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    let arr = json.as_array().expect("Expected JSON array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "Sprint Board");
}

#[test]
fn test_dashboard_create_sends_post() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/dashboard")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "10400",
                "name": "New Dashboard",
                "self": "https://example.atlassian.net/rest/api/3/dashboard/10400"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira", "dashboard", "create",
            "--name", "New Dashboard",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "10400");
}

#[test]
fn test_dashboard_view_sends_get_by_id() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/dashboard/10300");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "10300",
                "name": "Sprint Board",
                "owner": {"displayName": "Test User", "accountId": "abc123"}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira", "dashboard", "view", "10300"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "10300");
    assert_eq!(json["name"], "Sprint Board");
}

#[test]
fn test_dashboard_edit_sends_put() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/rest/api/3/dashboard/10300")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "10300",
                "name": "Renamed Board"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "dashboard", "edit", "10300",
            "--name", "Renamed Board",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Updated") || stdout.contains("10300"),
        "Expected edit confirmation in stdout: {}",
        stdout
    );
}

#[test]
fn test_dashboard_delete_sends_delete() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/rest/api/3/dashboard/10300");
        then.status(204);
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["jira", "dashboard", "delete", "10300", "--yes"]),
    );

    mock.assert();
    assert!(
        stdout.contains("Deleted") || stdout.contains("10300"),
        "Expected delete confirmation in stdout: {}",
        stdout
    );
}
