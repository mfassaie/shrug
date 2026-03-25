//! Mocked E2E tests for Jira issue watcher commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_watcher_list_sends_get() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issue/TEAM-1/watchers");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "watchCount": 2,
                "isWatching": true,
                "watchers": [
                    {"accountId": "abc123", "displayName": "User One"},
                    {"accountId": "def456", "displayName": "User Two"}
                ]
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira", "issue", "watcher", "list", "TEAM-1"]),
    );

    mock.assert();
    assert!(
        stdout.contains("abc123") || stdout.contains("User One"),
        "Expected watcher info in output: {}",
        stdout
    );
}

#[test]
fn test_watcher_create_sends_post_with_resolved_user() {
    let env = MockEnv::new();

    // Watcher create with --user resolves the user first via /myself,
    // then posts a JSON string body. When --user is an explicit account ID
    // (not @me), we need to mock the myself endpoint for @me resolution
    // since default is @me.
    let myself_mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/myself");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "accountId": "current-user-id",
                "displayName": "Current User"
            }));
    });

    let watcher_mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/issue/TEAM-1/watchers")
            .header("content-type", "application/json");
        then.status(204);
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "issue", "watcher", "create", "TEAM-1",
        ]),
    );

    myself_mock.assert();
    watcher_mock.assert();
    assert!(
        stdout.contains("Added") || stdout.contains("watcher") || stdout.contains("current-user-id"),
        "Expected add confirmation: {}",
        stdout
    );
}

#[test]
fn test_watcher_delete_sends_delete_with_query_param() {
    let env = MockEnv::new();

    // Watcher delete resolves the user, then sends DELETE with ?accountId= query param
    let myself_mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/myself");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "accountId": "current-user-id",
                "displayName": "Current User"
            }));
    });

    let watcher_mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/rest/api/3/issue/TEAM-1/watchers")
            .query_param("accountId", "current-user-id");
        then.status(204);
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "issue", "watcher", "delete", "TEAM-1",
            "--user", "@me", "--yes",
        ]),
    );

    myself_mock.assert();
    watcher_mock.assert();
    assert!(
        stdout.contains("Removed") || stdout.contains("watcher") || stdout.contains("current-user-id"),
        "Expected remove confirmation: {}",
        stdout
    );
}
