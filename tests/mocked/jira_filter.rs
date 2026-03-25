//! Mocked E2E tests for Jira filter commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_filter_list_sends_get_search() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/filter/search");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "values": [
                    {
                        "id": "10100",
                        "name": "My Bugs",
                        "jql": "type = Bug",
                        "owner": {"displayName": "Test User"}
                    }
                ],
                "total": 1,
                "isLast": true
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira", "filter", "list"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    let arr = json.as_array().expect("Expected JSON array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "My Bugs");
}

#[test]
fn test_filter_create_sends_post() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/filter")
            .header("content-type", "application/json")
            .json_body(serde_json::json!({
                "name": "Sprint Bugs",
                "jql": "type = Bug AND sprint in openSprints()"
            }));
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "10200",
                "name": "Sprint Bugs",
                "self": "https://example.atlassian.net/rest/api/3/filter/10200"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira", "filter", "create",
            "--name", "Sprint Bugs",
            "--jql", "type = Bug AND sprint in openSprints()",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "10200");
}

#[test]
fn test_filter_view_sends_get_by_id() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/filter/10100");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "10100",
                "name": "My Bugs",
                "jql": "type = Bug",
                "owner": {"displayName": "Test User", "accountId": "abc123"}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira", "filter", "view", "10100"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "10100");
    assert_eq!(json["name"], "My Bugs");
}

#[test]
fn test_filter_edit_sends_put() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/rest/api/3/filter/10100")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "10100",
                "name": "Updated Filter"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "filter", "edit", "10100",
            "--name", "Updated Filter",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Updated") || stdout.contains("10100"),
        "Expected edit confirmation in stdout: {}",
        stdout
    );
}

#[test]
fn test_filter_delete_sends_delete() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/rest/api/3/filter/10100");
        then.status(204);
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["jira", "filter", "delete", "10100", "--yes"]),
    );

    mock.assert();
    assert!(
        stdout.contains("Deleted") || stdout.contains("10100"),
        "Expected delete confirmation in stdout: {}",
        stdout
    );
}
