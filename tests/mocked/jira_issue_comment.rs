//! Mocked E2E tests for Jira issue comment commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_comment_list_sends_get() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issue/TEAM-1/comment");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "comments": [
                    {
                        "id": "10000",
                        "body": {"type": "doc", "version": 1, "content": []},
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
            .args(["--output", "json", "jira", "issue", "comment", "list", "TEAM-1"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    // Output may be the full response or extracted comments array
    assert!(
        stdout.contains("10000"),
        "Expected comment ID in output: {}",
        stdout
    );
    let _ = json; // parsed successfully
}

#[test]
fn test_comment_create_sends_post() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/issue/TEAM-1/comment")
            .header("content-type", "application/json");
        then.status(201)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "10001",
                "body": {"type": "doc", "version": 1, "content": []},
                "self": "https://example.atlassian.net/rest/api/3/issue/TEAM-1/comment/10001"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira", "issue", "comment", "create", "TEAM-1",
            "--body", "This is a test comment",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "10001");
}

#[test]
fn test_comment_view_sends_get_by_id() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issue/TEAM-1/comment/10000");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "10000",
                "body": {"type": "doc", "version": 1, "content": []},
                "author": {"displayName": "Test User"}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira", "issue", "comment", "view", "TEAM-1", "10000",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "10000");
}

#[test]
fn test_comment_edit_sends_put() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/rest/api/3/issue/TEAM-1/comment/10000")
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "10000",
                "body": {"type": "doc", "version": 1, "content": []}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "issue", "comment", "edit", "TEAM-1", "10000",
            "--body", "Updated comment",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Updated") || stdout.contains("10000"),
        "Expected edit confirmation: {}",
        stdout
    );
}

#[test]
fn test_comment_delete_sends_delete() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/rest/api/3/issue/TEAM-1/comment/10000");
        then.status(204);
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "issue", "comment", "delete", "TEAM-1", "10000", "--yes",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Deleted") || stdout.contains("10000"),
        "Expected delete confirmation: {}",
        stdout
    );
}
