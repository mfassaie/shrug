//! Mocked E2E tests for Jira issue remote link commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_remote_link_list_sends_get() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issue/TEAM-1/remotelink");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!([
                {
                    "id": 50000,
                    "object": {
                        "url": "https://example.com/doc",
                        "title": "Design Doc"
                    }
                }
            ]));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira", "issue", "remote-link", "list", "TEAM-1"]),
    );

    mock.assert();
    assert!(
        stdout.contains("Design Doc") || stdout.contains("50000"),
        "Expected remote link info in output: {}",
        stdout
    );
}

#[test]
fn test_remote_link_create_sends_post() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/issue/TEAM-1/remotelink")
            .header("content-type", "application/json");
        then.status(201)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": 50001,
                "self": "https://example.atlassian.net/rest/api/3/issue/TEAM-1/remotelink/50001"
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira", "issue", "remote-link", "create", "TEAM-1",
            "--url", "https://example.com/wiki",
            "--title", "Wiki Page",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("50001") || stdout.contains("Created"),
        "Expected create confirmation: {}",
        stdout
    );
}

#[test]
fn test_remote_link_view_sends_get_by_id() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issue/TEAM-1/remotelink/50000");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": 50000,
                "object": {
                    "url": "https://example.com/doc",
                    "title": "Design Doc"
                }
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "--output", "json",
            "jira", "issue", "remote-link", "view", "TEAM-1", "50000",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("50000"),
        "Expected remote link in output: {}",
        stdout
    );
}

#[test]
fn test_remote_link_edit_sends_put() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(PUT)
            .path("/rest/api/3/issue/TEAM-1/remotelink/50000")
            .header("content-type", "application/json");
        then.status(204);
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "issue", "remote-link", "edit", "TEAM-1", "50000",
            "--title", "Updated Doc",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Updated") || stdout.contains("50000"),
        "Expected edit confirmation: {}",
        stdout
    );
}

#[test]
fn test_remote_link_delete_sends_delete() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/rest/api/3/issue/TEAM-1/remotelink/50000");
        then.status(204);
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "issue", "remote-link", "delete", "TEAM-1", "50000", "--yes",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Deleted") || stdout.contains("50000"),
        "Expected delete confirmation: {}",
        stdout
    );
}
