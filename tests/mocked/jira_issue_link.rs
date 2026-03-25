//! Mocked E2E tests for Jira issue link commands.

use httpmock::prelude::*;

use crate::helpers::{self, MockEnv};

#[test]
fn test_link_list_sends_get_issue_with_fields() {
    let env = MockEnv::new();

    // Link list fetches the issue with ?fields=issuelinks
    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issue/TEAM-1")
            .query_param("fields", "issuelinks");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "key": "TEAM-1",
                "fields": {
                    "issuelinks": [
                        {
                            "id": "40000",
                            "type": {"name": "Blocks", "inward": "is blocked by", "outward": "blocks"},
                            "outwardIssue": {"key": "TEAM-2", "fields": {"summary": "Blocked issue"}}
                        }
                    ]
                }
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira", "issue", "link", "list", "TEAM-1"]),
    );

    mock.assert();
    assert!(
        stdout.contains("40000") || stdout.contains("Blocks"),
        "Expected link info in output: {}",
        stdout
    );
}

#[test]
fn test_link_create_sends_post_to_issuelink() {
    let env = MockEnv::new();

    // Link create posts to /rest/api/3/issueLink (flat path, not nested)
    let mock = env.server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/issueLink")
            .header("content-type", "application/json");
        then.status(201);
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "issue", "link", "create",
            "--from", "TEAM-1",
            "--to", "TEAM-2",
            "--type", "Blocks",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Created") || stdout.contains("link"),
        "Expected create confirmation: {}",
        stdout
    );
}

#[test]
fn test_link_view_sends_get_by_id() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issueLink/40000");
        then.status(200)
            .header("content-type", "application/json")
            .json_body_obj(&serde_json::json!({
                "id": "40000",
                "type": {"name": "Blocks"},
                "outwardIssue": {"key": "TEAM-2"},
                "inwardIssue": {"key": "TEAM-1"}
            }));
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd()
            .args(["--output", "json", "jira", "issue", "link", "view", "40000"]),
    );

    mock.assert();

    let json = helpers::parse_json(&stdout);
    assert_eq!(json["id"], "40000");
}

#[test]
fn test_link_delete_sends_delete() {
    let env = MockEnv::new();

    let mock = env.server.mock(|when, then| {
        when.method(DELETE)
            .path("/rest/api/3/issueLink/40000");
        then.status(204);
    });

    let (stdout, _stderr) = helpers::assert_success(
        env.cmd().args([
            "jira", "issue", "link", "delete", "40000", "--yes",
        ]),
    );

    mock.assert();
    assert!(
        stdout.contains("Deleted") || stdout.contains("40000"),
        "Expected delete confirmation: {}",
        stdout
    );
}
