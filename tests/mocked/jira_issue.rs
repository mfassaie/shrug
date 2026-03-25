//! Mocked E2E tests for Jira issue commands.
//!
//! Verifies the CLI sends correct HTTP requests to the Jira REST API
//! and formats responses properly.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_issue_list_sends_get_search() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/search/jql");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "issues": [
                    {
                        "key": "TEAM-1",
                        "fields": {
                            "summary": "First issue",
                            "status": {"name": "Open"}
                        }
                    },
                    {
                        "key": "TEAM-2",
                        "fields": {
                            "summary": "Second issue",
                            "status": {"name": "Done"}
                        }
                    }
                ],
                "total": 2,
                "startAt": 0,
                "maxResults": 50
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira", "issue", "list", "--project", "TEAM"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    let arr = json.as_array().expect("Expected JSON array");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["key"], "TEAM-1");
    assert_eq!(arr[1]["key"], "TEAM-2");
}

#[test]
fn test_issue_create_sends_post_issue() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/issue")
            .header("content-type", "application/json")
            .json_body(serde_json::json!({"fields":{"summary":"Test bug","project":{"key":"TEAM"},"issuetype":{"name":"Bug"}}}));
        then.status(201)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "10001",
                "key": "TEAM-42",
                "self": "https://example.atlassian.net/rest/api/3/issue/10001"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira", "issue", "create",
            "-s", "Test bug",
            "--project", "TEAM",
            "--type", "Bug",
        ]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["key"], "TEAM-42");
}

#[test]
fn test_issue_view_sends_get_with_key() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issue/TEAM-123");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "key": "TEAM-123",
                "fields": {
                    "summary": "Existing issue",
                    "status": {"name": "In Progress"},
                    "issuetype": {"name": "Task"},
                    "priority": {"name": "Medium"}
                }
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira", "issue", "view", "TEAM-123"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["key"], "TEAM-123");
    assert_eq!(json["fields"]["summary"], "Existing issue");
}

#[test]
fn test_issue_edit_sends_put() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/rest/api/3/issue/TEAM-10")
            .header("content-type", "application/json");
        then.status(204);
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "issue", "edit", "TEAM-10",
            "-s", "Updated summary",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Updated") || stdout.contains("TEAM-10"),
        "Expected edit confirmation in stdout: {}",
        stdout
    );
}

#[test]
fn test_issue_delete_sends_delete_with_yes() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/rest/api/3/issue/TEAM-99");
        then.status(204);
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["jira", "issue", "delete", "TEAM-99", "--yes"]),
    );

    mock.assert();
    assert!(
        stdout.contains("Deleted") || stdout.contains("TEAM-99"),
        "Expected delete confirmation in stdout: {}",
        stdout
    );
}

#[test]
fn test_issue_dry_run_does_not_send_request() {
    let env = MockEnv::new();

    // No mock registered: if the CLI sends a request, httpmock will not match
    // and the CLI will get an unexpected response. Dry run should skip the request.
    let (stdout, stderr) = helpers::assert_success(
        env.cmd().args([
            "--dry-run",
            "jira", "issue", "create",
            "-s", "Dry run test",
            "--project", "TEAM",
            "--type", "Task",
        ]),
    );

    // dry_run_request writes to stderr
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        combined.contains("POST") && combined.contains("/rest/api/3/issue"),
        "Dry run should show the method and URL.\ncombined output: {}",
        combined
    );
}
