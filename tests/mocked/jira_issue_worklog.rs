//! Mocked E2E tests for Jira issue worklog commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_worklog_list_sends_get() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issue/TEAM-1/worklog");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "worklogs": [
                    {
                        "id": "20000",
                        "timeSpent": "2h",
                        "author": {"displayName": "Test User"}
                    }
                ],
                "total": 1,
                "startAt": 0,
                "maxResults": 50
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira", "issue", "worklog", "list", "TEAM-1"]),
    );

    mock.assert();
    assert!(
        stdout.contains("20000"),
        "Expected worklog ID in output: {}",
        stdout
    );
}

#[test]
fn test_worklog_create_sends_post() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/issue/TEAM-1/worklog")
            .header("content-type", "application/json");
        then.status(201)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "20001",
                "timeSpent": "2h",
                "self": "https://example.atlassian.net/rest/api/3/issue/TEAM-1/worklog/20001"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira", "issue", "worklog", "create", "TEAM-1",
            "--time", "2h",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "20001");
}

#[test]
fn test_worklog_view_sends_get_by_id() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issue/TEAM-1/worklog/20000");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "20000",
                "timeSpent": "2h",
                "author": {"displayName": "Test User"}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira", "issue", "worklog", "view", "TEAM-1", "20000",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "20000");
}

#[test]
fn test_worklog_edit_sends_put() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/rest/api/3/issue/TEAM-1/worklog/20000")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "20000",
                "timeSpent": "3h"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "issue", "worklog", "edit", "TEAM-1", "20000",
            "--time", "3h",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Updated") || stdout.contains("20000"),
        "Expected edit confirmation: {}",
        stdout
    );
}

#[test]
fn test_worklog_delete_sends_delete() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/rest/api/3/issue/TEAM-1/worklog/20000");
        then.status(204);
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "issue", "worklog", "delete", "TEAM-1", "20000", "--yes",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Deleted") || stdout.contains("20000"),
        "Expected delete confirmation: {}",
        stdout
    );
}
